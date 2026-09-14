+++
tag = "R:five-files-no-more"
title = "A project carries five standing documents, and resists a sixth"
error_class = "Standing project documents accumulating one reasonable addition at a time, so attention is spread across a set nobody re-reads, and the files that are load-bearing decay behind the ones that are merely present"
home = { kind = "global" }
created = "2026-08-25"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-25 from ~/.claude/skills/project-discipline/SKILL.md, the artefact that defines the practice, and specified in *Aiming the Stochastic Machine* (Zenodo, doi:10.5281/zenodo.22083202) which names this repository as its worked case. Surfaced when the emitted Copilot pack was read and found to carry the type discipline and the verification discipline but not one rule about the repository discipline the paper is *about* -- so it reached Claude through a skill and reached Cursor, Copilot and AGENTS.md not at all. Same vendor-lock failure as the ten Rust patterns ported 2026-08-24, in the part of the corpus that had the strongest claim to be there."
published_incident = "Codified from a standing repository discipline rather than mined from a failure. It surfaced in an audit of an emitted instruction pack, which carried no rule about that discipline at all while the type-design half was fully present — so one of the two disciplines the author works by reached every assistant and the other reached none. The practice: a project carries a small fixed set of standing documents and resists a sixth. Each new document is individually reasonable and collectively the reason none of them is read; the constraint is what keeps the set small enough that every one is maintained."
+++

A project keeps exactly five standing documents, each with one job:

- `CLAUDE.md` — the recreation standard: purpose, design decisions with their *why*, invariants, build and test commands.
- `ARCHITECTURE.md` — modules, data flow, perimeter, and an append-only decisions log.
- `FEATURES.md` — the regression ledger, every entry naming the artefact that enforces it.
- `TODO.md` — open work, including every unenforced gap the ledger declares.
- `README.md` — the public face, and the only one of the five written for outsiders. Keep the other four written for the maintainer and the machine.

Resist the sixth. Every resident document taxes attention on the others, and the tax is paid by the documents that matter most, because they are the longest. New standing knowledge goes *into* one of the five, or into a domain rule if it generalises past this project -- never into a new file whose creation felt tidy at the time.

The two failure classes the set exists to kill are staleness and unenforced guarantees. A file claiming completeness that has drifted is a confident lie, and confident stale context is read and acted on; a documented feature with no executable check is shipped and inert. Both are worse than the absence of the document, because both stop people looking.

Do not duplicate domain rules into the project layer. Reference them. A rule stated in two places forks and rots, and the copy is always the one someone reads.
