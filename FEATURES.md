# FEATURES.md — relearn

**The regression ledger.** Every entry names the artifact that *enforces* it. No entry without an enforcing artifact — an unenforced guarantee is a documented wish, and this ledger must never let a wish look like a guarantee. Gaps are marked `NOTHING YET — exposed` and carried in TODO.md until closed.

> **This file is only load-bearing if it is read before every change.** Check 2 of the definition of done in `CLAUDE.md` requires re-reading this ledger and *running* the enforcing artifacts of any feature a change could plausibly touch: **a fix that breaks another documented feature is not a fix.** The "Enforced by" column is what makes that check executable rather than aspirational — it names precisely what to run.

*Status at scaffold (2026-08-13): nothing is implemented yet. Every entry below is therefore exposed by definition, and the whole list is mirrored in TODO.md. Entries move to an enforced state as they ship, in the same change as their enforcing artifact — never later.*

---

## Rule parsing (v0.1)

### Neutral rule format parses
What: `rules/<tag>.md` with TOML front-matter (`+++`) and markdown body parses into a `Rule`.
**Enforced by: NOTHING YET — exposed**

### Malformed rule stops the build
What: any parse failure aborts with a diagnostic naming the file and field; no rule is ever skipped silently.
**Enforced by: NOTHING YET — exposed**

### Tag shape is enforced at the perimeter
What: `RuleTag::parse` accepts only `R:[a-z0-9][a-z0-9-]*`; nothing downstream re-validates.
**Enforced by: NOTHING YET — exposed** *(target: the type itself — `RuleTag` constructible only via `parse`)*

### Provenance is mandatory
What: a rule missing `created`, `incident`, or `error_class` fails to parse.
**Enforced by: NOTHING YET — exposed**

### Duplicate tags are rejected
What: two rules sharing a tag is a library-level error.
**Enforced by: NOTHING YET — exposed**

### One home per rule is unrepresentable otherwise
What: `Home` is a sum type; a rule cannot carry two homes.
**Enforced by: NOTHING YET — exposed** *(target: the type itself)*

### Out-of-range dates report as out-of-range
What: a date field of valid shape but impossible value reports a range error, not a syntax error (dogfoods `[R:parse-wide-then-range-check]`).
**Enforced by: NOTHING YET — exposed**

---

## Emission (v0.1)

### Emitters accept only validated libraries
What: `emit::*` takes `&Library<Validated>`; passing an unvalidated library does not compile.
**Enforced by: NOTHING YET — exposed** *(target: typestate — a compile-fail test)*

### Claude skill emitter
What: produces one skill **per home layer** — `skills/<home>/SKILL.md` with valid YAML front-matter (`name`, `description` aggregating that home's rules). Not one skill per rule (P6; decision 2026-08-13).
**Enforced by: NOTHING YET — exposed**

### Cursor rules emitter
What: produces `.cursor/rules/<tag>.mdc` with valid YAML front-matter (`description`, `globs`, `alwaysApply`), where `globs`/`alwaysApply` are **derived from `Home`** via the shared scope helper — not carried on the rule (P2; decision 2026-08-13).
**Enforced by: NOTHING YET — exposed**

### Copilot instructions emitter
What: produces a single `.github/copilot-instructions.md`, rules ordered by home then tag.
**Enforced by: NOTHING YET — exposed**

### AGENTS.md emitter
What: produces a single `AGENTS.md`.
**Enforced by: NOTHING YET — exposed**

### Emission is idempotent
What: re-running `build` on unchanged rules produces byte-identical output.
**Enforced by: NOTHING YET — exposed** *(target: property test over generated rule sets)*

### Round-trip fidelity
What: parse → emit → re-parse (for formats that support it) preserves tag, home, status, and body.
**Enforced by: NOTHING YET — exposed** *(target: property test)*

### Generated files are never clobbered
What: `build` refuses to overwrite a target file lacking the generated-by header and hash; it atticks or aborts rather than deleting unversioned content (`[R:generate-guards-unversioned]`).
**Enforced by: NOTHING YET — exposed**

---

## CLI (v0.1)

### `relearn check` validates without writing
What: exits non-zero on any parse or validation failure; writes nothing.
**Enforced by: NOTHING YET — exposed**

### `relearn build --targets`
What: emits the named targets; unknown target names are rejected before any file is written.
**Enforced by: NOTHING YET — exposed**

### `relearn list --home`
What: lists rules filtered by home layer.
**Enforced by: NOTHING YET — exposed**

---

## Deliberately out of scope for v0.1

- **Linter for contradictory or overlapping rules** — phase B; requires the format to settle first.
- **Recurrence / outcome instrumentation** — phase C; lives in the stochos-lab ledger, not here.
- **Importing existing rules from Cursor/Copilot formats** — reverse direction; only if a real need appears (P7: verify the target exists).
