+++
tag = "R:doc-currency"
title = "Update the doc in the same change as the thing it describes"
error_class = "A checked-in list or example (hygiene set, format sample, doc snippet) drifting from the reality it describes, so a reader trusts it and acts on stale guidance"
home = { kind = "global" }
created = "2026-07-16"
status = { kind = "active" }
incident = "project-discipline (2026-07-16): the default-hygiene list still named a stale .zed/tasks.json entry after the workflow had moved on, and Claude scaffolded mesh-watchdog from that stale list. Fix: replaced it with the actually-observed hygiene set (README/FEATURES/ARCHITECTURE). Retagged on 2026-08-16: the tag it previously carried had been independently assigned on 2026-08-11 to a different rule (verify internal references after restructuring), and the tag-uniqueness invariant made the collision a hard build failure."
+++

When you change a process, format, or artifact, update every checked-in description of it -- list, example, doc snippet -- in the same change. A doc that lags the code it describes is worse than no doc: a reader trusts it and acts on stale guidance. The behavior contract and its examples are only load-bearing if they are true.

Not to be confused with the sibling rule on a document's *internal* consistency
after an edit -- antecedents, cross-references, counts -- which currently lives
in the `~/.claude` skill layer under the tag this rule used to carry. This rule
is about a document's *external* currency: it still describes a reality that has
since moved. A file can pass one and fail the other.

The sibling is still named in prose rather than cited by tag, because it is not
in this library yet and a real citation would genuinely dangle. The *provenance*
above may now name it freely: since 2026-08-16 the linter reads citations from
rule bodies only, so recording why a tag changed no longer trips it.
