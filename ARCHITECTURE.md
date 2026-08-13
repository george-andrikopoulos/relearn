# ARCHITECTURE.md — relearn

## Shape

A three-stage pipeline, with a typestate boundary between each stage so that later stages cannot receive earlier-stage data:

```
rules/*.md  ──parse──▶  Library<Unvalidated>  ──validate──▶  Library<Validated>  ──emit──▶  target files
                 │                                    │                              │
             ParseError                          ValidationError               io::Error
             (stops build)                       (stops build)                 (stops build)
```

No stage is permitted to skip a failing rule. A dropped rule is a lost correction.

## Crates / modules

| Module | Responsibility | Must NOT |
|---|---|---|
| `rule` | The neutral rule type and its newtypes (`RuleTag`, `ErrorClass`, `Incident`, `Home`, `Status`). Parsing **and** serialization (`parse_document` / `to_document`, inverses) live here. | Know anything about output formats |
| `library` | Collection semantics: uniqueness of tags, home partitioning, the typestate (`Unvalidated`/`Validated`) | Perform I/O |
| `emit` | One submodule per target: `claude`, `cursor`, `copilot`, `agents`, `claude_md`. Pure functions `&Library<Validated> -> Vec<OutputFile>`. Shared scope/ordering helpers (`Scope`, `home_rank`, `HomeSlug`) | Read the filesystem, or read anything not carried by the rule |
| `lint` | Advisory static analysis over `&Library<Validated>` → `Vec<Finding>`: overlapping scope, home-slug collision, dangling references, retired references. Reports only; findings carry a `Severity` (`Info`/`Warning`/`Error`). | Modify or delete rules, or perform I/O |
| `fsio` | All filesystem reads and writes, including the generated-file guard | Contain business logic |
| `cli` | Argument parsing, command dispatch, human-readable diagnostics | Contain business logic |

The dependency direction is strictly `cli → library → rule` and `cli → emit → library`; `lint → {library, emit, rule}` (it reuses `emit::HomeSlug` because a slug collision is defined by emit's output paths); `fsio` is a leaf used by `cli` only. `emit` never touches `fsio` — emitters return values, the caller writes them. This is what makes emitters trivially testable. The `lint` module is pure and never writes: a flagged rule is input to a human decision, never auto-deleted.

## The neutral rule format

```
+++
tag        = "R:parse-wide-then-range-check"
title      = "Parse wide, then range-check"
error_class = "Range-validating by parsing into the target narrow type, making the out-of-range case unreachable"
home       = { kind = "domain", name = "rust" }
created    = "2026-07-23"     # quoted string, parsed by Date::parse (see decisions log)
status     = { kind = "active" }
incident   = "Grouping task 01: 5/5 samples parsed into u16, so 70000 returned NotANumber; OutOfRange unreachable."
+++

Parse into a type wide enough to *represent* the out-of-range value, then range-check
to mint the narrow newtype. The perimeter must be able to see the illegal value in
order to name it illegal.
```

`status` is a sum type, so a graduated or atticked rule carries its destination:
`{ kind = "graduated", to = "hook:no-narrow-parse" }`, `{ kind = "attic", reason = "...", date = "2026-09-01" }`.

## Type-driven decisions

- **`RuleTag`** is a newtype minted only by `RuleTag::parse`, which enforces the `R:[a-z0-9][a-z0-9-]*` shape. Nothing downstream re-checks; possessing a `RuleTag` *is* the proof.
- **`Home`** is an enum (`Global | Domain { name } | Project { path }`). One home per rule is a type-level guarantee, not a lint.
- **`Status`** is an enum carrying its payload, so "graduated" without a destination, or "attic" without a reason, cannot be constructed.
- **Typestate on `Library<S>`** — `emit` accepts only `Library<Validated>`, so emitting unvalidated rules does not compile.
- **Parse wide, then range-check** applies to date fields: parse into a full date type, then constrain, so an out-of-range date reports as out-of-range rather than as a syntax error. (Dogfooding `[R:parse-wide-then-range-check]`.)

## Emission targets

| Target | Output | Notes |
|---|---|---|
| `claude` | `<out>/skills/<home>/SKILL.md` | **One skill per home layer** (not per rule — decision 2026-08-13). `<name>` is the home; `description` aggregates that home's rules for trigger coverage |
| `cursor` | `<out>/.cursor/rules/<tag-body>.mdc` | One `.mdc` per rule (Cursor's native granularity). Filename is the tag **body** (`R:foo` → `foo.mdc`) — the full tag's `:` is not a valid filename character everywhere. Front-matter `description`, `globs`, `alwaysApply` — **`globs`/`alwaysApply` derived from `Home`** via the shared `Scope` helper (domain→pattern table) in `emit`, never carried on the rule (decision 2026-08-13). Values are **not** YAML-quoted: Cursor parses `.mdc` front-matter leniently and `globs` holds a raw glob list |
| `copilot` | `<out>/.github/copilot-instructions.md` | Single concatenated file; rules ordered by home then tag |
| `agents` | `<out>/AGENTS.md` | Single concatenated file |
| `claude_md` | `<out>/CLAUDE.md` | Project-layer rules only |

Every emitted file carries a generated-by header naming the source rule(s) and a hash, which is what the overwrite guard checks.

## Decisions log

| Date | Decision | Why | Rejected alternative |
|---|---|---|---|
| 2026-08-13 | TOML front-matter, `+++` delimiters | `toml` crate is well maintained in Rust; YAML's Rust ecosystem is fragmented (`serde_yaml` unmaintained) | YAML front-matter — more conventional, worse maintained |
| 2026-08-13 | One file per rule, not one library file | Diffable, individually attributable, survives concurrent edits; matches "one home per rule" physically | Single `rules.toml` — simpler parse, worse provenance |
| 2026-08-13 | Emitters are pure, `fsio` writes | Makes every emitter unit-testable without a filesystem; keeps the destructive operation in one audited place | Emitters write directly — fewer types, untestable |
| 2026-08-13 | Parse failure stops the build | A silently skipped rule is a lost correction — the exact failure this project exists to prevent | Warn and continue — friendlier, unsafe |
| 2026-08-13 | Provenance fields are mandatory | The governance argument rests on provenance; optional provenance makes the tool complicit in sediment | Optional fields with defaults |
| 2026-08-13 | Type-Driven Design, not test-first | Standing doctrine (`rust-typedd`): a type constrains the whole space, a test samples points; in an AI-assisted pipeline the compiler is the cheapest retirement stage | TDD-first |
| 2026-08-13 | Claude skill emitter is one skill **per home layer**, not per rule | The skill *description* is always-resident metadata (P6, context is finite); per-rule grows it `O(rules)` and this library is built to grow. Matches the existing hand-authored skills (`project-discipline`, `rust-typedd`) which bundle many rules by domain; domain-level triggering is the right granularity. Future escape hatch: an optional `skill_group` field only if one home ever needs more than one skill. | One skill per rule — precise triggers, but unbounded resident-metadata growth |
| 2026-08-13 | Cursor `globs`/`alwaysApply` are **derived from `Home`**, not carried as target-specific fields | `Home` already *is* the neutral scoping concept; translating it into Cursor's vocabulary is the emitter's job, and vendor knowledge belongs in the emitter. Rule-level `cursor.*` fields would fragment the neutral format (every vendor then wants its own), contradicting P2. If finer scope than `Home` expresses is ever needed, enrich `Home` so every emitter benefits — never add a Cursor-only field. | Optional `cursor.*` fields on the rule — flexible, but rots the neutral format into a union of vendor front-matters |
| 2026-08-13 | Dates in front-matter are **quoted strings**, parsed by `Date::parse`, not TOML's native date literal | Preserves the `[R:parse-wide-then-range-check]` dogfood and the "out-of-range dates report as out-of-range" feature end to end: an impossible date (`2026-13-01`) must surface as `MonthOutOfRange(13)`, but a native TOML date would reject it as a *syntax* error first, collapsing the class. A quoted string reaches `Date::parse` intact. | Native TOML dates — nicer syntax, but defeats the dogfood and couples the range semantics to the TOML crate |
| 2026-08-13 | The generated-by header (marker + version + source tags + `sha256` of the body) is rendered and applied by **`fsio` at write time**, not by the emitters | Keeps every emitter a pure `&Library<Validated> → Vec<OutputFile>` body-producer with no hashing concern, and centralizes the marker format in the one module (`fsio`) that also reads it back — the writer and the overwrite-guard checker cannot drift. The emitter still supplies provenance via `OutputFile.sources`; `fsio` only serializes it. Guard placement: an HTML comment **after** the body, valid in every current (markdown-family) target and never disturbing the leading YAML front-matter of a skill or `.mdc`. | Emitters embed the header — spreads the marker format across five emitters, couples each to `sha2`, and forces the checker to trust five independent renderers to stay byte-identical |
| 2026-08-13 | The write guard is **all-or-nothing** via a pre-flight pass: check every target for the marker first, abort the whole write before touching disk if any existing target is unversioned | A single human-authored collision must never leave a half-generated tree; refusing up front is the safe failure. Matches "a dropped rule is a lost correction" applied to the write side — do not destroy unversioned content. | Write-as-you-go, error on first conflict — simpler, but leaves partial output behind |
| 2026-08-13 | **Contradiction detection is a Claude review pass** (a governance process-control), not a `lint` code check | Contradiction is a semantic judgment. Per the model-selection doctrine, intelligence-needing work is delegated to Claude through the subscription; a keyword heuristic in the linter would be a probabilistic guess sold as a deterministic guarantee — exactly the sediment the tool fights. The corpus (`rules/*.md`) is readable; Claude judges consistency periodically (first pass 2026-08-13, clean). | A keyword/similarity heuristic in `lint` — deterministic-looking, but dishonest and prone to both false positives and misses |
