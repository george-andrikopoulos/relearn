+++
tag = "R:borrow-in-signatures"
title = "Take a borrow in the signature unless the function needs to own"
error_class = "A parameter typed String or Vec<T> where the body only reads it, forcing every caller to allocate, or to surrender ownership it still needs and then clone to get it back"
home = { kind = "domain", name = "rust" }
created = "2026-08-24"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-24 from the General Coding Rules of ~/.claude/CLAUDE.md, which state it in prose. Surfaced when the emitted Copilot artefact was audited against that pattern list and ten practices were found to have no rule in this library -- so they reached Claude through the always-on boot index and reached no other assistant at all. A practice that lives in one vendor's instruction layer only is the vendor lock this tool exists to remove."
+++

Take `&str` over `String`, `&[T]` over `Vec<T>`, `&Path` over `PathBuf`. A signature is a statement about what the function needs, and an owned parameter says "I will keep this" -- so when the body only reads, the signature is untrue and every caller pays for it in an allocation.

Own the parameter exactly where the function stores it. There, `impl Into<String>` is the courteous form: a caller holding a `String` moves it in for nothing, and a caller holding a `&str` allocates once, knowingly. On the way out, return `&str` rather than `&String` -- the extra indirection buys the caller nothing and pins the field's representation into the public API.

The cost is rarely the single allocation. It is that an owned parameter propagates: the caller clones to satisfy it, its caller clones to satisfy that, and a signature chosen without thought becomes a column of clones that each look locally necessary. R:justify-every-clone is where those clones surface; this rule is how they are never created.
