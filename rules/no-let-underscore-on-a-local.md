+++
tag = "R:no-let-underscore-on-a-local"
title = "Do not silence an unused local with let _ =; use it, report it, or delete it"
error_class = "A value computed to decide something or to report a failure -- a parsed flag, a decoded mode, a caught error -- is bound and then handed to let _ = to silence the unused-binding warning, so the distinction or the failure it carries is dropped with no diagnostic while the code reads as though it had been considered"
home = { kind = "domain", name = "rust" }
created = "2026-09-26"
origin = "mined"
status = { kind = "active" }
incident = "fix-gateway, 2026-09-26. `on_sequence_reset` parsed GapFillFlag(123) into a bool and then wrote `let _ = gap_fill;`, so a FIX SequenceReset in GapFill mode was handled exactly like one in Reset mode: a GapFill numbered ahead of the expected sequence moved the session straight past messages it had never received, and never asked for them (ARCHITECTURE.md D-052). Conformance case Q9 had been ticked on a test that only ever sent a fill at the expected number, the one input where the two modes agree. Auditing the same tree for the shape the same day found a second production site of a different kind: `intake.rs` binds the error from `set_nonblocking` on a freshly accepted socket and discards it with `let _ = e; continue;`, so the peer is dropped with no event at all."

[[detector]]
name = "no-let-underscore-local"
engine = "line-regex"
pattern = '\blet _ = &?(mut )?[a-z_][a-z0-9_]*[[:space:]]*;'
comment = "//"
allow = "allow:let-underscore"
files = ["**/*.rs"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "let _ = on a local switches off the unused-binding warning at the moment it was right. Use the value, report or propagate it, or delete the code that computed it. A discarded call result is a different case: write the call, not the local."

[[detector.fixture]]
path = "src/cli.rs"
input = '    let _ = mode;'
expect = "hit"

[[detector.fixture]]
path = "src/cli.rs"
input = 'if let Err(e) = op() { let _ = e; continue; }'
expect = "hit"

[[detector.fixture]]
path = "src/cli.rs"
input = '    let _ = tx.send(msg);'
expect = "clean"

[[detector.fixture]]
path = "src/cli.rs"
input = '    let _guard = lock.acquire();'
expect = "clean"

[[detector.fixture]]
path = "src/cli.rs"
input = '    let _ = mode; // allow:let-underscore: reserved for the v2 flag, see TODO'
expect = "clean"

[[detector.fixture]]
path = "src/cli.rs"
input = '// let _ = mode; is the shape to avoid'
expect = "clean"
+++

The compiler's unused-binding warning is a reader. It says that a value you went to the trouble of computing reaches nothing. `let _ = x;` exists to switch that reader off, and on a local it almost always switches it off at the moment it was right.

It takes two shapes, and both compile clean under `-D warnings` while reading to a reviewer as a decision someone made:

- **A distinction computed and dropped.** A flag is parsed, a mode is decoded, a config value is read -- and then discarded, so the cases it was meant to separate are silently handled as one.
- **A failure caught and dropped.** `if let Err(e) = op() { let _ = e; continue; }` leaves no event anywhere; the failure happened, and nothing in the system can say so afterwards.

When the warning fires, do one of three things. **Use the value** -- an unused distinction usually means a branch is missing, and the warning is pointing at it. **Report or propagate it** -- an error that is genuinely safe to ignore still gets a named event, or a comment saying why nobody downstream needs to know. **Or delete the code that computed it.** Where a binding is dead by design, such as the last assignment to a loop variable, rewrite the loop so the dead assignment never happens rather than muting the warning over it.

`let _ = f();` on a *call* is a different case: it discards a return value, which R:must-use-on-consequential-returns governs. This rule is about a local you already hold. R:guarantee-needs-a-reader relies on the compiler to catch unread bindings; `let _ =` is the one spelling it cannot see.

Failure-mode check, whenever you are about to write `let _ = name;`: *what was this value computed to decide, and where does that decision now happen?* If the answer is "nowhere", the warning was right.
