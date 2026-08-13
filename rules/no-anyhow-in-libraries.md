+++
tag = "R:no-anyhow-in-libraries"
title = "No anyhow in library return types"
error_class = "anyhow in a library crate's return types, erasing the typed error contract a Result is supposed to state"
home = { kind = "domain", name = "rust" }
created = "2026-07-16"
status = { kind = "graduated", to = "hook:no-anyhow-in-lib" }
incident = "A recurring class strong enough to graduate to a deterministic control: anyhow in a library's public errors hides exactly what can go wrong, so callers cannot match on it. The guarantee moved from instruction to the no-anyhow-in-lib PreToolUse hook."
+++

Library crates return typed error enums (thiserror), so a Result says exactly what can go wrong and callers can match on it. anyhow belongs only at the outermost binary edge. This rule has graduated: the no-anyhow-in-lib hook now enforces it deterministically at write time.
