(function() {
    var type_impls = Object.fromEntries([["iroh_willow",[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-AuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+UserPublicKey,+PD%3E-for-McAuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+NamespaceSignature,+UserPublicKey,+UserSignature%3E\" class=\"impl\"><a href=\"#impl-AuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+UserPublicKey,+PD%3E-for-McAuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+NamespaceSignature,+UserPublicKey,+UserSignature%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;const MCL: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, const MCC: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, const MPL: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature, PD&gt; AuthorisationToken&lt;MCL, MCC, MPL, NamespacePublicKey, UserPublicKey, PD&gt; for McAuthorisationToken&lt;MCL, MCC, MPL, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt;<div class=\"where\">where\n    NamespacePublicKey: NamespaceId + Encodable + Verifier&lt;NamespaceSignature&gt; + <a class=\"trait\" href=\"iroh_willow/proto/meadowcap/trait.IsCommunal.html\" title=\"trait iroh_willow::proto::meadowcap::IsCommunal\">IsCommunal</a>,\n    UserPublicKey: SubspaceId + Encodable + Verifier&lt;UserSignature&gt;,\n    NamespaceSignature: Encodable + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    UserSignature: Encodable + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    PD: PayloadDigest + Encodable,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.is_authorised_write\" class=\"method trait-impl\"><a href=\"#method.is_authorised_write\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a class=\"fn\">is_authorised_write</a>(\n    &amp;self,\n    entry: &amp;Entry&lt;MCL, MCC, MPL, NamespacePublicKey, UserPublicKey, PD&gt;,\n) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.bool.html\">bool</a></h4></section></summary><div class='docblock'>Determine whether this type (nominally a <a href=\"https://willowprotocol.org/specs/data-model/index.html#AuthorisationToken\"><code>AuthorisationToken</code></a>) is able to prove write permission for a given [<code>Entry</code>].</div></details></div></details>","AuthorisationToken<MCL, MCC, MPL, NamespacePublicKey, UserPublicKey, PD>","iroh_willow::proto::meadowcap::McAuthorisationToken"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-McAuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+NamespaceSignature,+UserPublicKey,+UserSignature%3E\" class=\"impl\"><a href=\"#impl-Clone-for-McAuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+NamespaceSignature,+UserPublicKey,+UserSignature%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;const MCL: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, const MCC: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, const MPL: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for McAuthorisationToken&lt;MCL, MCC, MPL, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt;<div class=\"where\">where\n    NamespacePublicKey: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + NamespaceId + Encodable + Verifier&lt;NamespaceSignature&gt; + <a class=\"trait\" href=\"iroh_willow/proto/meadowcap/trait.IsCommunal.html\" title=\"trait iroh_willow::proto::meadowcap::IsCommunal\">IsCommunal</a>,\n    NamespaceSignature: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + Encodable,\n    UserPublicKey: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + SubspaceId + Encodable + Verifier&lt;UserSignature&gt;,\n    UserSignature: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + Encodable,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(\n    &amp;self,\n) -&gt; McAuthorisationToken&lt;MCL, MCC, MPL, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt;</h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/clone.rs.html#174\">Source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: &amp;Self)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","iroh_willow::proto::meadowcap::McAuthorisationToken"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-McAuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+NamespaceSignature,+UserPublicKey,+UserSignature%3E\" class=\"impl\"><a href=\"#impl-Debug-for-McAuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+NamespaceSignature,+UserPublicKey,+UserSignature%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;const MCL: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, const MCC: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, const MPL: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for McAuthorisationToken&lt;MCL, MCC, MPL, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt;<div class=\"where\">where\n    NamespacePublicKey: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + NamespaceId + Encodable + Verifier&lt;NamespaceSignature&gt; + <a class=\"trait\" href=\"iroh_willow/proto/meadowcap/trait.IsCommunal.html\" title=\"trait iroh_willow::proto::meadowcap::IsCommunal\">IsCommunal</a>,\n    NamespaceSignature: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + Encodable + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    UserPublicKey: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + SubspaceId + Encodable + Verifier&lt;UserSignature&gt;,\n    UserSignature: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> + Encodable + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Error.html\" title=\"struct core::fmt::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","iroh_willow::proto::meadowcap::McAuthorisationToken"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-McAuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+NamespaceSignature,+UserPublicKey,+UserSignature%3E\" class=\"impl\"><a href=\"#impl-McAuthorisationToken%3CMCL,+MCC,+MPL,+NamespacePublicKey,+NamespaceSignature,+UserPublicKey,+UserSignature%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;const MCL: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, const MCC: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, const MPL: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.usize.html\">usize</a>, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt; McAuthorisationToken&lt;MCL, MCC, MPL, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt;<div class=\"where\">where\n    NamespacePublicKey: NamespaceId + Encodable + Verifier&lt;NamespaceSignature&gt; + <a class=\"trait\" href=\"iroh_willow/proto/meadowcap/trait.IsCommunal.html\" title=\"trait iroh_willow::proto::meadowcap::IsCommunal\">IsCommunal</a>,\n    UserPublicKey: SubspaceId + Encodable + Verifier&lt;UserSignature&gt;,\n    NamespaceSignature: Encodable + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,\n    UserSignature: Encodable + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a>,</div></h3></section></summary><div class=\"impl-items\"><section id=\"method.new\" class=\"method\"><h4 class=\"code-header\">pub fn <a class=\"fn\">new</a>(\n    capability: McCapability&lt;MCL, MCC, MPL, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt;,\n    signature: UserSignature,\n) -&gt; McAuthorisationToken&lt;MCL, MCC, MPL, NamespacePublicKey, NamespaceSignature, UserPublicKey, UserSignature&gt;</h4></section></div></details>",0,"iroh_willow::proto::meadowcap::McAuthorisationToken"]]]]);
    if (window.register_type_impls) {
        window.register_type_impls(type_impls);
    } else {
        window.pending_type_impls = type_impls;
    }
})()
//{"start":55,"fragment_lengths":[12051]}