+++
tag = "R:no-anyhow-in-libraries"
title = "No anyhow in library return types"
error_class = "anyhow in a library crate's return types, erasing the typed error contract a Result is supposed to state"
home = { kind = "domain", name = "rust" }
created = "2026-07-16"
origin = "mined"
status = { kind = "graduated", to = "hook:no-anyhow-in-lib", date = "2026-07-21" }
incident = "A recurring class strong enough to graduate to a deterministic control: anyhow in a library's public errors hides exactly what can go wrong, so callers cannot match on it. The guarantee moved from instruction to the no-anyhow-in-lib PreToolUse hook."
published_incident = "A recurring class strong enough to graduate to a deterministic control. A catch-all error type in a library's public signatures hides exactly what can go wrong, so a caller cannot match on the cases and ends up parsing prose that changes the next time somebody edits a message. The guarantee moved out of the instruction layer and into a write-time hook that blocks the pattern in library code, because an instruction is only as good as the attention of whoever is reading it that day."
+++

Library crates return typed error enums (thiserror), so a Result says exactly what can go wrong and callers can match on it. anyhow belongs only at the outermost binary edge. This rule has graduated: the no-anyhow-in-lib hook now enforces it deterministically at write time.
