+++
tag = "R:private-fields-only"
title = "Struct fields are private; construction goes through a constructor"
error_class = "A public field on a domain type, so a value can be built or mutated into a state the type's own constructor would have rejected, and the invariant the type advertises is unenforceable"
home = { kind = "domain", name = "rust" }
created = "2026-08-24"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-24 from Pattern 6 and the General Coding Rules of ~/.claude/CLAUDE.md, which state it in prose. Surfaced when the emitted Copilot artefact was audited against that pattern list and ten practices were found to have no rule in this library -- so they reached Claude through the always-on boot index and reached no other assistant at all. A practice that lives in one vendor's instruction layer only is the vendor lock this tool exists to remove."
+++

Domain types have private fields. The only way to construct one is a smart constructor that enforces the invariant and returns a `Result`; the only way to read one is an accessor.

A `pub` field is a second constructor that checks nothing. Every guarantee the smart constructor establishes is void the moment a caller can write the field directly, and the type's name goes on claiming it. This is the same defect as a validator that some call sites skip, moved from the function layer down to the field layer, where it is harder to see.

Hand out the narrowest borrow the caller can use: `fn host(&self) -> &str`, not `&String`, and never `&mut` on a field the invariant depends on. Where a caller genuinely must change the value, give it a method that re-establishes the invariant, or one that consumes the value and mints a new one.

R:parse-dont-validate is what the constructor does; this rule is what makes it the only door.
