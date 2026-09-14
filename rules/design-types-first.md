+++
tag = "R:design-types-first"
title = "Design by writing the types first, before any logic"
error_class = "Starting a design with function bodies or with a failing test, so the type-level specification is back-filled to fit code that already exists rather than constraining it"
home = { kind = "domain", name = "rust" }
created = "2026-08-22"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-22 from the standing rust-typedd discipline (~/.claude/skills/rust-typedd/SKILL.md, 'Standing override' and 'Design by writing types first', revised 2026-07-22), which is deliberate policy rather than a reaction to one failure. Named in this repository's own CLAUDE.md as the design discipline it is built under."
published_incident = "Codified from a standing type-design discipline rather than mined from a failure — deliberate policy, stated in prose in a hand-maintained layer for months before it became a rule that could travel. The practice: sketch the types until the design falls out, then write the logic. A test samples points of the behaviour space; a type constrains the whole space and the compiler proves it everywhere, at compile time, for as long as the code exists. In an assistant-written codebase the type system is the only deterministic whole-space checker available, which is why this ordering supersedes a test-first default rather than sitting beside it."
+++

Sketch the types until the design falls out, then write the logic. The type-level sketch is the executable specification a test-first red phase is reaching for, and it is the stronger one: a test samples points of the behaviour space, a type constrains the whole space and the compiler proves it everywhere, at compile time, for as long as the code exists. When the types are right much of the implementation writes itself, and many wrong implementations stop compiling. In Rust this ordering supersedes any test-first default -- tests are not removed, they are demoted to the layer where they are the right tool: property tests for behavioural laws the types cannot encode, unit tests as regression pins for past bugs. Apply each guarantee at the strongest layer that can hold it, and when reviewing, ask first not "does it pass" but "which of these guarantees could move up a layer?"
