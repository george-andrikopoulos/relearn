# relearn

**Write the rule once. Compile it to every assistant.**

`relearn` stores the rules an engineering team derives from its own mistakes as **versioned instruction artifacts** — one neutral file per rule, carrying its text, its error class, its triggering incident, and its provenance — and compiles them out to whichever instruction layer a given AI assistant reads: Claude skills, Cursor rules, GitHub Copilot instructions, `AGENTS.md`, a Claude project layer.

## Why

When an expert corrects an AI assistant, the correction usually dies with the session and the same class of error returns for the next engineer. The fix is to write the correction into the assistant's persistent instruction layer — but every assistant has a different one, so a rule written for one tool is locked to that tool. The correction survives the reboot and still fails to travel.

`relearn` makes the **rule** the artifact and the vendor format a build target. It also refuses to let a rule exist without provenance: a date, the incident that caused it, and the error class it retires. Rules that cannot say where they came from cannot be audited, and a library that cannot be audited becomes sediment.

Behind most corrections is a decision the assistant made for itself. It extracted the helper because it judged the duplication to matter; it renamed the field because it judged the old name misleading. Those are **invented requirements** — requirements the model gave itself, on no evidence in the repository — and their visible residue is **skedasis** (from σκεδάννυμι, to scatter): the fraction of a change nobody requested. A ratio, not a size; two hundred lines is not skedasis if the task needed two hundred. Writing the correction down is how an invented requirement gets retired once instead of re-invented every session.

## Status

Early. **Phase A (portability) is complete**; **Phase B (the linter + `relearn verify`) is functionally complete** — its one remaining item (cold-surface) is blocked on Phase-C runtime data. **Phase D (public release): released** under Apache-2.0, with a worked example in place. Companion papers: *Tuning the Stochastic Machine* ([arXiv:2608.19125](https://arxiv.org/abs/2608.19125)) — the operating discipline this implements — *Grouping the Stochastic Machine* ([arXiv:2608.19140](https://arxiv.org/abs/2608.19140)), which references this repository, and *Aiming the Stochastic Machine: A Repository Discipline for First-Time-Right, and What Survived Measuring It* ([doi:10.5281/zenodo.22083202](https://doi.org/10.5281/zenodo.22083202)), which specifies the four-file repository discipline this repo is built on and uses this repository as its worked case. Paper 3 is a Zenodo preprint, not an arXiv one: arXiv moderation declined it on 2026-08-24 and will consider an appeal only after publication in a conventional journal with a resolving DOI to the published version. Its study materials — pre-registration, blinded pack, both codings, `kappa.py`, results — are deposited separately at [doi:10.5281/zenodo.22082967](https://doi.org/10.5281/zenodo.22082967), so the evidence can be checked without reading the paper first. See `TODO.md`.

### What was measured, including what failed

Paper 3 pre-registered its metrics and its falsification conditions before reading any commit history, and **its headline hypothesis was refuted.** Across nine of the author's repositories, the discipline leaves no mark on the size of a change: median lines added per code-changing commit sit in a common band whether a repository is governed or not, and the one adequately powered governed repository is if anything larger than its ungoverned neighbours. The paper attributes this to the instrument rather than rescuing the claim — skedasis is a ratio and commit size measures only its numerator — but the prediction failed and is reported as a failure, not as a limitation of the study. Capture of deferred work is the one that separated: the parking lot fires in a fifth to two-fifths of code-changing commits in governed repositories and under five per cent in ungoverned ones.

`relearn` is in the governed arm of that commit-history comparison. It is **excluded** from the paper's second study — the hand-coded invented-requirements measurement — because its build sessions ran with `~/.claude` as the working directory, so its transcripts are co-mingled with hundreds of unrelated sessions in one store; the feasibility report records its pairability as **"plausible but unconfirmed."** Nothing here should be read as an efficacy claim for this repository. The discipline is implemented and the evidence for it is one refuted prediction, one correlational separation, and a single-author corpus that cannot establish causation either way.

For an end-to-end walk-through — one real incident becoming one rule and compiling out to all five instruction layers, with actual `relearn build` output — see [`docs/worked-example.md`](docs/worked-example.md).

The pipeline `rules/*.md → parse → validate → emit → write` works today for all five targets — Claude skills, Cursor rules, GitHub Copilot instructions, `AGENTS.md`, and a Claude project layer — and every v0.1 guarantee in `FEATURES.md` names a real enforcing artifact (types, property tests, and a compile-fail pin). A real rule set lives in [`rules/`](rules/), ported from George's engineering discipline and spanning global, domain-rust and two project homes, with active and graduated statuses. `relearn build` compiles it end to end, `relearn lint` reports advisory findings without ever touching a rule, and `relearn verify` checks the emitted tree has not drifted from it — the emitted tree is committed and CI runs `verify` bare on every push, so the command that grades the repository is the default one.

*(The corpus size is deliberately not written here. `relearn check` prints it — `ok: N rule(s) validated` — and a count in prose is a claim nothing checks, which is how this README came to say "fourteen" for two days after the fifteenth rule landed. `[R:doc-currency]`)*

## Usage

```sh
# Validate the rule library; write nothing. Non-zero exit on any failure.
relearn check --rules ./rules

# List rules as `tag  [home-slug]  title`, optionally filtered by home.
relearn list --rules ./rules
relearn list --rules ./rules --home domain-rust

# Compile the library to an output root. Defaults to every target
# (claude, cursor, copilot, agents, claude-rules); pass --targets to narrow.
relearn build --rules ./rules --out .
relearn build --rules ./rules --out . --targets cursor,copilot

# Narrow by audience rather than by owner. A rule declaring no `applies_to`
# is emitted whatever you ask for; only a scoped rule can be withheld, and
# only from an audience it does not name. Repeatable, composes with --home,
# and an audience no rule declares is an error rather than a quiet drop.
relearn build --rules ./rules --out . --scope rust
relearn build --rules ./rules --out . --scope rust --scope java

# Take a deliberate fork of a cached rule — a rule whose home is another
# install. A cache is never edited in place: that is a silent fork, and the
# write path refuses it. This is the loud one, and the adopted rule records
# what it was forked from, at which version, and when. The date is given
# rather than read from a clock, so the result is reproducible.
relearn adopt --rules ./rules --tag R:some-rule --on 2026-09-13

# Prepare one rule to leave this machine. The rule's `incident` is a verbatim
# quotation from a private session and never travels: what travels is
# `published_incident`, a separate field you write yourself. Prints exactly
# what would go and writes nothing without --confirm; the term list is
# required, because a scrub that can run disarmed is not a scrub.
relearn contribute --rules ./rules --tag R:some-rule --terms ~/terms.sha256
relearn contribute --rules ./rules --tag R:some-rule --terms ~/terms.sha256 --confirm

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

`home` says who **owns** a rule, and there is exactly one. An optional
`applies_to` says who should **load** it, and there may be several:

```
home        = { kind = "domain", name = "low-latency" }
applies_to  = ["rust", "java"]
```

One file, one tag, one incident, one owner — compiled into both language
builds. The alternative, a nested `rust/low-latency`, writes the shared
principle twice and it drifts from the first commit. Omit `applies_to` and the
rule serves every audience, which is what every rule written before the field
existed does.

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

`relearn` is the reference implementation of the error loop described in *Tuning the Stochastic Machine: A Systems Engineer's Operating Model for Human-AI Engineering* (George Andrikopoulos, 2026, [arXiv:2608.19125](https://arxiv.org/abs/2608.19125)) and of the Stochos framework's first two principles: **persist or perish**, and **one home per rule**. Its precision-measurement companion, *Grouping the Stochastic Machine* ([arXiv:2608.19140](https://arxiv.org/abs/2608.19140)), references this repository as its reference system.

The third paper in the series, *Aiming the Stochastic Machine: A Repository Discipline for First-Time-Right, and What Survived Measuring It* ([doi:10.5281/zenodo.22083202](https://doi.org/10.5281/zenodo.22083202); study materials at [doi:10.5281/zenodo.22082967](https://doi.org/10.5281/zenodo.22082967)), specifies the four-file repository discipline this repo is built on — `CLAUDE.md`, `ARCHITECTURE.md`, `TODO.md` as aiming inputs and `FEATURES.md` as the self-check — and names this repository as its worked case (§8). It is also where this repository's own defects are reported rather than quietly fixed first: §8 walks through the `claude-rules` target collision found here on 2026-08-20, and §9 generalises it into the sharpest limit of the enforced-by format, that the column records *enforcement* and not *invocation*. Both are closed here as of 2026-08-22, after the paper was submitted.

## Licence

Apache-2.0 — see [`LICENSE`](LICENSE). The explicit patent grant matters for anything intended to be safely adoptable inside enterprises.

## Author

George Andrikopoulos — United Kingdom.
