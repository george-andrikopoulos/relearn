+++
tag = "R:claude-md-recreates-the-project"
title = "The project charter is written to the recreation standard"
error_class = "A project charter that describes what the code is rather than what would be needed to rebuild it, so the decisions and their reasons live only in the head of whoever made them and are re-litigated or silently reversed by the next reader"
home = { kind = "global" }
created = "2026-08-25"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-25 from ~/.claude/skills/project-discipline/SKILL.md, the artefact that defines the practice. Surfaced in the same 2026-08-25 audit that found the repository discipline absent from the emitted rule library while the type discipline was fully present. This repository's own CLAUDE.md opens by stating the standard -- *from this file alone, this project could be rebuilt* -- and check 4 of its definition of done exists to keep that sentence true, but no rule in the library said so, so the standard travelled to no assistant that does not load the project-discipline skill."
+++

Write the project charter so that from it alone the project could be rebuilt. That is the bar, and it is testable: hand the file to someone with the toolchain and nothing else, and ask what they could not reconstruct.

It holds the purpose, the core design decisions **with their reasons**, the invariants that must never break, the commands to build, run and test, and pointers to the other standing documents. The reasons are the part that decays first and matters most: a decision recorded without its why is indistinguishable from an accident, so the next reader either reverses it or preserves it superstitiously, and both are expensive.

Check it explicitly on every change that touches design, as a question rather than a glance: *could this file still recreate the project?* Skipping that check is how the standard becomes a lie -- not in one edit, but in twenty, each individually defensible.

The charter is the project layer and nothing else. Domain rules are referenced from it, never copied into it (R:five-files-no-more), and a claim it makes about an enforcing control is subject to R:guarantee-needs-a-reader like any other.
