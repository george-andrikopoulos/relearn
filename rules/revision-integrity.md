+++
tag = "R:revision-integrity"
title = "Update the doc in the same change as the thing it describes"
error_class = "A checked-in list or example (hygiene set, format sample, doc snippet) drifting from the reality it describes, so a reader trusts it and acts on stale guidance"
home = { kind = "global" }
created = "2026-07-16"
status = { kind = "active" }
incident = "project-discipline (2026-07-16): the default-hygiene list still named a stale .zed/tasks.json entry after the workflow had moved on, and Claude scaffolded mesh-watchdog from that stale list. Fix: replaced it with the actually-observed hygiene set (README/FEATURES/ARCHITECTURE) and made revision-integrity a rule."
+++

When you change a process, format, or artifact, update every checked-in description of it -- list, example, doc snippet -- in the same change. A doc that lags the code it describes is worse than no doc: a reader trusts it and acts on stale guidance. The behavior contract and its examples are only load-bearing if they are true.
