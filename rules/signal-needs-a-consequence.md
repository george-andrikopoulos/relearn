+++
tag = "R:signal-needs-a-consequence"
title = "Give a repeated signal a consequence, or stop emitting it"
error_class = "A check that is correct and whose verdict arrives intact, but that gates nothing -- so it prints into a stream a person must choose to read, is read once, and becomes invisible while the condition it reports goes on degrading"
home = { kind = "global" }
created = "2026-09-19"
origin = "mined"
status = { kind = "active" }
incident = "2026-09-19, stochos-lab. The `unheld-recurrence-report` SessionStart hook printed, correctly and at the top of every session, that the installed `relearn` binary could not parse the rule library. It did so for five days. Nobody read it, and in that window a new rule home of 13 rules and 26 promoted rules never reached the always-loaded skills layer on the machine, so every session on it was governed by a corpus five days stale. The hook was not broken: its verdict was accurate, whole, and on screen. It gated nothing, so ignoring it cost nothing. The aggravating detail is one level up, and is the reason the rule is stated as it is -- a gate that WOULD have failed on the same drift already existed, `verify-deploy.sh` section 8, and it was not run either, because running it was also something a person had to remember. Attaching the consequence to a command someone must choose to invoke reproduces the defect it was meant to fix. The counter-example arrived in the same session: `relearn lint` fails CI on an unheld recurrence, and that signal has never once been ignored. Found by proposing this as a recurrence of `[R:guarantee-needs-a-reader]` and rejecting it -- nothing failed to read the state there, a person failed to act on a correct report -- which is how the gap was located at all."
+++

Attach a consequence to a signal, or stop emitting it. A check that is right, whose verdict arrives whole, and that changes nothing is read once out of novelty and never again -- and the condition it reports then degrades quietly behind a line of text that says it is degrading.

**The consequence is what makes a signal a control.** Exit non-zero. Fail the build. Block the write. Refuse the push. Until one of those is attached, what exists is a description of a problem, and a ledger that counts it as coverage is overstating what the system holds.

**Attach it to something that already must happen.** A commit, a push, a build, a test run. A consequence bolted to a command someone has to remember to invoke is the same defect one level up, and it fails the same way: the gate that would have caught the incident behind this rule existed and was not run, because running it was also a choice. Ask which unavoidable event this fires on. If the answer names a person's intention rather than an event, nothing has been attached yet.

**Advisory is a legitimate design, and it is the expensive one.** Sometimes a signal must not block: the judgement is genuinely human, or the input comes from strangers and failing CI on it would hand them your build. Then the consequence is a *named reader on a schedule* -- whose job, how often, and what they do with it. "It is printed at startup" is not a reader; nobody is on the hook for a line of output. Choosing advisory means choosing to spend somebody's attention every time, forever, rather than deciding the escalation rule once. Say so when you choose it, and say who pays.

**Two shapes, one cause.** A signal that never fires is ignored because it is invisible; a signal that always fires is ignored because it is furniture. An always-red gate is a muted gate exactly as a never-red one is. What both lack is a state change that costs somebody something.

The test, at the moment you write the warning: *what breaks if nobody reads this?* If the answer is "nothing breaks, it just stays wrong", it is not a control and must not be counted as one.

Fourth position in an existing family, and the one where every link holds. `[R:guarantee-needs-a-reader]` -- nothing checks the claim. `[R:wired-artifact]` -- something checks it and accepts evidence anything could produce. `[R:verdict-survives-the-channel]` -- the check is right and its verdict is destroyed in transit. `[R:reconcile-wiring-at-start]` -- the control goes dark without announcing it. Here the check is correct, the wiring is live, the verdict is intact and delivered, and the chain still ends, because its last link was a person with nothing at stake.
