+++
tag = "R:no-stale-push-over-fresh"
title = "Never push a stale copy over a fresher target"
error_class = "A script that writes a mirror, template, or snapshot onto a live target without establishing which side is authoritative, so an older copy silently replaces newer real content -- and a diff prompt does not prevent it, because a diff shows what differs and never which side is behind"
home = { kind = "global" }
created = "2026-08-16"
status = { kind = "active" }
incident = "claude-workflow, 2026-08-16: `sync-to-system.sh --apply --yes` would have replaced a July global configuration with a May fork, and with it four of the enforcement hooks themselves -- ten files, no prompt, no indication that the source was the older side. The direction was never established anywhere in the script; it simply wrote. Ported into this library 2026-08-25."
+++

Any script that writes a mirror, template, or snapshot onto a live target must refuse when the target is newer than the source, and must say which side is behind. Direction is the property that has to be checked; sameness is not.

Showing a diff is not a guard. A diff reveals *what* differs and never *which side is authoritative*, and a human clicking through a prompt cannot see modification times, provenance, or which copy anyone has been editing. Approving a diff feels like a decision and supplies none of the information the decision needs.

Where a live authority exists -- a global configuration, a production dataset, a running deployment -- the template must lose unconditionally, and the force flag must not reach it. A flag that overrides a directional guard will be used, by the person who is most sure and least informed, on the day the guard was right.

Ask before writing such a push: *if my copy is the stale one, what does this destroy, and would anything tell me?* The characteristic damage is that the answer is nothing: the operation succeeds, reports success, and the loss is discovered later by someone looking for a change they know they made. This is the directional sibling of R:generate-guards-unversioned -- that rule guards against destroying content that exists nowhere else, this one against destroying the newer of two copies that both exist.
