+++
tag = "R:reconcile-wiring-at-start"
title = "Reconcile declared against active wiring on a schedule the guard cannot break"
error_class = "A control that is correctly declared but no longer active -- commented out, moved, un-executable, overwritten -- going dark without announcing it, so the system runs unguarded for as long as the interval between whatever happens to notice"
home = { kind = "global" }
created = "2026-08-16"
status = { kind = "active" }
incident = "2026-08-16: all nine PreToolUse hooks went unwired when settings.json was rewritten by an unidentified writer. Nothing announced it. It surfaced about eighty minutes later, and only because an unrelated audit happened to run the deploy verifier -- edits that should have been blocked went through in the interval. The file was tracked, which is why the entry condition had to be declared-not-equal-to-active rather than 'the wiring is unversioned': scoped to unversioned files, this rule would have excluded the very case that produced it. Ported into this library 2026-08-25."
+++

The entry condition is **declared is not active**. It is not "the wiring is unversioned". A tracked control goes dark just as quietly as an untracked one: a commented-out entry, a moved path, a lost executable bit. Version control yields a diff, but a diff does not tell you a guard is inert *now*, and nobody diffs a file they have no reason to suspect. Being unversioned is an aggravating factor -- no diff, no revert, no attribution -- never the qualifier.

So check the declared-versus-active gap on a schedule that does not depend on the guard being alive, and report **by name** which control is inert. A count is not actionable and reads as noise; a name is a work item.

**Prove the alarm in the dark state.** An alarm never observed firing is not known to fire. Exercise it against a deliberately broken configuration -- delete the hook block in a temporary home directory and confirm the check names each drifted control. Without that drill the reconciliation becomes an unverified control one level up and the regress simply moves; the drill is what terminates it, which is R:wired-artifact applied to the thing doing the watching.

Ask: *if this control switched itself off, what would tell me, and when?* If the honest answer is "the next audit", the control is off for as long as audits are apart, and that interval is the real guarantee -- not the control.
