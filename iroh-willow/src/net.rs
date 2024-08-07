use std::future::Future;

use anyhow::{ensure, Context as _, Result};
use futures_concurrency::future::TryJoin;
use futures_lite::future::Boxed;
use futures_util::future::TryFutureExt;
use iroh_base::key::NodeId;
use iroh_net::endpoint::{get_remote_node_id, Connection, RecvStream, SendStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{debug, instrument, trace};

use crate::{
    proto::sync::{
        AccessChallenge, ChallengeHash, Channel, InitialTransmission, LogicalChannel, Message,
        CHALLENGE_HASH_LENGTH, MAX_PAYLOAD_SIZE_POWER,
    },
    session::{
        channels::{
            ChannelReceivers, ChannelSenders, Channels, LogicalChannelReceivers,
            LogicalChannelSenders,
        },
        Role,
    },
    util::channel::{
        inbound_channel, outbound_channel, Guarantees, Reader, Receiver, Sender, Writer,
    },
};

pub const CHANNEL_CAP: usize = 1024 * 64;
pub const ALPN: &[u8] = b"iroh-willow/0";

pub type ConnRunFut = Boxed<Result<()>>;

/// Wrapper around [`iroh_net::endpoint::Connection`] that keeps the remote node's [`NodeId`] and
/// our role (whether we accepted or initiated the connection).
// TODO: Integrate this into iroh_net::endpoint::Connection by making that a struct and not a reexport? Seems universally useful.
#[derive(Debug, Clone)]
pub struct PeerConn {
    our_role: Role,
    remote_node_id: NodeId,
    node_id: NodeId,
    conn: iroh_net::endpoint::Connection,
}

impl std::ops::Deref for PeerConn {
    type Target = iroh_net::endpoint::Connection;
    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl PeerConn {
    pub fn new(conn: iroh_net::endpoint::Connection, our_role: Role, me: NodeId) -> Result<Self> {
        let peer = get_remote_node_id(&conn)?;
        Ok(Self {
            conn,
            node_id: me,
            remote_node_id: peer,
            our_role,
        })
    }
    pub fn peer(&self) -> NodeId {
        self.remote_node_id
    }

    pub fn me(&self) -> NodeId {
        self.node_id
    }

    pub fn our_role(&self) -> Role {
        self.our_role
    }
}

pub async fn dial_and_establish(
    endpoint: &iroh_net::Endpoint,
    node_id: NodeId,
    our_nonce: AccessChallenge,
) -> Result<(PeerConn, InitialTransmission)> {
    let conn = endpoint.connect_by_node_id(&node_id, ALPN).await?;
    let conn = PeerConn::new(conn, Role::Alfie, endpoint.node_id())?;
    let initial_transmission = establish(&conn, our_nonce).await?;
    Ok((conn, initial_transmission))
}

pub async fn establish(conn: &PeerConn, our_nonce: AccessChallenge) -> Result<InitialTransmission> {
    debug!(our_role=?conn.our_role(), "start initial transmission");
    let challenge_hash = our_nonce.hash();
    let mut send_stream = conn.open_uni().await?;
    send_stream.write_u8(MAX_PAYLOAD_SIZE_POWER).await?;
    send_stream.write_all(challenge_hash.as_bytes()).await?;

    let mut recv_stream = conn.accept_uni().await?;

    let their_max_payload_size = {
        let power = recv_stream.read_u8().await?;
        ensure!(power <= 64, "max payload size too large");
        2u64.pow(power as u32)
    };

    let mut received_commitment = [0u8; CHALLENGE_HASH_LENGTH];
    recv_stream.read_exact(&mut received_commitment).await?;
    debug!(our_role=?conn.our_role(), "initial transmission complete");
    Ok(InitialTransmission {
        our_nonce,
        received_commitment: ChallengeHash::from_bytes(received_commitment),
        their_max_payload_size,
    })

    // let our_role = conn.our_role();
    // let (mut setup_send, mut setup_recv) = match our_role {
    //     Role::Alfie => conn.open_bi().await?,
    //     Role::Betty => conn.accept_bi().await?,
    // };
    // debug!("setup channel ready");

    // let initial_transmission =
    //     exchange_commitments(&mut setup_send, &mut setup_recv, our_nonce).await?;
    // Ok(initial_transmission)
}

pub async fn setup(
    conn: &PeerConn,
) -> Result<(Channels, impl Future<Output = Result<()>> + Send + 'static)> {
    let our_role = conn.our_role();
    let (channels, fut) = launch_channels(&conn, our_role).await?;
    Ok((channels, fut))
}

#[derive(derive_more::Debug)]
pub struct WillowConn {
    pub(crate) our_role: Role,
    pub(crate) peer: NodeId,
    #[debug("InitialTransmission")]
    pub(crate) initial_transmission: InitialTransmission,
    #[debug("Channels")]
    pub(crate) channels: Channels,
}

impl WillowConn {
    #[cfg(test)]
    #[instrument(skip_all, name = "conn", fields(me=%me.fmt_short(), peer=tracing::field::Empty))]
    async fn connect(
        conn: Connection,
        me: NodeId,
        our_role: Role,
        our_nonce: AccessChallenge,
    ) -> Result<Self> {
        let conn = PeerConn::new(conn, our_role, me)?;
        tracing::Span::current().record("peer", tracing::field::display(conn.peer().fmt_short()));
        let initial_transmission = establish(&conn, our_nonce).await?;
        let (channels, fut) = setup(&conn).await?;
        tokio::task::spawn(fut);
        Ok(WillowConn {
            initial_transmission,
            our_role,
            peer: conn.peer(),
            channels,
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("missing channel: {0:?}")]
struct MissingChannel(Channel);

type ChannelStreams = [(Channel, SendStream, RecvStream); Channel::COUNT];

async fn open_channels(conn: &Connection, our_role: Role) -> Result<ChannelStreams> {
    let channels = match our_role {
        // Alfie opens a quic stream for each logical channel, and sends a single byte with the
        // channel id.
        Role::Alfie => {
            Channel::all()
                .map(|ch| {
                    let conn = conn.clone();
                    async move {
                        let (mut send, recv) = conn.open_bi().await?;
                        send.write_u8(ch.id()).await?;
                        trace!(?ch, "opened bi stream");
                        Ok::<_, anyhow::Error>((ch, send, recv))
                    }
                })
                .try_join()
                .await
        }
        // Betty accepts as many quick streams as there are logical channels, and reads a single
        // byte on each, which is expected to contain a channel id.
        Role::Betty => {
            Channel::all()
                .map(|_| async {
                    let (send, mut recv) = conn.accept_bi().await?;
                    // trace!("accepted bi stream");
                    let channel_id = recv.read_u8().await?;
                    // trace!("read channel id {channel_id}");
                    let channel = Channel::from_id(channel_id)?;
                    trace!(?channel, "accepted bi stream for channel");
                    Result::Ok((channel, send, recv))
                })
                .try_join()
                .await
        }
    }?;
    Ok(channels)
}

fn start_channels(
    channels: ChannelStreams,
) -> Result<(Channels, impl Future<Output = Result<()>> + Send)> {
    let mut channels = channels.map(|(ch, send, recv)| (ch, Some(prepare_channel(ch, send, recv))));

    let mut find = |channel| {
        channels
            .iter_mut()
            .find_map(|(ch, streams)| (*ch == channel).then(|| streams.take()))
            .flatten()
            .ok_or(MissingChannel(channel))
    };

    let ctrl = find(Channel::Control)?;
    let pai = find(Channel::Logical(LogicalChannel::Intersection))?;
    let rec = find(Channel::Logical(LogicalChannel::Reconciliation))?;
    let stt = find(Channel::Logical(LogicalChannel::StaticToken))?;
    let aoi = find(Channel::Logical(LogicalChannel::AreaOfInterest))?;
    let cap = find(Channel::Logical(LogicalChannel::Capability))?;
    let dat = find(Channel::Logical(LogicalChannel::Data))?;

    let fut = (ctrl.2, pai.2, rec.2, stt.2, aoi.2, cap.2, dat.2)
        .try_join()
        .map_ok(|_| ());

    let logical_send = LogicalChannelSenders {
        intersection_send: pai.0,
        reconciliation_send: rec.0,
        static_tokens_send: stt.0,
        aoi_send: aoi.0,
        capability_send: cap.0,
        data_send: dat.0,
    };
    let logical_recv = LogicalChannelReceivers {
        intersection_recv: pai.1.into(),
        reconciliation_recv: rec.1.into(),
        static_tokens_recv: stt.1.into(),
        aoi_recv: aoi.1.into(),
        capability_recv: cap.1.into(),
        data_recv: dat.1.into(),
    };
    let channels = Channels {
        send: ChannelSenders {
            control_send: ctrl.0,
            logical_send,
        },
        recv: ChannelReceivers {
            control_recv: ctrl.1,
            logical_recv,
        },
    };
    Ok((channels, fut))
}

async fn launch_channels(
    conn: &Connection,
    our_role: Role,
) -> Result<(Channels, impl Future<Output = Result<()>> + Send)> {
    let channels = open_channels(conn, our_role).await?;
    start_channels(channels)
}

fn prepare_channel(
    ch: Channel,
    send_stream: SendStream,
    recv_stream: RecvStream,
) -> (
    Sender<Message>,
    Receiver<Message>,
    impl Future<Output = Result<()>> + Send,
) {
    let guarantees = match ch {
        Channel::Control => Guarantees::Unlimited,
        Channel::Logical(_) => Guarantees::Limited(0),
    };
    let cap = CHANNEL_CAP;
    let (sender, outbound_reader) = outbound_channel(cap, guarantees);
    let (inbound_writer, receiver) = inbound_channel(cap);

    let recv_fut = recv_loop(recv_stream, inbound_writer)
        .map_err(move |e| e.context(format!("receive loop for {ch:?} failed")));

    let send_fut = send_loop(send_stream, outbound_reader)
        .map_err(move |e| e.context(format!("send loop for {ch:?} failed")));

    let fut = (recv_fut, send_fut).try_join().map_ok(|_| ());

    (sender, receiver, fut)
}

async fn recv_loop(mut recv_stream: RecvStream, mut channel_writer: Writer) -> Result<()> {
    trace!("recv: start");
    let max_buffer_size = channel_writer.max_buffer_size();
    while let Some(buf) = recv_stream
        .read_chunk(max_buffer_size, true)
        .await
        .context("failed to read from quic stream")?
    {
        // trace!(len = buf.bytes.len(), "read");
        channel_writer.write_all(&buf.bytes[..]).await?;
        // trace!(len = buf.bytes.len(), "sent");
    }
    trace!("recv: stream close");
    channel_writer.close();
    trace!("recv: loop close");
    Ok(())
}

async fn send_loop(mut send_stream: SendStream, channel_reader: Reader) -> Result<()> {
    trace!("send: start");
    while let Some(data) = channel_reader.read_bytes().await {
        // let len = data.len();
        // trace!(len, "send");
        send_stream
            .write_chunk(data)
            .await
            .context("failed to write to quic stream")?;
        // trace!(len, "sent");
    }
    trace!("send: close writer");
    send_stream.finish().await?;
    trace!("send: done");
    Ok(())
}

async fn exchange_commitments(
    send_stream: &mut SendStream,
    recv_stream: &mut RecvStream,
    our_nonce: AccessChallenge,
) -> Result<InitialTransmission> {
    let challenge_hash = our_nonce.hash();
    send_stream.write_u8(MAX_PAYLOAD_SIZE_POWER).await?;
    send_stream.write_all(challenge_hash.as_bytes()).await?;

    let their_max_payload_size = {
        let power = recv_stream.read_u8().await?;
        ensure!(power <= 64, "max payload size too large");
        2u64.pow(power as u32)
    };

    let mut received_commitment = [0u8; CHALLENGE_HASH_LENGTH];
    recv_stream.read_exact(&mut received_commitment).await?;
    Ok(InitialTransmission {
        our_nonce,
        received_commitment: ChallengeHash::from_bytes(received_commitment),
        their_max_payload_size,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeSet,
        time::{Duration, Instant},
    };

    use futures_lite::StreamExt;
    use iroh_base::key::SecretKey;
    use iroh_net::{endpoint::Connection, Endpoint, NodeAddr, NodeId};
    use rand::SeedableRng;
    use rand_chacha::ChaCha12Rng;
    use tracing::info;

    use crate::{
        auth::{CapSelector, DelegateTo, RestrictArea},
        engine::ActorHandle,
        form::{AuthForm, EntryForm, PayloadForm, SubspaceForm, TimestampForm},
        net::WillowConn,
        proto::{
            grouping::ThreeDRange,
            keys::{NamespaceId, NamespaceKind, UserId},
            meadowcap::AccessMode,
            sync::AccessChallenge,
            willow::{Entry, InvalidPath, Path},
        },
        session::{intents::Intent, Interests, Role, SessionHandle, SessionInit, SessionMode},
    };

    const ALPN: &[u8] = b"iroh-willow/0";

    fn create_rng(seed: &str) -> ChaCha12Rng {
        let seed = iroh_base::hash::Hash::new(seed);
        rand_chacha::ChaCha12Rng::from_seed(*(seed.as_bytes()))
    }

    pub async fn run(
        me: NodeId,
        actor: ActorHandle,
        conn: Connection,
        our_role: Role,
        our_nonce: AccessChallenge,
        intents: Vec<Intent>,
    ) -> anyhow::Result<SessionHandle> {
        let conn = WillowConn::connect(conn, me, our_role, our_nonce).await?;
        let handle = actor.init_session(conn, intents).await?;
        Ok(handle)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn net_smoke() -> anyhow::Result<()> {
        iroh_test::logging::setup_multithreaded();
        let mut rng = create_rng("net_smoke");
        let n_betty = parse_env_var("N_BETTY", 100);
        let n_alfie = parse_env_var("N_ALFIE", 100);

        let (ep_alfie, node_id_alfie, _) = create_endpoint(&mut rng).await?;
        let (ep_betty, node_id_betty, addr_betty) = create_endpoint(&mut rng).await?;

        let start = Instant::now();
        let mut expected_entries = BTreeSet::new();

        let handle_alfie = ActorHandle::spawn_memory(Default::default(), node_id_alfie);
        let handle_betty = ActorHandle::spawn_memory(Default::default(), node_id_betty);

        let user_alfie = handle_alfie.create_user().await?;
        let user_betty = handle_betty.create_user().await?;

        let namespace_id = handle_alfie
            .create_namespace(NamespaceKind::Owned, user_alfie)
            .await?;

        let cap_for_betty = handle_alfie
            .delegate_caps(
                CapSelector::widest(namespace_id),
                AccessMode::ReadWrite,
                DelegateTo::new(user_betty, RestrictArea::None),
            )
            .await?;

        handle_betty.import_caps(cap_for_betty).await?;

        insert(
            &handle_alfie,
            namespace_id,
            user_alfie,
            n_alfie,
            |n| Path::new(&[b"alfie", n.to_string().as_bytes()]),
            |n| format!("alfie{n}"),
            &mut expected_entries,
        )
        .await?;

        insert(
            &handle_betty,
            namespace_id,
            user_betty,
            n_betty,
            |n| Path::new(&[b"betty", n.to_string().as_bytes()]),
            |n| format!("betty{n}"),
            &mut expected_entries,
        )
        .await?;

        let init_alfie = SessionInit::new(Interests::All, SessionMode::ReconcileOnce);
        let init_betty = SessionInit::new(Interests::All, SessionMode::ReconcileOnce);
        let (intent_alfie, mut intent_handle_alfie) = Intent::new(init_alfie);
        let (intent_betty, mut intent_handle_betty) = Intent::new(init_betty);

        info!("init took {:?}", start.elapsed());

        let start = Instant::now();
        let (conn_alfie, conn_betty) = tokio::join!(
            async move { ep_alfie.connect(addr_betty, ALPN).await.unwrap() },
            async move { ep_betty.accept().await.unwrap().await.unwrap() }
        );
        info!("connecting took {:?}", start.elapsed());

        let start = Instant::now();
        let nonce_alfie = AccessChallenge::generate_with_rng(&mut rng);
        let nonce_betty = AccessChallenge::generate_with_rng(&mut rng);
        let (session_alfie, session_betty) = tokio::join!(
            run(
                node_id_alfie,
                handle_alfie.clone(),
                conn_alfie,
                Role::Alfie,
                nonce_alfie,
                vec![intent_alfie]
            ),
            run(
                node_id_betty,
                handle_betty.clone(),
                conn_betty,
                Role::Betty,
                nonce_betty,
                vec![intent_betty]
            )
        );
        let mut session_alfie = session_alfie?;
        let mut session_betty = session_betty?;

        let (res_alfie, res_betty) = tokio::join!(
            intent_handle_alfie.complete(),
            intent_handle_betty.complete()
        );
        info!("alfie intent res {:?}", res_alfie);
        info!("betty intent res {:?}", res_betty);
        assert!(res_alfie.is_ok());
        assert!(res_betty.is_ok());

        let (res_alfie, res_betty) =
            tokio::join!(session_alfie.complete(), session_betty.complete());
        info!("alfie session res {:?}", res_alfie);
        info!("betty session res {:?}", res_betty);
        assert!(res_alfie.is_ok());
        assert!(res_betty.is_ok());

        info!(time=?start.elapsed(), "reconciliation finished");

        let alfie_entries = get_entries(&handle_alfie, namespace_id).await?;
        let betty_entries = get_entries(&handle_betty, namespace_id).await?;
        info!("alfie has now {} entries", alfie_entries.len());
        info!("betty has now {} entries", betty_entries.len());
        // not using assert_eq because it would print a lot in case of failure
        assert!(alfie_entries == expected_entries, "alfie expected entries");
        assert!(betty_entries == expected_entries, "betty expected entries");

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn net_live_data() -> anyhow::Result<()> {
        iroh_test::logging::setup_multithreaded();
        let mut rng = create_rng("net_live_data");

        let (ep_alfie, node_id_alfie, _) = create_endpoint(&mut rng).await?;
        let (ep_betty, node_id_betty, addr_betty) = create_endpoint(&mut rng).await?;

        let handle_alfie = ActorHandle::spawn_memory(Default::default(), node_id_alfie);
        let handle_betty = ActorHandle::spawn_memory(Default::default(), node_id_betty);

        let user_alfie = handle_alfie.create_user().await?;
        let user_betty = handle_betty.create_user().await?;

        let namespace_id = handle_alfie
            .create_namespace(NamespaceKind::Owned, user_alfie)
            .await?;

        let cap_for_betty = handle_alfie
            .delegate_caps(
                CapSelector::widest(namespace_id),
                AccessMode::ReadWrite,
                DelegateTo::new(user_betty, RestrictArea::None),
            )
            .await?;

        handle_betty.import_caps(cap_for_betty).await?;

        let mut expected_entries = BTreeSet::new();
        let start = Instant::now();

        let n_init = 2;
        insert(
            &handle_alfie,
            namespace_id,
            user_alfie,
            n_init,
            |n| Path::new(&[b"alfie-init", n.to_string().as_bytes()]),
            |n| format!("alfie{n}"),
            &mut expected_entries,
        )
        .await?;

        insert(
            &handle_betty,
            namespace_id,
            user_betty,
            n_init,
            |n| Path::new(&[b"betty-init", n.to_string().as_bytes()]),
            |n| format!("betty{n}"),
            &mut expected_entries,
        )
        .await?;

        info!("init took {:?}", start.elapsed());

        let start = Instant::now();
        let (conn_alfie, conn_betty) = tokio::join!(
            async move { ep_alfie.connect(addr_betty, ALPN).await.unwrap() },
            async move { ep_betty.accept().await.unwrap().await.unwrap() }
        );
        info!("connecting took {:?}", start.elapsed());

        let start = Instant::now();
        let (done_tx, done_rx) = tokio::sync::oneshot::channel();

        // alfie insert 3 enries after waiting a second
        let _insert_task_alfie = tokio::task::spawn({
            let handle_alfie = handle_alfie.clone();
            let count = 3;
            let content_fn = |i: usize| format!("alfie live {i}");
            let path_fn = |i: usize| Path::new(&[b"alfie-live", i.to_string().as_bytes()]);
            let mut track_entries = vec![];

            async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                insert(
                    &handle_alfie,
                    namespace_id,
                    user_alfie,
                    count,
                    path_fn,
                    content_fn,
                    &mut track_entries,
                )
                .await
                .expect("failed to insert");
                done_tx.send(track_entries).unwrap();
            }
        });

        let init_alfie = SessionInit::new(Interests::All, SessionMode::Live);
        let init_betty = SessionInit::new(Interests::All, SessionMode::Live);

        let (intent_alfie, mut intent_handle_alfie) = Intent::new(init_alfie);
        let (intent_betty, mut intent_handle_betty) = Intent::new(init_betty);

        let nonce_alfie = AccessChallenge::generate_with_rng(&mut rng);
        let nonce_betty = AccessChallenge::generate_with_rng(&mut rng);

        let (session_alfie, session_betty) = tokio::join!(
            run(
                node_id_alfie,
                handle_alfie.clone(),
                conn_alfie,
                Role::Alfie,
                nonce_alfie,
                vec![intent_alfie]
            ),
            run(
                node_id_betty,
                handle_betty.clone(),
                conn_betty,
                Role::Betty,
                nonce_betty,
                vec![intent_betty]
            )
        );
        let mut session_alfie = session_alfie?;
        let mut session_betty = session_betty?;

        let live_entries = done_rx.await?;
        expected_entries.extend(live_entries);
        // TODO: replace with event
        tokio::time::sleep(Duration::from_secs(1)).await;
        session_alfie.close();

        let (res_alfie, res_betty) = tokio::join!(
            intent_handle_alfie.complete(),
            intent_handle_betty.complete()
        );
        info!(time=?start.elapsed(), "reconciliation finished");
        info!("alfie intent res {:?}", res_alfie);
        info!("betty intent res {:?}", res_betty);
        assert!(res_alfie.is_ok());
        assert!(res_betty.is_ok());

        let (res_alfie, res_betty) =
            tokio::join!(session_alfie.complete(), session_betty.complete());

        info!("alfie session res {:?}", res_alfie);
        info!("betty session res {:?}", res_betty);
        assert!(res_alfie.is_ok());
        assert!(res_betty.is_ok());
        let alfie_entries = get_entries(&handle_alfie, namespace_id).await?;
        let betty_entries = get_entries(&handle_betty, namespace_id).await?;
        info!("alfie has now {} entries", alfie_entries.len());
        info!("betty has now {} entries", betty_entries.len());
        // not using assert_eq because it would print a lot in case of failure
        assert!(alfie_entries == expected_entries, "alfie expected entries");
        assert!(betty_entries == expected_entries, "betty expected entries");

        Ok(())
    }

    pub async fn create_endpoint(
        rng: &mut rand_chacha::ChaCha12Rng,
    ) -> anyhow::Result<(Endpoint, NodeId, NodeAddr)> {
        let ep = Endpoint::builder()
            .secret_key(SecretKey::generate_with_rng(rng))
            .alpns(vec![ALPN.to_vec()])
            .bind(0)
            .await?;
        let addr = ep.node_addr().await?;
        let node_id = ep.node_id();
        Ok((ep, node_id, addr))
    }

    async fn get_entries(
        store: &ActorHandle,
        namespace: NamespaceId,
    ) -> anyhow::Result<BTreeSet<Entry>> {
        let entries: anyhow::Result<BTreeSet<_>> = store
            .get_entries(namespace, ThreeDRange::full())
            .await?
            .try_collect()
            .await;
        entries
    }

    async fn insert(
        handle: &ActorHandle,
        namespace_id: NamespaceId,
        user_id: UserId,
        count: usize,
        path_fn: impl Fn(usize) -> Result<Path, InvalidPath>,
        content_fn: impl Fn(usize) -> String,
        track_entries: &mut impl Extend<Entry>,
    ) -> anyhow::Result<()> {
        for i in 0..count {
            let payload = content_fn(i).as_bytes().to_vec();
            let path = path_fn(i).expect("invalid path");
            let entry = EntryForm {
                namespace_id,
                subspace_id: SubspaceForm::User,
                path,
                timestamp: TimestampForm::Now,
                payload: PayloadForm::Bytes(payload.into()),
            };
            let (entry, inserted) = handle.insert(entry, AuthForm::Any(user_id)).await?;
            assert!(inserted);
            track_entries.extend([entry]);
        }
        Ok(())
    }

    fn parse_env_var<T>(var: &str, default: T) -> T
    where
        T: std::str::FromStr,
        T::Err: std::fmt::Debug,
    {
        match std::env::var(var).as_deref() {
            Ok(val) => val
                .parse()
                .unwrap_or_else(|_| panic!("failed to parse environment variable {var}")),
            Err(_) => default,
        }
    }

    // async fn get_entries_debug(
    //     store: &StoreHandle,
    //     namespace: NamespaceId,
    // ) -> anyhow::Result<Vec<(SubspaceId, Path)>> {
    //     let entries = get_entries(store, namespace).await?;
    //     let mut entries: Vec<_> = entries
    //         .into_iter()
    //         .map(|e| (e.subspace_id, e.path))
    //         .collect();
    //     entries.sort();
    //     Ok(entries)
    // }
    //
    //
    //
    // tokio::task::spawn({
    //     let handle_alfie = handle_alfie.clone();
    //     let handle_betty = handle_betty.clone();
    //     async move {
    //         loop {
    //             info!(
    //                 "alfie count: {}",
    //                 handle_alfie
    //                     .get_entries(namespace_id, ThreeDRange::full())
    //                     .await
    //                     .unwrap()
    //                     .count()
    //                     .await
    //             );
    //             info!(
    //                 "betty count: {}",
    //                 handle_betty
    //                     .get_entries(namespace_id, ThreeDRange::full())
    //                     .await
    //                     .unwrap()
    //                     .count()
    //                     .await
    //             );
    //             tokio::time::sleep(Duration::from_secs(1)).await;
    //         }
    //     }
    // });
}
