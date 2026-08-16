# FEATURES.md — relearn

**The regression ledger.** Every entry names the artifact that *enforces* it. No entry without an enforcing artifact — an unenforced guarantee is a documented wish, and this ledger must never let a wish look like a guarantee. Gaps are marked `NOTHING YET — exposed` and carried in TODO.md until closed.

> **This file is only load-bearing if it is read before every change.** Check 2 of the definition of done in `CLAUDE.md` requires re-reading this ledger and *running* the enforcing artifacts of any feature a change could plausibly touch: **a fix that breaks another documented feature is not a fix.** The "Enforced by" column is what makes that check executable rather than aspirational — it names precisely what to run.

*Status (2026-08-13): Phase A **runs end to end**. `relearn check | build | list` wire load → validate → emit → write through the typed pipeline; the binary has been smoke-tested through its real argv path, not only via the dispatch functions. Shipped and enforced: the value types, the library typestate, the neutral-format parser, the `fsio` read side, the **Claude skill emitter** (one skill per home layer — making the `Library<Validated>` typestate load-bearing), the **`fsio` write-side overwrite guard** (pre-flight marker check aborts the whole write rather than clobber unversioned content), and the **CLI** (`clap` derive; an unknown `--targets` value is a parse error before any write, because `Target` only names buildable emitters), the **Cursor emitter** (one `.mdc` per rule; `globs`/`alwaysApply` derived from `Home` via the shared `Scope` helper), and the **Copilot / `AGENTS.md` / project-`CLAUDE.md` emitters** (concatenated instruction files, home-rank ordered; `CLAUDE.md` is project-layer only). **All five emitters ship** — `relearn build` compiles the whole set by default and has been smoke-tested end to end (7 files from a 2-rule library). **Emission idempotence and neutral-format round-trip are now property-tested** (`tests/properties.rs`), and a `rule::to_document` serializer makes the pipeline lossless. **Phase A is complete** — every v0.1 guarantee names a real enforcing artifact (the last, "emitters reject an unvalidated library", closed by a trybuild compile-fail pin). **Phase B is advancing**: the advisory linter (`relearn lint`) ships overlapping-scope, home-slug-collision, dangling-reference, and retired-reference detection — reporting only, never destructive, with `Info` findings that inform without failing CI. Contradiction detection is handled as a **process-control** (a periodic Claude review pass; first pass 2026-08-13, clean). **Emit-status filtering ships** (decision 2026-08-13): atticked rules are suppressed from every emitter so withdrawn guidance never leaks into an active instruction file, and graduated rules are emitted annotated with the stronger control that also holds them — the policy lives in one exhaustive match (`Status::emittability`), applied once (`emit::emittable`), and is property-tested whole-space. **`relearn verify` ships** (2026-08-13): a read-only CI gate that classifies every generated file `Ok`/`Missing`/`Unversioned`/`HandEdited`/`Stale` against a fresh emission and exits non-zero on any drift — the detection complement to the write-side clobber guard (closes the Phase A hand-edit-detection item). **Phase B is functionally complete**: its one remaining item, cold-surface, is genuinely blocked on phase-C runtime data (skill-invocation counts live in stochos-lab, not this repo) and is tracked under Phase C where its data lives. 110 unit + 1 compile-fail + 12 lint + 4 property + 5 verify integration tests, all passing. The contradiction row stays `exposed` **by design** (no deterministic artifact to name, and faking one would be dishonest) and mirrored in TODO.md until their enforcing artifact ships, in the same change — never later.*

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
**Enforced by:** every emitter's `&Library<Validated>` signature (the typestate gate) **and** the compile-fail pin `tests/compile_fail/emit_rejects_unvalidated_library.rs` (driven by `tests/compile_fail.rs::typestate_gate_rejects_unvalidated_libraries`) — trybuild confirms the code is rejected *with the type mismatch* `expected &Library<Validated>, found &Library<Unvalidated>`, so a regression that relaxed an emitter to accept any `Library<S>` would fail the test.

### Claude skill emitter
What: produces one skill **per home layer** — `skills/<home-slug>/SKILL.md` with valid YAML front-matter (`name`, `description` aggregating that home's rules). Not one skill per rule (P6; decision 2026-08-13). The `description` value is YAML-double-quoted, so a `:` or `"` in a rule title stays valid front-matter.
**Enforced by:** `emit::claude::emit` (groups by `HomeSlug`, one `OutputFile` per home, rules sorted by tag) + `emit::claude` tests (`one_skill_per_home_layer`, `home_skill_aggregates_its_rules_sorted_by_tag`, `front_matter_names_the_home_slug`, `description_covers_each_rule_in_the_home`, `description_is_quoted_so_a_colon_in_a_title_stays_valid_yaml`).

### Cursor rules emitter
What: produces `.cursor/rules/<tag-body>.mdc` (one per rule; filename is the colon-free tag body) with front-matter `description`, `globs`, `alwaysApply`, where `globs`/`alwaysApply` are **derived from `Home`** via the shared `Scope` helper — not carried on the rule (P2; decision 2026-08-13). A global/project rule is `alwaysApply: true`; a known-language domain auto-attaches on that language's globs; an unknown domain is agent-requested (no globs, not always-on), never blanket-applied.
**Enforced by:** `emit::cursor::emit` + `emit::Scope::for_home` + the shared domain→glob table; `emit::cursor` tests (`one_mdc_per_rule_named_by_tag_body`, `rust_domain_rule_auto_attaches_on_rs_globs`, `global_rule_always_applies_with_no_globs`, `front_matter_is_delimited_and_ordered`, `emission_is_deterministic`) and `emit` `Scope` tests (`known_domain_scopes_to_language_globs_and_is_not_always`, `unknown_domain_has_no_globs_and_is_not_always`, `global_scope_always_applies_with_no_globs`, `domain_matching_is_case_insensitive_and_aliased`); the colon-free filename by `RuleTag::body` + `rule::tag` test `body_drops_the_prefix_and_is_a_safe_stem`.

### Copilot instructions emitter
What: produces a single `.github/copilot-instructions.md` holding every rule, ordered by home rank (global → domain → project), then home slug, then tag. An empty library emits no file.
**Enforced by:** `emit::copilot::emit` (+ shared `emit::home_rank`) + `emit::copilot` tests (`single_file_at_the_github_path`, `all_rules_present_ordered_by_home_then_tag`, `empty_library_emits_nothing`, `emission_is_deterministic`) + `emit` test `home_rank_orders_general_before_specific`.

### AGENTS.md emitter
What: produces a single `AGENTS.md` at the repository root holding every rule, same home-rank ordering. An empty library emits no file.
**Enforced by:** `emit::agents::emit` (+ shared `emit::home_rank`) + `emit::agents` tests (`single_file_at_the_repo_root`, `all_rules_present_ordered_by_home_then_tag`, `empty_library_emits_nothing`, `emission_is_deterministic`).

### Project `CLAUDE.md` emitter
What: produces a project-layer `CLAUDE.md` containing **only** `Home::Project` rules, ordered by tag. When there are no project rules it emits no file — never an empty `CLAUDE.md` that would clobber a hand-authored one.
**Enforced by:** `emit::claude_md::emit` (filters `Home::Project`) + `emit::claude_md` tests (`only_project_rules_are_included`, `file_is_at_claude_md_path`, `no_project_rules_emits_nothing`, `emission_is_deterministic`).

### Emission respects rule status — withdrawn guidance never leaks
What: every emitter draws from `emit::emittable(library)`: `active` and `graduated` rules are written; `attic` (retired) rules are **suppressed**, so a withdrawn rule is never written into an active instruction file (skill, `.mdc`, `AGENTS.md`, Copilot, or project `CLAUDE.md`) — not even into its provenance. A `graduated` rule *is* emitted, **annotated** with a `> Also enforced by <destination>.` line under its heading, because the instruction layer tunes generation before the fact whereas the graduated-to control catches after (decision 2026-08-13). The full library (atticked rules included) stays visible to `lint`, which needs retired rules present to flag references to them.
**Enforced by:** `Status::emittability` (the exhaustive-match policy — a new status variant cannot compile until its emit disposition is decided) + `emit::emittable` (applies it once for all five emitters) + `emit::graduation_note` (the annotation); the property tests `tests/properties.rs::atticked_rules_never_leak_into_any_emitter` (no atticked tag reaches any emitter's `sources`, over generated mixed-status libraries) and `active_and_graduated_rules_all_reach_copilot` (the suppression is exactly `attic`, no over-suppression); unit pins `rule::status::active_and_graduated_emit_but_attic_is_suppressed`, `emit::emittable_keeps_active_and_graduated_and_drops_attic`, `emit::graduation_note_only_annotates_graduated_rules`, `emit::copilot::atticked_rule_is_excluded_and_graduated_is_annotated`, `emit::claude::a_home_with_only_atticked_rules_produces_no_skill`, `emit::claude::a_graduated_rule_is_annotated_in_its_skill`.

### Emission is idempotent
What: re-running `build` on unchanged rules produces byte-identical output — every emitter is a deterministic function of the library, and the `fsio` write (header + `sha256`) reproduces the same bytes.
**Enforced by:** the property test `tests/properties.rs::emission_is_idempotent` (all five emitters, over generated libraries with varied homes) + each emitter's own `emission_is_deterministic` unit test + `fsio::tests::rewriting_the_same_files_is_byte_identical` (the write side, header hash included).

### Round-trip fidelity — the neutral format is lossless
What: a rule serialized to the neutral format and parsed back is identical — tag, title, error class, home, created date, status (with payload), and body all survive, including text that must be TOML-escaped. `relearn` can read, transform, and rewrite a rule without decay.
**Enforced by:** `rule::to_document` (the inverse of `parse_document`; TOML-escapes text, emits dates as quoted strings) + the property test `tests/properties.rs::neutral_round_trip_preserves_the_rule` + `rule::serialize` unit round-trips (`round_trips_quotes_and_backslashes_in_text`, `round_trips_graduated_status`, `round_trips_attic_status_and_project_home`, `round_trips_a_multi_line_body`).

### Generated files are never clobbered
What: writing refuses to overwrite a target file lacking the generated-by marker; a pre-flight pass aborts the **whole** write before touching disk if any target is unversioned, rather than deleting human-authored content (`[R:generate-guards-unversioned]`). A file relearn previously generated (marker present) is regenerated freely. Every written file carries the marker, the tool version, its source-rule tags, and a `sha256` of the body.
**Enforced by:** `fsio::write_all` (pre-flight marker check, then commit) + `fsio` tests (`refuses_to_clobber_a_file_lacking_the_marker`, `a_single_conflict_aborts_the_whole_write`, `overwrites_a_file_it_previously_generated`, `writes_files_creating_parent_dirs_and_appends_the_header`). *(The CLI surface that calls `write_all` and turns a `WriteError` into a process exit is still pending.)*

---

## CLI (v0.1)

### `relearn check` validates without writing
What: loads and validates the rules directory, writes nothing, and exits non-zero on any parse or validation failure.
**Enforced by:** `cli::check` (`load_rules` → `validate`, no write path) + `cli` tests (`check_reports_a_parse_failure_by_file`, `check_reports_a_duplicate_tag_as_a_validation_error`).

### `relearn build --targets`
What: loads, validates, emits the named targets, and writes them under `--out` through the overwrite guard. An unknown target name is a `clap` parse error, rejected before any file is written — because `Target` only has variants with a working emitter.
**Enforced by:** `cli::build` (`load_rules` → `validate` → `emit::*` → `fsio::write_all`) + the `Target` value-enum (an unbuildable target is unrepresentable) + `cli` tests (`build_runs_the_whole_pipeline_and_writes_skills`, `build_refuses_to_clobber_and_never_writes_a_partial_tree`).

### `relearn list --home`
What: lists rules as `tag  [home-slug]  title`, optionally filtered to one home layer by its slug.
**Enforced by:** `cli::list` (filters on `HomeSlug::of(rule.home())`) + `cli` test `list_without_a_filter_lists_all_and_a_missing_dir_is_an_error`.

### `relearn lint`
What: reports advisory findings; writes nothing and never deletes. Exits non-zero when any **actionable** finding (severity `Warning` or `Error`) is reported so CI catches a regression; `Info` findings print but do not fail the run; exit 0 on a clean library.
**Enforced by:** `cli::lint_rules` (`load_rules` → `validate` → `lint::lint`, no write path; `ExitCode::FAILURE` only when a `Warning`/`Error` finding exists) + the `lint` behavior below. Verified on the real seed (`rules/`): clean, exit 0; and on a synthetic `Info`-only case: prints, exit 0.

### `relearn verify` — the committed generated tree is in sync
What: reads only, writes nothing, and checks that the files `build` would emit from the current rules match what is on disk under `--out`. Each expected file is classified `Ok` / `Missing` / `Unversioned` (a non-generated file shadows the path) / `HandEdited` (body no longer hashes to its own header's recorded `sha256`) / `Stale` (self-consistent but not what the current rules emit); and any marker-bearing file in a relearn-owned location that the current rules no longer emit — its rule or home was deleted — is reported `Orphan`. Exits `FAILURE` if any file drifted, `SUCCESS` (exit 0) when all match — the CI gate that a checkout's generated files were not hand-edited, are not stale, and left no orphan behind. `build` and `verify` emit through the **same** `cli::emit_selected`, so they can never disagree about the expected set. **Ownership policy:** orphan detection scans only the subtrees relearn emits to (`skills/`, `.cursor/rules/`, and the fixed `AGENTS.md`/`CLAUDE.md`/`.github/copilot-instructions.md`) and flags only files carrying the generated-by marker, so a hand-authored file — even inside an owned dir — is never claimed.
**Enforced by:** `cli::verify` (`load_rules` → `validate` → `emit_selected` → `fsio::verify_all`; `ExitCode::FAILURE` iff any report is not `Ok`) + `fsio::verify_all` / `fsio::classify` / `fsio::parse_generated` (the integrity + freshness classification; `render_with_header` hashes the **body only** so a hand edit is detectable) + `fsio::scan_orphans` / `fsio::collect_files` (the owned-location, marker-scoped orphan pass) + `tests/verify.rs` (`freshly_built_files_verify_clean`, `a_missing_file_is_reported`, `a_hand_edited_body_is_detected`, `an_unversioned_file_at_a_target_path_is_reported`, `a_file_that_no_longer_matches_the_rules_is_stale`, `an_orphan_generated_file_is_reported`, `an_unmarked_file_in_an_owned_dir_is_not_an_orphan`) + `fsio` unit pins (`parse_generated_recovers_the_exact_body_and_hash`, `parse_generated_handles_a_body_without_a_trailing_newline`, `parse_generated_rejects_a_file_without_the_marker`, `verify_reports_ok_for_a_freshly_written_file_and_missing_when_absent`, `an_orphaned_fixed_target_is_reported`, `a_marked_file_outside_owned_locations_is_not_scanned`) + `cli` unit pins (`verify_succeeds_on_a_fresh_build_and_fails_after_drift`, `verify_fails_when_a_generated_file_is_missing`). Verified through the binary on the real seed: clean build → exit 0; hand-edited `AGENTS.md` → `hand-edited`, exit 1; a deleted rule → its `.cursor/rules/*.mdc` reported `orphan` (+ the concatenated files `stale`), exit 1.

---

## Linter (Phase B — advisory, never destructive)

The linter *reports*; it never edits or deletes a rule (a flagged rule is input to a human cut-list decision). Findings are sorted most-severe first.

### Overlapping-scope detection
What: two or more rules that declare the same error class (case-insensitive) are flagged as possibly redundant or in tension.
**Enforced by:** `lint::lint` (the `overlapping_scope` check) + `tests/lint.rs::overlapping_scope_flags_a_shared_error_class` and `a_clean_library_has_no_findings`.

### Home-slug-collision detection
What: two or more rules with **different** homes whose slugs collide (would silently share one emitted skill file — a lost rule) are flagged as an `Error`-severity finding. Rules in the *same* home sharing a slug are not flagged.
**Enforced by:** `lint::lint` (the `home_slug_collisions` check, over `emit::HomeSlug`) + `tests/lint.rs::home_slug_collision_flags_distinct_homes_with_the_same_slug`, `same_home_is_not_a_collision`, `findings_are_sorted_most_severe_first`. *(This closes the collision edge filed by the 2026-08-13 TDP scan.)*

### Dangling-reference detection
What: a rule that cites another rule's tag (`R:...`, in its body or incident) which is not present in the library is flagged (`Warning`). `OR:`/`FOR:` in prose is not mistaken for a citation, and a tag cited in both body and incident is one finding, not two.
**Enforced by:** `lint::lint` (`reference_checks` + `cited_tags`; citations are read from the rule **body only** — `incident` is provenance, not a citation surface, and scanning it made the linter flag its own history: `[R:detector-excludes-own-definitions]`, fixed 2026-08-16) + `tests/lint.rs::dangling_reference_flags_an_unknown_cited_tag`, `a_resolved_reference_is_not_flagged`, `or_in_prose_is_not_read_as_a_tag`, `a_tag_cited_twice_in_the_body_yields_one_finding`, `a_tag_named_only_in_provenance_is_not_a_dangling_reference`, `a_tag_cited_in_the_body_is_still_flagged`.

### Retired-reference detection
What: a rule that cites another rule which *exists* but is retired (atticked or graduated) is flagged `Info` — likely a historical pointer, but worth confirming the citation isn't building on withdrawn guidance.
**Enforced by:** `lint::lint` (the `reference_checks` check, over `rule::Status`) + `tests/lint.rs::retired_reference_flags_a_citation_of_a_retired_rule`, `a_retired_reference_is_info_severity`, `citing_an_active_rule_is_not_a_retired_reference`.

### Contradiction detection
What: two rules that semantically contradict each other.
**Enforced by:** a **process-control**, not a code artifact — a periodic *Claude review pass* over the corpus (the rule bodies are readable markdown; Claude judges consistency, delegated through the subscription per doctrine). Deliberately **not** a keyword heuristic in the linter: a heuristic dressed up as certainty would be worse than the honest absence. *Review 2026-08-13 (re-run after the corpus grew to fourteen rules): no contradictions, no problematic overlap, homes consistent (recorded in TODO.md). One relationship noted, not a contradiction: `R:prefer-by-construction` (design a mistake out rather than guard against it) and `R:generate-guards-unversioned` (a runtime guard protecting hand-authored files) are the hierarchy-of-controls in action — the guard is the honest fallback exactly where by-construction is unavailable, since "a human authored this file" cannot be made unrepresentable at the filesystem level. This row is `exposed` by design — there is no deterministic code artifact to name, and inventing one would be the dishonesty the tool exists to prevent.*

### Cold-surface / uncited report
What: rules that nothing exercises (candidates for the attic cut-list).
**Enforced by: NOTHING YET — relocated to Phase C** *(not a Phase B gap: "cold" needs runtime invocation data, which lives in the stochos-lab observability layers — Phase C — not in this repo's rule text. Reporting "uncited" alone would be noise, since a standalone rule is legitimately uncited. Tracked under Phase C in TODO.md until that data feed exists; not buildable here without faking a signal.)*

---

## Deliberately out of scope for v0.1

- **Recurrence / outcome instrumentation** — phase C; lives in the stochos-lab ledger, not here.
- **Importing existing rules from Cursor/Copilot formats** — reverse direction; only if a real need appears (P7: verify the target exists).
