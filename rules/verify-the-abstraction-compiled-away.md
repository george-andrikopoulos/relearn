+++
tag = "R:verify-the-abstraction-compiled-away"
title = "Verify a zero-cost claim; never assert it"
error_class = "Claiming an abstraction is zero-cost from its reputation -- newtype, iterator chain, generic wrapper -- without inspecting what the compiler emitted, so a real cost such as a bounds check, a heap allocation, or a missed inline ships as a claimed absence of cost"
home = { kind = "domain", name = "rust" }
created = "2026-08-24"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-24 from Pattern 9 of ~/.claude/CLAUDE.md, which states the verification step in prose. Surfaced when the emitted Copilot artefact was audited against that pattern list and ten practices were found to have no rule in this library. Sharpened in the porting: the library already asserted zero cost inside R:newtype-liberally while carrying no rule that such a claim must be checked, which is exactly the shape of an unverified guarantee travelling as a fact."
published_incident = "Codified from standing practice rather than mined from a failure. It surfaced in an audit of an emitted instruction artefact against the hand-written pattern list it was supposed to replace: ten practices were stated in that one hand-maintained layer and had no rule in the library, so they reached a single assistant and travelled to no other tool — which is the failure the library exists to prevent, arriving in the library's own contents. The practice: a zero-cost claim is a property of a particular abstraction under a particular optimiser on a particular profile. It is usually true, which is exactly why it gets asserted instead of checked, and why the cases where it is false survive review. Where the cost is load-bearing, look at what was emitted; where it is not, do not make the claim."
+++

"Zero-cost" is a property of a particular abstraction, under a particular optimiser, on a particular build profile. In Rust it is usually true, which is precisely why it gets asserted instead of checked, and why the cases where it is false survive review.

Where the cost is load-bearing -- a hot path, a latency budget, an allocation-free claim -- look at what was emitted. `cargo asm` for the function, `cargo bloat` for the binary, a benchmark on the profile that actually ships. A debug build proves nothing about a release binary: a newtype that is free at `opt-level = 3` need not be at `opt-level = 0`, and the two are different programs.

Then run the failure-mode check R:measure-cost-per-task states in the general case -- under what configuration does this cost exactly what it was chosen not to cost? A `#[repr(transparent)]` newtype crossing an FFI boundary, an iterator chain that fails to fuse because the closure captures by reference, a generic that is never monomorphised because it went out through a trait object. Bound that configuration, or drop the claim.

This is R:verify-through-production-path applied to code generation: a claim measured on a build the user never runs is evidence about that build alone. Where the cost is not load-bearing, do not make the claim at all -- an unverified performance assertion in a doc comment is read as a measured one.
