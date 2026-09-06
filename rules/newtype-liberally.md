+++
tag = "R:newtype-liberally"
title = "Newtype liberally: distinct concepts get distinct types"
error_class = "Passing a bare primitive across a function boundary, so two values the domain treats as different are interchangeable to the compiler and can be swapped silently"
home = { kind = "domain", name = "rust" }
created = "2026-08-22"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-22 from the standing rust-typedd discipline (~/.claude/skills/rust-typedd/SKILL.md, Core practices, revised 2026-07-22) and Pattern 1 of ~/.claude/CLAUDE.md, both of which state it in prose with the Miles/Kilometers example. Ported because the Rust discipline had two homes -- that skill and this corpus -- with four rules already stated in both, which is the one-home-per-rule violation this tool exists to prevent."
+++

Give every domain concept its own type, even when the underlying representation is identical. `Miles(f64)` and `Kilometers(f64)`, `Host(String)` and `Port(u16)` -- never two bare `f64`s or a `String` and a `u16` whose order only a human remembers. Newtypes are ordinarily zero-cost: they compile to the same machine code as the primitive, so the only thing they add is the compile error you want -- but where that cost is load-bearing, R:verify-the-abstraction-compiled-away requires the claim to be checked rather than repeated. The unit mix-up that cannot be written is cheaper than the one caught in review, and far cheaper than the one that is not. R:parse-dont-validate is where the newtype comes from -- the boundary mints it as a witness; this rule is about carrying it everywhere afterwards instead of unwrapping back to the primitive.
