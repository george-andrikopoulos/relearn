# CLAUDE.md — relearn

*The recreation standard: from this file alone, this project could be rebuilt.*

## What this is

`relearn` is a Rust CLI that stores an engineering team's correction-derived rules **once**, in a neutral versioned form, and **compiles** them out to every AI assistant's native instruction layer — Claude skills, Cursor `.mdc` rules, GitHub Copilot instruction files, `AGENTS.md`, plain `CLAUDE.md`.

It is the reference implementation of the error loop described in *Tuning the Stochastic Machine* (George Andrikopoulos, 2026, arXiv:2608.19125) and of the Stochos framework's P1 (persist or perish) and P2 (one home per rule). The precision-measurement companion, *Grouping the Stochastic Machine* (arXiv:2608.19140), references this repository.

**The problem it solves.** When an expert corrects an AI assistant, the correction dies with the session unless it is written into the instruction layer. But every assistant has its own instruction format, so a rule written for one does not travel — the correction persists but is *locked to a vendor*. `relearn` makes the rule the artifact and the vendor format a build target.

## Core design decisions (and why)

1. **One rule = one file, neutral format.** `rules/<tag>.md`, TOML front-matter delimited by `+++`, imperative markdown body after. TOML because the parser is well maintained in Rust; front-matter because the body must stay human-editable and diff-friendly. Rules are the source of truth; every emitted format is a build artifact and is never hand-edited.
2. **Emitters are pure functions of validated rules.** No emitter may read anything the neutral rule does not carry. If a target needs data, the data belongs in the rule.
3. **Type-Driven Design, not test-first** (see `rust-typedd`). Illegal states are made unrepresentable; validation happens once at the perimeter and mints a witness type; interior code never re-checks. Property tests carry behavioural laws; unit tests are regression pins for past bugs.
4. **Provenance is mandatory, not optional.** A rule without a date, a triggering incident, and an error class does not parse. The framework's whole governance argument rests on provenance; making it optional would make the tool complicit in the failure it exists to prevent.
5. **Emission is idempotent and destructive-safe.** Re-running the compiler must produce byte-identical output for unchanged rules, and must refuse to overwrite a target file it did not generate (see `[R:generate-guards-unversioned]`).

## Invariants that must never break

- A rule that fails to parse **stops the build** — it is never silently skipped. A silently dropped rule is a correction lost, which is the exact failure the project exists to prevent.
- No emitted artifact is a source. Anything under an emitter's output path may be regenerated at any time and must never be hand-edited.
- Every rule has exactly **one home** (`Home::Global | Domain | Project`). Two homes for one rule is unrepresentable in the type system, not merely discouraged.
- Tags are unique across the library; a duplicate tag is a parse error.

## Build, run, test

```bash
cargo build --release
cargo test                     # unit + property tests
cargo clippy -- -D warnings
cargo fmt --check

relearn build --targets claude,cursor,copilot,agents   # compile rules to all targets
relearn check                                          # validate library, no output written
relearn list --home global                             # inspect
```

## Definition of done — runs on EVERY feature or fix

Binding for this repository. A change is not complete until all five pass, in order:

1. **Code and its enforcing artifact ship together.** The type, property test, or unit test that locks the new behaviour is in the same change — never "tests later."
2. **Regression pass.** Re-read `FEATURES.md`; run the enforcing artifacts of every feature the change could plausibly touch. **A fix that breaks another documented feature is not a fix.** This is the control that keeps the ledger true: features are only protected from each other if the enforcing artifacts are actually run.
3. **`FEATURES.md` updated** — a new entry with its enforcing artifact, or an existing entry's artifact revised. A feature with no enforcing artifact is recorded as `NOTHING YET — exposed` and carried in `TODO.md`.
4. **`CLAUDE.md` / `ARCHITECTURE.md` sync.** Ask explicitly: could this file still recreate the project? Does the architecture still describe it? Any design decision made goes into the decisions log with its *why*. Skipping this check is how the recreation standard becomes a lie.
5. **`TODO.md` updated** — done items cleared, discovered work added.

State the five results briefly at the end of any task. If a request conflicts with the discipline ("just patch it quickly"), do the patch, then say which checks were skipped and what exposure that creates — never silently drop it, never block the user with ceremony.

*This is the project-layer application of the author's `project-discipline` skill. It is written here, not merely referenced, because this repository will be published: an external contributor has no access to that skill, and a control that applies only when a skill happens to load is not a control.*

## Design discipline: Type-Driven, not test-first

Standing and deliberate (`rust-typedd`). A test samples points of the behaviour space; a type constrains the whole space and the compiler proves it everywhere, at compile time, forever. In a codebase written with AI assistance, the type system is the only deterministic, whole-space checker available — a retirement stage in silicon. Order of application:

1. **Types** — structure, states, boundaries, invariants (illegal states unrepresentable; parse, don't validate; parse wide then range-check).
2. **Property-based tests** — behavioural laws over generated input spaces (`proptest`).
3. **Unit tests** — regression pins for past bugs, documentary examples, edge cases neither of the above can express.
4. **Prose** — only what none of the above can hold; anything held only in prose is flagged as unenforced.

Unit testing is **not** removed — it is demoted to the layer where it is the right tool. Every bug fix ships its pinning test (check 1 above). Never delete a failing test to make a change pass.

## Standing rules referenced by this repo

These come from the author's rule library and are cited by tag elsewhere in these files. Defined here so no reference dangles for a reader without that library; they are scheduled for migration into `rules/` in Phase A, at which point this section points there instead.

- **`[R:wired-artifact]`** — a success check must consume a write-once sentinel unique to the verified artifact class, emitted as the final act of the success path and producible by nothing else. Pattern-matching on dates, headers, or file presence is not verification. Ask: *what other process can produce the string my check accepts?*
- **`[R:generate-guards-unversioned]`** — a script that regenerates a directory by delete-and-rebuild must refuse, or attic with provenance, when untracked content is present. Never silent `rm`.
- **`[R:pin-eol-for-executable-text]`** — pin end-of-lines for any executable text layer (`.gitattributes`, `text eol=lf`). A platform line-ending default silently breaks the shell layer on fresh clone, with no error. Fix structurally, not with a check.
- **`[R:revision-integrity]`** — after restructuring any document, verify references as a distinct pass: pronouns and comparatives resolve in the *current* text, cross-references point where they claim, announced counts match, terms are defined before use. Ask: *what did this edit quietly leave pointing at nothing?*
- **`[R:parse-wide-then-range-check]`** — parse into a type wide enough to *represent* the out-of-range value, then range-check to mint the narrow newtype. The perimeter must be able to see the illegal value in order to name it illegal.

## Where the rest is

- `ARCHITECTURE.md` — modules, data flow, the decisions log.
- `FEATURES.md` — the regression ledger; every feature names the artifact that enforces it.
- `TODO.md` — open work, including every `NOTHING YET — exposed` gap from FEATURES.md.
- `README.md` — the public face.

## Domain rules referenced, not duplicated

This project is governed by the author's standing skills: `rust-typedd` (design discipline), `project-discipline` (these five files and the definition of done), `relearn` (the error loop itself). **Do not copy their content into this repo** — reference them. Duplicated domain rules fork and rot (P2).
