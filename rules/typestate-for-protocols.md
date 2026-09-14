+++
tag = "R:typestate-for-protocols"
title = "Typestate for protocols: out-of-order calls should not compile"
error_class = "Encoding a protocol's stage as runtime data on one type, so a method invalid in the current stage still exists and must be rejected at runtime"
home = { kind = "domain", name = "rust" }
created = "2026-08-22"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-22 from the standing rust-typedd discipline (~/.claude/skills/rust-typedd/SKILL.md, Core practices, revised 2026-07-22) and Pattern 3 of ~/.claude/CLAUDE.md, which carries the DbusConnection<Disconnected|Connected|Authenticated> worked example. Ported as part of consolidating the Rust discipline into one home."
published_incident = "Codified from a standing type-design discipline rather than mined from a failure — deliberate policy, stated in prose in a hand-maintained layer for months before it became a rule that could travel. The practice: where a sequence has rules — connect before authenticate, initialise before run — encode the stage in the type rather than in a field, so an operation at the wrong moment is a method that does not exist instead of a runtime guard somebody has to remember to write. A check for a state that should not be reachable yet is the shape to look for."
+++

When a sequence has rules -- connect before authenticate, init before run, configure before start -- encode the stage in the type, not in a field. Each step consumes the value in one state and produces it in the next, so a method that is invalid in the current state simply does not exist and calling it is a compile error. This is R:make-illegal-states-unrepresentable applied to time rather than to structure: the illegal thing is not a contradictory pair of fields but an operation at the wrong moment, and the same remedy applies -- make it unrepresentable rather than guarded. A runtime `if !self.authenticated { return Err(...) }` in a method that should not exist yet is the shape to look for.
