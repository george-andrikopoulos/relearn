# relearn

**Write the rule once. Compile it to every assistant.**

`relearn` stores the rules an engineering team derives from its own mistakes as **versioned instruction artifacts** — one neutral file per rule, carrying its text, its error class, its triggering incident, and its provenance — and compiles them out to whichever instruction layer a given AI assistant reads: Claude skills, Cursor rules, GitHub Copilot instructions, `AGENTS.md`, project `CLAUDE.md`.

## Why

When an expert corrects an AI assistant, the correction usually dies with the session and the same class of error returns for the next engineer. The fix is to write the correction into the assistant's persistent instruction layer — but every assistant has a different one, so a rule written for one tool is locked to that tool. The correction survives the reboot and still fails to travel.

`relearn` makes the **rule** the artifact and the vendor format a build target. It also refuses to let a rule exist without provenance: a date, the incident that caused it, and the error class it retires. Rules that cannot say where they came from cannot be audited, and a library that cannot be audited becomes sediment.

## Status

Early. Phase A (portability) in progress; see `TODO.md`. Not yet released.

## Design

- One rule, one file, neutral format — TOML front-matter, markdown body.
- Emitters are pure functions of a *validated* library; a rule that fails to parse stops the build rather than being skipped, because a silently dropped rule is a lost correction.
- Type-driven: illegal states — two homes for one rule, a graduated rule with no destination, a tag of the wrong shape — are unrepresentable rather than merely discouraged.
- Emitted files are never sources. They carry a generated-by header and a hash, and the compiler refuses to overwrite anything it did not write.

See `ARCHITECTURE.md` for the pipeline and the decisions log, `CLAUDE.md` for the full recreation standard.

## Background

`relearn` is the reference implementation of the error loop described in *Tuning the Stochastic Machine: A Systems Engineer's Operating Model for Human-AI Engineering* (George Andrikopoulos, 2026) and of the Stochos framework's first two principles: **persist or perish**, and **one home per rule**.

## Licence

Apache-2.0 (planned) — the explicit patent grant matters for anything intended to be safely adoptable inside enterprises.

## Author

George Andrikopoulos — United Kingdom.
