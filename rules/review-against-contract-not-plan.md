+++
tag = "R:review-against-contract-not-plan"
title = "Review a change against the behaviour contract, never against the plan that produced it"
error_class = "A review conducted against the plan, specification or template that generated the change, so a defect transcribed faithfully from defective reference material passes every check -- the diff matches its instructions, and the document supplying the instructions is also supplying the standard of correctness"
home = { kind = "global" }
created = "2026-09-19"
origin = "mined"
status = { kind = "active" }
incident = "2026-07-16, the mesh-watchdog build. Every one of the four Important findings and the single Critical originated in plan-template code that implementers had transcribed faithfully, and every one of them was caught by a reviewer holding the project's FEATURES.md rather than the plan. Reviewers armed only with the plan approved the diff, correctly by their own lights: the code matched what the plan said to write. The Critical was a stopped service mapped to a zero-valued age, which an anti-thrash gate then read as \"too young to restart\" and suppressed the restart forever -- the exact recovery the tool existed to perform. Every per-task review passed it; only the whole-branch review against the behaviour contract caught it. The plan was not careless. It was unreviewed input that arrived wearing the authority of a design document, and its authorship cannot grade its own work."
+++

Review a change against the **behaviour contract**, never against the plan or the template that produced it.

Plan code, scaffold code and reference implementations are **unreviewed input**. A faithful transcription of a defective template is still a defect, and the transcription is the part review is worst at seeing: the diff matches its instructions exactly, so every local question a reviewer asks has a satisfying answer.

**Arm the reviewer with the contract, not the intent.** The question is *what does this system now promise, and does this change keep every one of those promises* -- not *does this match what we said we would build*. Those two questions diverge precisely where a plan is wrong, which is the only case where review had anything to catch.

**A plan cannot grade its own work.** If the same document supplies both the instruction and the standard of correctness, review reduces to checking transcription accuracy. Where a plan is the only artefact, that is worth saying out loud in the review rather than letting the approval imply more than it checked.

**The failure survives a per-item review and dies at the whole-change one.** Each task, judged against its own slice of the plan, is correct; the contract violation only becomes visible against the feature set entire. Where a change spans several tasks, one pass must read the whole of it against the whole contract.

Failure-mode check, before approving: *what did I compare this against, and could that thing itself be wrong?* If the answer is the plan, the specification or the ticket, the contract has not been consulted yet.
