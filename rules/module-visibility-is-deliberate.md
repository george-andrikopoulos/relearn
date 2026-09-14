+++
tag = "R:module-visibility-is-deliberate"
title = "Modules are the encapsulation boundary; pub is a deliberate export"
error_class = "Items marked pub by reflex, so a crate's internal structure becomes its public API and every later change to it is a breaking change nobody chose to make"
home = { kind = "domain", name = "rust" }
created = "2026-08-24"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-24 from the General Coding Rules of ~/.claude/CLAUDE.md, which state it in prose. Surfaced when the emitted Copilot artefact was audited against that pattern list and ten practices were found to have no rule in this library -- so they reached Claude through the always-on boot index and reached no other assistant at all. A practice that lives in one vendor's instruction layer only is the vendor lock this tool exists to remove."
published_incident = "Codified from standing practice rather than mined from a failure. It surfaced in an audit of an emitted instruction artefact against the hand-written pattern list it was supposed to replace: ten practices were stated in that one hand-maintained layer and had no rule in the library, so they reached a single assistant and travelled to no other tool — which is the failure the library exists to prevent, arriving in the library's own contents. The practice: visibility is the only statement a compiler can check about what may still be changed freely. A public item is cheap to add and expensive to remove — once it is public the module cannot be reorganised and the helper written for one call site is a supported interface, none of which was decided."
+++

Use `pub(crate)` and `pub(super)` freely; reserve bare `pub` for what the crate deliberately exports. Visibility is not paperwork. It is the statement of what may still be changed freely, and it is the only such statement a compiler can check.

The default matters because `pub` is cheap to add and expensive to remove. Once an item is public an external caller may depend on it, so the module can no longer be reorganised, the field can no longer be renamed, and the helper written for one call site is now a supported API. None of that was decided; it was defaulted into.

A module holds an invariant in the same way a type does. A helper that may only be called after a check belongs beside that check, private to the module, so "only after" is enforced by nobody elsewhere being able to call it at all. Making it public converts that guarantee into a doc comment.

Re-export the intended surface explicitly at the crate root with `pub use`, and let everything behind it be as private as it can be. The public API is then a list someone wrote, rather than the residue of where the code happened to live.

R:private-fields-only is the same argument one level down, at the field rather than the item. The two are separable -- a private field on a `pub` type leaks structure, a `pub(crate)` type with public fields leaks none -- so satisfying either says nothing about the other.
