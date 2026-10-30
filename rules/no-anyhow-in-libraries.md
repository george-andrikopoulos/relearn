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

[[detector]]
name = "no-anyhow-in-lib"
engine = "line-regex"
pattern = '\banyhow\b'
comment = "//"
allow = "allow:anyhow"
files = ["**/*.rs"]
exclude = ["**/main.rs", "**/*build.rs", "**/tests/**", "**/benches/**", "**/examples/**", "**/tests.rs", "**/*_test.rs", "**/*_tests.rs", "**/*test_*.rs"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "anyhow belongs only in a binary's main.rs. Library code returns a thiserror enum with a variant per failure, so a caller can match on what went wrong."

[[detector.fixture]]
path = "src/lib.rs"
input = 'use anyhow::Result;'
expect = "hit"

[[detector.fixture]]
path = "src/lib.rs"
input = 'use thiserror::Error;'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = 'use anyhow::Result; // allow:anyhow'
expect = "clean"

[[detector.fixture]]
path = "README.md"
input = 'anyhow is fine in prose'
expect = "clean"

[[detector.fixture]]
path = "src/main.rs"
input = 'use anyhow::Result;'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = 'use anyhow_ext::Thing;'
expect = "clean"
+++

Library crates return typed error enums (thiserror), so a Result says exactly what can go wrong and callers can match on it. anyhow belongs only at the outermost binary edge. This rule has graduated: the no-anyhow-in-lib hook now enforces it deterministically at write time.
