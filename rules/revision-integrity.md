+++
tag = "R:revision-integrity"
title = "After restructuring, verify references as a distinct pass"
error_class = "A restructuring edit silently invalidating references that were correct in the previous version -- antecedents, cross-references, counts, enumerations, promises -- with no error raised, and the author least able to see it because they autocomplete the missing text from memory of the draft they deleted"
home = { kind = "global" }
created = "2026-08-11"
origin = "mined"
status = { kind = "active" }
incident = "*Tuning the Stochastic Machine* v0.3, 2026-08-11: the clause 'an instruction in context neither applies nor errors' had lost its setup -- 'a sysctl either applies or errors' -- in an earlier restructure, so the contrast pointed at nothing. Detected by translating the paragraph into Greek, which could not be rendered without adding the missing words. A fresh-reader sweep then found fifteen more: a cross-reference to a section that belonged to the next one, 'seven principles, each forced by an axiom' where only four were, 'three incidents' containing four, and 'the driver' with no driver introduced. Ported into this library 2026-08-25; this repository's charter had listed it as scheduled for migration since Phase A and cited it in prose, so the corpus was citing a rule it did not define."
published_incident = "A paper's clause survived a restructure that removed its setup, so a contrast pointed at nothing and read as a non-sequitur. It was caught only by translating the paragraph into another language, which could not be rendered without supplying the missing words. A fresh-reader sweep then found fifteen more of the same kind: a cross-reference to a section that had moved, a count of items that no longer matched the list beneath it, an antecedent whose noun had been renamed. Each was individually invisible to the author, because a writer re-reads what they meant rather than what is there, and every one was produced by an edit that was correct in itself."

[[detector]]
name = "no-count-in-heading"
engine = "line-regex"
pattern = '^#{1,6} (([Tt]he|[Aa]ll|[Oo]ur) )?([Tt]wo|[Tt]hree|[Ff]our|[Ff]ive|[Ss]ix|[Ss]even|[Ee]ight|[Nn]ine|[Tt]en|[Ee]leven|[Tt]welve|[0-9]+) [a-z-]+s\b'
allow = "allow:counted-heading"
files = ["**/*.md"]
events = ["claude-code-pre-tool-use"]
action = "warn"
message = "This heading states a count, and the count goes stale on the next addition with nothing to say so. Write the heading without the number, or mark the line with the override token if the number is the subject rather than a tally of what follows."

[[detector.fixture]]
path = "docs/design.md"
input = '## The three guarantees'
expect = "hit"

[[detector.fixture]]
path = "README.md"
input = '### 12 commands'
expect = "hit"

[[detector.fixture]]
path = "docs/design.md"
input = '## The guarantees'
expect = "clean"

[[detector.fixture]]
path = "docs/design.md"
input = 'There are three guarantees, listed below.'
expect = "clean"

[[detector.fixture]]
path = "docs/design.md"
input = '## Rule parsing (v0.1)'
expect = "clean"

[[detector.fixture]]
path = "docs/design.md"
input = '## A gap between two writes is not a settled state'
expect = "clean"

[[detector.fixture]]
path = "docs/design.md"
input = '## Two installs, one rule <!-- allow:counted-heading -->'
expect = "clean"

[[detector.fixture]]
path = "src/lib.rs"
input = '## The three guarantees'
expect = "clean"
+++

Editing a structured artefact silently breaks references that the previous version made true. The edit raises no error, and rereading does not catch it: the author restores the deleted context from memory and reads a coherent passage that is not on the page.

So run referential integrity as a **distinct pass**, after the restructuring and not during it. Every pronoun and comparative -- *this*, *neither*, *the former*, *the more important* -- must resolve within the current text. Every cross-reference must point where it claims. Every announced count must match what follows. Every term must be defined before it is used.

Two methods defeat author blindness where rereading cannot. Give the passage to a fresh reader instructed to report **comprehension failures only**, not content or style -- they have no deleted draft to autocomplete from. Or translate it into another language: if it cannot be rendered without adding words, the words are missing in the original.

Prefer structure that cannot carry the defect. A heading that states a count goes stale on the next addition, so write the heading without the count rather than remembering to update it -- R:prefer-by-construction applied to prose.

This is the prose sibling of R:wired-artifact: a locally correct change with a silent non-local effect. Ask, every time: *what did this edit quietly leave pointing at nothing?*
