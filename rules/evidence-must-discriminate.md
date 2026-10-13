+++
tag = "R:evidence-must-discriminate"
title = "Name the rival hypothesis before calling one confirmed"
error_class = "An observation that both a hypothesis and its rival predict is taken as confirmation, and the unearned conclusion is written into decisions and documents before the discriminating measurement is taken"
home = { kind = "global" }
created = "2026-10-13"
origin = "mined"
status = { kind = "active" }
incident = "Home Assistant heating, 2026-10-11. A 30 % OpenTherm modulation cap (ID 14) was set on an Ideal combi. A kitchen-tap draw then ran the burner at 87–100 %, and Claude concluded \"the boiler applies ID 14 to CH only; hot water keeps full output\", marked the cap KEPT and wrote that claim into four project files and memory. The same observation was equally predicted by the rival hypothesis — the boiler ignores ID 14 altogether — and the CH half was never measured. The first night under the cap showed CH burns still at ~50 %, unchanged; the controller's capability map later showed ID 14 as unsupported. Four documents had to be corrected and the decision log carried a dated retraction."
+++

Before calling a hypothesis confirmed, name its rival and check the evidence tells them apart.

An observation supports a hypothesis only to the extent that it would have come out
differently if the hypothesis were false. Evidence that both explanations predict is not
evidence for either, however neatly it fits the one you are holding. The trap is that a
fitting observation feels like confirmation, and the feeling arrives before anyone asks
what else would have produced it.

So when a result seems to settle a question, write down the most plausible rival
explanation — "it ignores the setting", "load also dropped", "the test passed before the
fix too" — and ask what each predicts for this observation. If they predict the same
thing, the question is still open: say so, and find the measurement on which they
disagree before anything is marked kept, fixed or confirmed. That measurement is usually
the one you have not yet taken because it needs waiting for — the next night, the next
load, the case that was not exercised.

This matters most when the conclusion is about to be written down. A claim recorded in
a decision log, a contract or several documents acquires authority it never earned, and
`[R:repair-the-lying-artefact]` then has to undo it everywhere.

Failure-mode check: **what does the rival hypothesis predict for exactly this
observation?** If the answer is "the same", you have not confirmed anything yet.
