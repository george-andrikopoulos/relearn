+++
tag = "R:delegate-a-fan-out"
title = "Delegate a fan-out once the work has been enumerated"
error_class = "Walking N independent follow-ups one at a time after a build, gate or plan has already listed them -- and, where delegation is unavailable, absorbing that cost silently instead of putting the choice"
home = { kind = "global" }
created = "2026-08-31"
origin = "mined"
status = { kind = "active" }
incident = "Trigger 2026-08-31, the same session as `[R:no-silent-spend]`: roughly ten compile failures repaired strictly serially *after* the build had already enumerated them, plus a five-file documentation set walked once per release across nine tagged releases, without once mentioning that delegation was off. The rule as it then stood read 'ALWAYS use parallel Task execution' and carried no tag -- unconditional and therefore dead, because a session harness policy forbade subagents outright and that policy wins by default. Because the rule said nothing about being overridden, the behaviour that was correct under the override -- surfacing the cost -- was written nowhere and never happened. Repaired in place as a broken control rather than answered with a second rule beside it. Ported into this library 2026-09-16 from ~/.claude/rules/ecc/common/agents.md, where it reached no instruction layer but the one hand-maintained file and no rule could cite it."
published_incident = "A build listed roughly ten independent compile failures and they were repaired one at a time, in sequence, after the list already existed; across nine releases in the same session a five-document set was walked once per release the same way. The session's harness policy forbade subagents, so the serial path was in fact the only one available — but the instruction covering this said only 'always use parallel execution', with no account of what to do when that is impossible. So the behaviour that was actually correct under the override, saying out loud what the serial path was costing, was written nowhere and never happened. An unconditional rule contradicted by policy is not a strong rule; it is an absent one."
+++

Compiler-driven repair is genuinely serial while you are still deriving what is broken:
each fix changes the next error. It stops being serial the moment a build, a gate or a plan
has **enumerated** N independent follow-ups. From there the edits do not depend on each
other, and walking them one at a time buys nothing. The fan-out point is after the list
exists, not before.

The same shape covers a documentation set updated once per release, a review that reads
several files against one contract, and research needing several independent sources.

**When delegation is disabled, put the choice -- do not merely announce the cost.** This
rule is routinely overridden: a session's harness policy may forbid delegation outright,
and that policy wins by default. What it does not do is make the cost disappear, and a
notice issued after you have already chosen is information rather than a decision. At the
first real fan-out, state the fork in one line -- what would be delegated, roughly how many
serial round trips it replaces, and what turns it on -- then carry on serially unless told
otherwise, without repeating the offer. Pointing at a settings toggle does not count: a
global switch someone must go and change is a regime change, not a choice about *this*
task.

Failure-mode check: **has something already listed this work for me?** If a build, a gate
or a plan produced the list, the serial walk is a choice you are making on somebody else's
budget.

Sibling, deliberately not merged. `[R:no-silent-spend]` is the general form -- every trade
of the user's time against their money against thoroughness. This rule is kept to the
delegation instance so the two carry separate metrics: two rules with distinct surfaces can
be told apart by a review and each carries its own evidence, where one wide rule that fires
for either reason is falsifiable by nothing.
