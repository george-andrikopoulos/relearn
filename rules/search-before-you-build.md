+++
tag = "R:search-before-you-build"
title = "Search your own work before specifying a component"
error_class = "Writing the requirement for a component whose need has only been asserted, without first asking whether the thing already exists -- and searching third-party sources when the most likely author of it is you"
home = { kind = "global" }
created = "2026-08-16"
origin = "mined"
status = { kind = "active" }
incident = "Incident 2026-08-16: a prompt specified building a bash emitter with a list of required guards -- generated header, ownership guard, staleness refusal, idempotence, content hash. `relearn`, a repository mounted in the same session and whose CLAUDE.md had already been read, implemented every one of them, enforced and property-tested. The prompt's own stated purpose was preventing duplication; the duplication was in the specification. Fourth assertion-without-verification that month -- the earlier three (arXiv endorsement, Fable economics, `#[must_use]` coverage) all asked 'does this work?'; this was the first 'does this exist?', which is why the existing rule did not fire. Drift rather than a new class: the Research & Reuse step should have caught it and was too narrow (third-party only), untagged, and buried in inherited boilerplate. Ported into this library 2026-09-16 from ~/.claude/CLAUDE.md, where it reached no instruction layer but the one hand-maintained file and no rule could cite it."
published_incident = "A prompt specified a component and listed the guarantees it had to provide — a generated header, an ownership guard, a staleness refusal, idempotence, a content hash. Another repository, mounted in the same session and whose charter had already been read that session, implemented every one of them, enforced and property-tested. The prompt's own stated purpose was preventing duplication, and the duplication was in the specification. The search step that should have caught it looked only at third-party sources: package registries, vendor documentation, public code search — the wrong end of the search space, because for a component built to your own discipline the most likely author of it is you."
+++

Before writing the requirement for any component, ask whether it already exists -- and
search **your own work first**.

This is a different question from "does this work?", and it fails at a different moment:
earlier, before the mechanism is evaluated at all, while its need is still merely
asserted. A design review that starts once the requirement is written has already
accepted the premise that something must be built.

The usual research step points outward -- public code search, vendor documentation,
package registries -- and that is the wrong end of the search space. For a tool built to
your own discipline, *you* are the most likely author of the thing you are about to
specify. Grep your own repositories first, and read the charter of anything adjacent,
including whatever is already open in this session.

Failure-mode check: **if this already existed, where would it be -- and have I looked
there?** If you cannot name the place you looked, you have not searched; you have
assumed.

Note what this is not. It is not an argument against building, and it is not satisfied by
a vague sense that something similar exists somewhere. It asks for a location and a
result: the path you grepped, the repository you read, and what was or was not there.
