+++
tag = "R:decisions-log-records-rejected-alternatives"
title = "The decisions log records the why and what was rejected, append-only"
error_class = "Architecture recorded as structure without provenance, so a decision cannot be distinguished from an accident and the alternatives already considered and rejected are re-proposed, re-argued, and sometimes adopted"
home = { kind = "global" }
created = "2026-08-25"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-25 from ~/.claude/skills/project-discipline/SKILL.md, the artefact that defines the practice. Surfaced in the 2026-08-25 audit of the emitted Copilot pack, which carried no rule about the repository discipline at all. The log format itself is load-bearing in this repository: the 2026-08-22 claude-rules collision entry is what lets a reader see that three specific alternatives -- a different constant, making Unversioned non-fatal, dropping the target from the default set -- were priced and refused, rather than never considered."
published_incident = "Codified from a standing repository discipline rather than mined from a failure. It surfaced in an audit of an emitted instruction pack, which carried no rule about that discipline at all while the type-design half was fully present — so one of the two disciplines the author works by reached every assistant and the other reached none. The practice: a decision is recorded with what was rejected and why, append-only. Without the rejected alternatives the log reads as a list of things that happened rather than choices that were made, and the next person re-derives — or re-makes — a decision somebody already paid for."
+++

Keep an append-only decisions log in the architecture document: date, decision, why, alternatives rejected. Append; never rewrite an entry to match what was later believed, because the value of the log is that it records what was known at the time.

The rejected alternatives are the half that gets dropped and the half that pays. A decision with its reason tells the next reader why this path was taken; the rejected list tells them why *their* idea was not, which is the question they actually have. Without it, the obvious alternative is re-proposed on a cycle set by staff turnover, and each round costs the same argument with less context than the last.

Anonymous structure accumulates as sediment. Given a module boundary with no recorded reason, a reader must either treat it as sacred or treat it as arbitrary, and neither is true. Write the entry when the decision is made, while the alternatives are still in mind -- reconstructed later, the rejected list is always the flattering one.

Record the decision that surprised you most, not the one that was easiest to write down. This is the same discipline R:claude-md-recreates-the-project applies to the charter: what a future reader cannot reconstruct is exactly what must be written.
