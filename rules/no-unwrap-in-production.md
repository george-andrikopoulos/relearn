+++
tag = "R:no-unwrap-in-production"
title = "No unwrap() in production code"
error_class = "unwrap() in production code, producing a panic with no context that is impossible to diagnose from a single log line"
home = { kind = "domain", name = "rust" }
created = "2026-07-16"
origin = "mined"
status = { kind = "graduated", to = "hook:no-unwrap-in-src", date = "2026-07-21" }
incident = "A recurring class strong enough to graduate to a deterministic control: an unwrap() in production yields a context-free panic. The guarantee moved from instruction to the no-unwrap-in-src PreToolUse hook, which blocks the pattern at write time rather than trusting a reviewer to catch it."
+++

No unwrap() in production code. Use expect() only with a meaningful panic message that names the resource and the invariant, or return the error with `?` and let the caller decide how to surface it. This rule has graduated: the no-unwrap-in-src hook now enforces it deterministically at write time, so the instruction layer no longer has to.
