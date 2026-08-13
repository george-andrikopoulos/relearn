# relearn

**Write the rule once. Compile it to every assistant.**

`relearn` stores the rules an engineering team derives from its own mistakes as **versioned instruction artifacts** — one neutral file per rule, carrying its text, its error class, its triggering incident, and its provenance — and compiles them out to whichever instruction layer a given AI assistant reads: Claude skills, Cursor rules, GitHub Copilot instructions, `AGENTS.md`, project `CLAUDE.md`.

## Why

When an expert corrects an AI assistant, the correction usually dies with the session and the same class of error returns for the next engineer. The fix is to write the correction into the assistant's persistent instruction layer — but every assistant has a different one, so a rule written for one tool is locked to that tool. The correction survives the reboot and still fails to travel.

`relearn` makes the **rule** the artifact and the vendor format a build target. It also refuses to let a rule exist without provenance: a date, the incident that caused it, and the error class it retires. Rules that cannot say where they came from cannot be audited, and a library that cannot be audited becomes sediment.

## Status

Early. **Phase A (portability) is complete**; **Phase B (the linter + `relearn verify`) is functionally complete** — its one remaining item (cold-surface) is blocked on Phase-C runtime data. **Phase D (public release) is in progress**: the licence and a worked example are in place; going fully public is gated on the companion paper's arXiv identifier. Not yet released. See `TODO.md`.

For an end-to-end walk-through — one real incident becoming one rule and compiling out to all five instruction layers, with actual `relearn build` output — see [`docs/worked-example.md`](docs/worked-example.md).

The pipeline `rules/*.md → parse → validate → emit → write` works today for all five targets — Claude skills, Cursor rules, GitHub Copilot instructions, `AGENTS.md`, and a project `CLAUDE.md` — and every v0.1 guarantee in `FEATURES.md` names a real enforcing artifact (types, property tests, and a compile-fail pin). A real rule set lives in [`rules/`](rules/) (fourteen rules ported from George's engineering discipline, spanning global, domain-rust, and two project homes, with active and graduated statuses); `relearn build` compiles it end to end, `relearn lint` reports advisory findings without ever touching a rule, and `relearn verify` checks the emitted tree has not drifted from it.

## Usage

```sh
# Validate the rule library; write nothing. Non-zero exit on any failure.
relearn check --rules ./rules

# List rules as `tag  [home-slug]  title`, optionally filtered by home.
relearn list --rules ./rules
relearn list --rules ./rules --home domain-rust

# Compile the library to an output root. Defaults to every target
# (claude, cursor, copilot, agents, claude-md); pass --targets to narrow.
relearn build --rules ./rules --out .
relearn build --rules ./rules --out . --targets cursor,copilot

# Report advisory findings (overlapping scope, home-slug collisions,
# dangling references). Writes nothing; non-zero exit if any are found.
relearn lint --rules ./rules

# Verify the generated files under --out match what build would write now.
# Reads only; non-zero exit if any file is missing, hand-edited, or stale.
# Use it in CI to keep the committed generated tree in sync with the rules.
relearn verify --rules ./rules --out .
```

A rule directory is `*.md` files with TOML front-matter and a markdown body:

```
+++
tag         = "R:parse-wide-then-range-check"
title       = "Parse wide, then range-check"
error_class = "Range-check collapses out-of-range into not-a-number"
home        = { kind = "domain", name = "rust" }
created     = "2026-07-23"
status      = { kind = "active" }
incident    = "Grouping task 01: 5/5 samples parsed into u16, so 70000 read as NotANumber."
+++

Parse into a type wide enough to represent the out-of-range value, then range-check.
```

Every emitted file carries a generated-by header and a content hash; `build` refuses to overwrite any file it did not write (and aborts the whole run rather than leave a half-generated tree), so a target directory can safely hold both generated and hand-authored files. `relearn verify` is the read-only complement: it recomputes each generated file's body hash and re-emits from the current rules, reporting any file that was hand-edited or has drifted from its source — a drop-in CI check that the committed instruction files are in sync.

Build from source with `cargo build --release`; the binary is `relearn`.

## Design

- One rule, one file, neutral format — TOML front-matter, markdown body.
- Emitters are pure functions of a *validated* library; a rule that fails to parse stops the build rather than being skipped, because a silently dropped rule is a lost correction.
- Type-driven: illegal states — two homes for one rule, a graduated rule with no destination, a tag of the wrong shape — are unrepresentable rather than merely discouraged.
- Emitted files are never sources. They carry a generated-by header and a hash, and the compiler refuses to overwrite anything it did not write.
- A rule's lifecycle status decides whether it is emitted: **active** and **graduated** rules are written (a graduated rule is annotated with the stronger control that also enforces it), while a **retired** (atticked) rule is suppressed, so withdrawn guidance never leaks into a live instruction file. The retired rule stays in the library — the linter still needs it to flag references to it.

See `ARCHITECTURE.md` for the pipeline and the decisions log, `CLAUDE.md` for the full recreation standard.

## Background

`relearn` is the reference implementation of the error loop described in *Tuning the Stochastic Machine: A Systems Engineer's Operating Model for Human-AI Engineering* (George Andrikopoulos, 2026) and of the Stochos framework's first two principles: **persist or perish**, and **one home per rule**.

## Licence

Apache-2.0 — see [`LICENSE`](LICENSE). The explicit patent grant matters for anything intended to be safely adoptable inside enterprises.

## Author

George Andrikopoulos — United Kingdom.
