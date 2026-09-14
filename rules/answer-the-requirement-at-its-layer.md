+++
tag = "R:answer-the-requirement-at-its-layer"
title = "Answer a requirement at the layer it lives at"
error_class = "A requirement stated at the systems level -- do not perturb a neighbouring isolated core, do not fault a page on the send path, do not take a lock a real-time thread contends for -- discharged by a library-level choice, so the constraint is recorded as satisfied while nothing in the system actually holds it"
home = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
created = "2026-09-14"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-09-14 from the Reasoning & Methodology section of ~/.claude/CLAUDE.md, which states it in prose and carries no tag -- so it could be neither cited by another rule nor compiled into any instruction layer but the one hand-maintained file. Surfaced when the rust domain was reviewed ahead of Chronicle Queue and Aeron work and found to contain eighteen general Rust-design rules and nothing at all about latency: `[R:measure-cost-per-task]` had absorbed the mechanism-selection half of that section during an earlier port, and this half had been left behind. The sibling below was left behind in the same way and is ported in the same change."
published_incident = "Codified from standing practice rather than mined from a failure, and the porting trigger was itself the finding: a domain library was reviewed before starting latency-sensitive work and found to hold eighteen general language-design rules and nothing about latency, while this practice sat untagged in a single hand-maintained instruction file -- uncitable by any other rule and compiled into no instruction layer. The practice: a constraint expressed about the machine is not discharged by a decision about a dependency, and recording it as met at the wrong layer is worse than leaving it open, because an open constraint still gets looked at."
+++

State which layer a requirement lives at before choosing anything, and satisfy it there.

A requirement about the machine -- do not perturb a neighbouring isolated core, do not
fault a page on the send path, do not take a lock a real-time thread contends for -- is
not answered by picking a faster crate, a better-reputed transport, or an asynchronous
API. Those are library-level decisions. They may be correct and they may even help, but
they do not *hold* a systems-level constraint, and treating them as though they do
converts an open question into a closed one with nothing behind it.

The failure is not the wrong choice; it is the wrong ledger entry. A constraint marked
satisfied stops being examined. An open constraint is still visible to the next person,
which makes leaving it open strictly better than closing it at a layer that cannot keep
it.

So name the layer explicitly, then name what holds the requirement there: a CPU affinity
mask, an isolated core list, a memory policy, a pre-faulted and locked mapping, a thread
priority, a cgroup. If the answer to "what holds this?" is the name of a library, the
requirement is unheld.

Two rules meet here and neither subsumes this one. `[R:measure-cost-per-task]` governs
*how to choose* a mechanism -- by what it actually costs and touches, never by
reputation -- and its failure-mode check asks under what configuration the mechanism
causes the exact harm it was chosen to prevent. That check is necessary and it is not
sufficient: a mechanism can pass it and still be the wrong layer to have asked at.
`[R:guarantee-needs-a-reader]` is the general form of the ledger failure -- a claim with
nothing reading the state it asserts -- and this is the shape that claim takes when the
state in question is a property of the machine rather than of the code.

Then run the pass the sibling rule requires: `[R:attack-the-design-in-a-second-pass]`
exists because the person who chose the mechanism is the last person able to see that
they answered at the wrong layer.
