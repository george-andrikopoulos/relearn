+++
tag = "R:justify-every-clone"
title = "Every clone() carries its reason, or the design is wrong"
error_class = "clone() reached for to silence the borrow checker, so an ownership problem is paid for in an allocation and a second copy of state that can drift from the original, instead of being designed out"
home = { kind = "domain", name = "rust" }
created = "2026-08-24"
origin = "codified"
status = { kind = "partial", by = "hook:no-clone-without-comment", uncovered = "whether the comment states a real reason rather than restating the call, and any Rust file outside a `src/` tree", date = "2026-09-19" }
incident = "Codification-dated, not single-incident: ported 2026-08-24 from the General Coding Rules of ~/.claude/CLAUDE.md, which state it in prose. Surfaced when the emitted Copilot artefact was audited against that pattern list and ten practices were found to have no rule in this library -- so they reached Claude through the always-on boot index and reached no other assistant at all. A practice that lives in one vendor's instruction layer only is the vendor lock this tool exists to remove."
published_incident = "Codified from standing practice rather than mined from a failure. It surfaced in an audit of an emitted instruction artefact against the hand-written pattern list it was supposed to replace: ten practices were stated in that one hand-maintained layer and had no rule in the library, so they reached a single assistant and travelled to no other tool — which is the failure the library exists to prevent, arriving in the library's own contents. The practice: a borrow-checker error is a question about ownership, and copying the value pays to avoid answering it. Where a copy survives examination the reason is written beside it, because unannotated copies are indistinguishable from one another — so a reviewer must re-derive the ownership argument for every one, and in practice re-derives it for none."

[[detector]]
name = "no-clone-without-comment"
engine = "line-regex"
pattern = '\.clone[[:space:]]*\('
comment = "//"
allow = "allow:clone"
files = ["src/**/*.rs"]
exclude = ["**/tests/**", "**/benches/**", "**/examples/**", "**/tests.rs", "**/*_test.rs", "**/*_tests.rs", "**/*test_*.rs"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "A .clone() call needs its reason on the same line. Borrow instead, restructure so one owner is obvious, or bump a refcount with Arc::clone(&x). Where a copy survives that, write the reason beside it: // allow:clone: <why ownership needs a second copy>."

[[detector.fixture]]
path = "src/lib.rs"
input = 'let b = a.clone();'
expect = "hit"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let b = a.clone(); // allow:clone: each worker owns its config'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let b = Arc::clone(&a);'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let v: Vec<Tag> = tags.iter().cloned().collect();'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = '// a.clone() would hide the ownership question'
expect = "clean"

[[detector.fixture]]
path = "src/parse_tests.rs"
input = 'let b = a.clone();'
expect = "clean"

[[detector.fixture]]
path = "tests/it.rs"
input = 'let b = a.clone();'
expect = "clean"
+++

A borrow-checker error is a question about ownership. `clone()` does not answer it; it pays to avoid answering it. Try the answers first: restructure so one owner is obvious, take a borrow with a named lifetime, split the borrow across smaller fields, or share with `Arc`/`Rc` where the value is genuinely shared rather than copied.

Where a clone survives that examination, write the reason beside it, in the house form used throughout this codebase:

```rust
let sources: Vec<RuleTag> = rules.iter().map(|r| r.tag().clone()).collect(); // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
```

The comment is what makes the rule reviewable. Unannotated clones are indistinguishable from one another, so a reviewer has to re-derive the ownership argument for every one and in practice re-derives it for none. An annotated clone states a claim that can be checked, and that can be found and removed the day the design around it changes.

The reason must be about ownership or lifetime. "To make it compile" is the error class restated, not a justification. Two copies of a value the code believes is one value is a correctness bug waiting for whoever mutates the wrong one.

Most clones that survive review were created by a signature, not by a call site: R:borrow-in-signatures is where the pressure comes from, and fixing the parameter usually deletes the clone rather than annotating it.
