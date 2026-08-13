# FEATURES.md — relearn

**The regression ledger.** Every entry names the artifact that *enforces* it. No entry without an enforcing artifact — an unenforced guarantee is a documented wish, and this ledger must never let a wish look like a guarantee. Gaps are marked `NOTHING YET — exposed` and carried in TODO.md until closed.

> **This file is only load-bearing if it is read before every change.** Check 2 of the definition of done in `CLAUDE.md` requires re-reading this ledger and *running* the enforcing artifacts of any feature a change could plausibly touch: **a fix that breaks another documented feature is not a fix.** The "Enforced by" column is what makes that check executable rather than aspirational — it names precisely what to run.

*Status (2026-08-13): Phase A is complete **through the first emitter**. Shipped and enforced: the value types, the library typestate, the neutral-format parser, the `fsio` read side (loading a rules directory, file-named diagnostics), and the **Claude skill emitter** (one skill per home layer) — which makes the `Library<Validated>` typestate load-bearing: `emit::claude::emit` accepts only a validated library, so emitting unchecked rules is a compile error. 58 tests, all passing. The remaining entries (the other four emitters, emission-idempotence as a property, round-trip fidelity, the CLI surface, and the `fsio` write-side overwrite guard) stay `NOTHING YET — exposed` and mirrored in TODO.md until their enforcing artifact ships, in the same change — never later.*

---

## Rule parsing (v0.1)

### Neutral rule format parses
What: `rules/<tag>.md` with TOML front-matter (`+++`) and markdown body parses into a `Rule`. Dates are quoted strings (parsed by `Date::parse`, not TOML) to keep the range check ours.
**Enforced by:** `rule::parse_document` + `rule::parse` tests (`parses_a_full_document`, `attic_status_parses_with_reason_and_date`).

### Malformed rule stops the build
What: any parse failure aborts with a diagnostic naming the file and field; no rule is ever skipped silently.
**Enforced by:** `rule::parse_document` returns a typed `ParseError` naming the field; `fsio::load_rules` wraps it as `LoadError::Parse { path, source }`, adding the file. Neither ever yields a partial or skipped `Rule`. Tests in `rule::parse` (field-level) and `fsio` (`a_bad_file_stops_the_load_and_names_it`). *(The `cli` surface that turns a `LoadError` into a process exit is still pending.)*

### A rules directory loads, or names the file that failed
What: `fsio::load_rules(dir)` reads every `*.md` in sorted (deterministic) order into `Library<Unvalidated>`; an unreadable or unparseable file stops the load with a `LoadError` naming it — never a silent skip.
**Enforced by:** `fsio::load_rules` + `fsio` tests (`loads_md_files_in_sorted_order_ignoring_others`, `a_bad_file_stops_the_load_and_names_it`, `missing_directory_is_a_readdir_error`, `empty_directory_loads_an_empty_library`).

### Tag shape is enforced at the perimeter
What: `RuleTag::parse` accepts only `R:[a-z0-9][a-z0-9-]*`; nothing downstream re-validates.
**Enforced by:** `RuleTag` (private field, constructible only via `RuleTag::parse`) + `rule::tag` tests (`wellformed_roundtrips`, `rejects_out_of_class`, and the `rejects_*` unit pins).

### Provenance is mandatory
What: a rule missing `created`, `incident`, or `error_class` fails to parse.
**Enforced by:** `RawRule` deserialization (a missing key → `ParseError::Toml`) + the non-empty newtypes (an empty value → `ParseError::Text`) + `rule::parse` tests. A missing or empty provenance field never yields a `Rule`.

### Duplicate tags are rejected
What: two rules sharing a tag is a library-level error.
**Enforced by:** `Library::validate` (checks tag uniqueness, returns `ValidationError::DuplicateTag`, never drops a rule) + `library` test `duplicate_tag_is_rejected_with_the_offending_tag`.

### One home per rule is unrepresentable otherwise
What: `Home` is a sum type; a rule cannot carry two homes.
**Enforced by:** the `Home` sum type (a rule holds one `Home`; each variant carries its own data) + `rule::home` tests.

### Out-of-range dates report as out-of-range
What: a date field of valid shape but impossible value reports a range error, not a syntax error (dogfoods `[R:parse-wide-then-range-check]`).
**Enforced by:** `Date::parse` (each component parsed into `i64` wide, then range-checked) + `rule::date` property test `out_of_range_month_is_named_not_syntax` and the `MonthOutOfRange`/`DayOutOfRange` unit pins.

### Provenance text is non-empty by construction
What: `ErrorClass`, `Incident` (and the `Destination`/`Reason`/`DomainName`/`ProjectPath` newtypes) reject empty/whitespace input at their `parse` perimeter — an empty provenance string is unrepresentable.
**Enforced by:** the newtypes over `rule::text::nonempty` + `rule::text` tests.

### Status payloads cannot be omitted
What: a `Graduated` status without a destination, or an `Attic` status without a reason and a date, cannot be constructed — the data lives in the variant.
**Enforced by:** the `Status` sum type + `rule::status` tests (`graduated_requires_destination`, `attic_requires_reason_and_carries_date`).

---

## Emission (v0.1)

### Emitters accept only validated libraries
What: `emit::*` takes `&Library<Validated>`; passing an unvalidated library does not compile.
**Enforced by:** `emit::claude::emit`'s signature `&Library<Validated>` — the first emitter to consume the typestate, so the compile-time gate is now load-bearing rather than latent. *(A `trybuild` compile-fail pin proving an `Unvalidated` library is rejected is still TODO — the negative is asserted by the type system today, not yet by a test.)*

### Claude skill emitter
What: produces one skill **per home layer** — `skills/<home-slug>/SKILL.md` with valid YAML front-matter (`name`, `description` aggregating that home's rules). Not one skill per rule (P6; decision 2026-08-13). The `description` value is YAML-double-quoted, so a `:` or `"` in a rule title stays valid front-matter.
**Enforced by:** `emit::claude::emit` (groups by `HomeSlug`, one `OutputFile` per home, rules sorted by tag) + `emit::claude` tests (`one_skill_per_home_layer`, `home_skill_aggregates_its_rules_sorted_by_tag`, `front_matter_names_the_home_slug`, `description_covers_each_rule_in_the_home`, `description_is_quoted_so_a_colon_in_a_title_stays_valid_yaml`).

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
**Enforced by:** the claude stage is a deterministic pure function (`emit::claude::tests::emission_is_deterministic` — same library → identical `Vec<OutputFile>`, homes ordered by slug, rules by tag). *The whole-`build` byte-identity property (across every emitter, and through the `fsio` write) is still `NOTHING YET — exposed`: target a property test over generated rule sets once more than one emitter exists.*

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
