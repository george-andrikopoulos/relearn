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
| `rule` | The neutral rule type and its newtypes (`RuleTag`, `ErrorClass`, `Incident`, `Home`, `Status`). Parsing lives here. | Know anything about output formats |
| `library` | Collection semantics: uniqueness of tags, home partitioning, the typestate (`Unvalidated`/`Validated`) | Perform I/O |
| `emit` | One submodule per target: `claude`, `cursor`, `copilot`, `agents`, `claude_md`. Pure functions `&Library<Validated> -> Vec<OutputFile>` | Read the filesystem, or read anything not carried by the rule |
| `fsio` | All filesystem reads and writes, including the generated-file guard | Contain business logic |
| `cli` | Argument parsing, command dispatch, human-readable diagnostics | Contain business logic |

The dependency direction is strictly `cli → library → rule` and `cli → emit → library`; `fsio` is a leaf used by `cli` only. `emit` never touches `fsio` — emitters return values, the caller writes them. This is what makes emitters trivially testable.

## The neutral rule format

```
+++
tag        = "R:parse-wide-then-range-check"
title      = "Parse wide, then range-check"
error_class = "Range-validating by parsing into the target narrow type, making the out-of-range case unreachable"
home       = { kind = "domain", name = "rust" }
created    = 2026-07-23
status     = { kind = "active" }
incident   = "Grouping task 01: 5/5 samples parsed into u16, so 70000 returned NotANumber; OutOfRange unreachable."
+++

Parse into a type wide enough to *represent* the out-of-range value, then range-check
to mint the narrow newtype. The perimeter must be able to see the illegal value in
order to name it illegal.
```

`status` is a sum type, so a graduated or atticked rule carries its destination:
`{ kind = "graduated", to = "hook:no-narrow-parse" }`, `{ kind = "attic", reason = "...", date = 2026-09-01 }`.

## Type-driven decisions

- **`RuleTag`** is a newtype minted only by `RuleTag::parse`, which enforces the `R:[a-z0-9][a-z0-9-]*` shape. Nothing downstream re-checks; possessing a `RuleTag` *is* the proof.
- **`Home`** is an enum (`Global | Domain { name } | Project { path }`). One home per rule is a type-level guarantee, not a lint.
- **`Status`** is an enum carrying its payload, so "graduated" without a destination, or "attic" without a reason, cannot be constructed.
- **Typestate on `Library<S>`** — `emit` accepts only `Library<Validated>`, so emitting unvalidated rules does not compile.
- **Parse wide, then range-check** applies to date fields: parse into a full date type, then constrain, so an out-of-range date reports as out-of-range rather than as a syntax error. (Dogfooding `[R:parse-wide-then-range-check]`.)

## Emission targets

| Target | Output | Notes |
|---|---|---|
| `claude` | `<out>/skills/<name>/SKILL.md` | YAML front-matter: `name`, `description` (built from title + error class for trigger coverage) |
| `cursor` | `<out>/.cursor/rules/<tag>.mdc` | YAML front-matter: `description`, `globs`, `alwaysApply` |
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
