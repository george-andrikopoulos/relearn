+++
tag = "R:no-unwrap-in-production"
title = "No unwrap() in production code"
error_class = "unwrap() in production code, producing a panic with no context that is impossible to diagnose from a single log line"
home = { kind = "domain", name = "rust" }
created = "2026-07-16"
origin = "mined"
status = { kind = "graduated", to = "hook:no-unwrap-in-src + hook:no-expect-empty-msg", date = "2026-07-21" }
incident = "A recurring class strong enough to graduate to a deterministic control: an unwrap() in production yields a context-free panic. The guarantee moved from instruction to the no-unwrap-in-src PreToolUse hook, which blocks the pattern at write time rather than trusting a reviewer to catch it."
published_incident = "A recurring class strong enough to graduate to a deterministic control. An unwrapped result in production yields a panic with no context: no resource named, no invariant stated, nothing for whoever is paged to act on. The guarantee moved out of the instruction layer and into a write-time hook that blocks the pattern at the moment it is written, rather than trusting a reviewer to catch one bare call among hundreds."

[[detector]]
name = "no-unwrap-in-src"
engine = "line-regex"
pattern = '\.unwrap[[:space:]]*\('
comment = "//"
allow = "allow:unwrap"
files = ["src/**/*.rs"]
exclude = ["**/tests/**", "**/benches/**", "**/examples/**", "**/tests.rs", "**/*_test.rs", "**/*_tests.rs", "**/*test_*.rs"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "No unwrap() in production code. Propagate with ?, or use .expect() with a message naming the resource and the invariant, or match both arms. A call that truly cannot fail is marked on its line."

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f().unwrap();'
expect = "hit"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f()?;'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f().unwrap(); // allow:unwrap'
expect = "clean"

[[detector.fixture]]
path = "src/foo_tests.rs"
input = 'let x = f().unwrap();'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = '// never do f().unwrap()'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f().unwrap_or(0);'
expect = "clean"

[[detector.fixture]]
path = "tests/it.rs"
input = 'let x = f().unwrap();'
expect = "clean"

[[detector]]
name = "no-expect-empty-msg"
engine = "line-regex"
pattern = '\.expect[[:space:]]*\([[:space:]]*"([[:space:]]{0,3}|[^"[:space:]]{1,3}[[:space:]]*)"'
comment = "//"
allow = "allow:expect-msg"
files = ["src/**/*.rs"]
exclude = ["**/tests/**", "**/benches/**", "**/examples/**", "**/tests.rs", "**/*_test.rs", "**/*_tests.rs", "**/*test_*.rs"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "An empty or three-character expect() message panics with nothing to diagnose from one log line. Name the resource and the invariant that should have held, or propagate with ?."

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f().expect("");'
expect = "hit"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f().expect("ok");'
expect = "hit"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f().expect("config parsed at startup");'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = 'let x = f().expect(""); // allow:expect-msg'
expect = "clean"

[[detector.fixture]]
path = "src/lib_test.rs"
input = 'let x = f().expect("");'
expect = "clean"
+++

No unwrap() in production code. Use expect() only with a meaningful panic message that names the resource and the invariant, or return the error with `?` and let the caller decide how to surface it. This rule has graduated: the no-unwrap-in-src hook now enforces it deterministically at write time, so the instruction layer no longer has to.
