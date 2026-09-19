# FEATURES.md — relearn

**The regression ledger.** Every entry names the artifact that *enforces* it. No entry without an enforcing artifact — an unenforced guarantee is a documented wish, and this ledger must never let a wish look like a guarantee. Gaps are marked `NOTHING YET — exposed` and carried in TODO.md until closed.

> **This file is only load-bearing if it is read before every change.** Check 2 of the definition of done in `CLAUDE.md` requires re-reading this ledger and *running* the enforcing artifacts of any feature a change could plausibly touch: **a fix that breaks another documented feature is not a fix.** The "Enforced by" column is what makes that check executable rather than aspirational — it names precisely what to run.

*Status (2026-08-13): Phase A **runs end to end**. `relearn check | build | list` wire load → validate → emit → write through the typed pipeline; the binary has been smoke-tested through its real argv path, not only via the dispatch functions. Shipped and enforced: the value types, the library typestate, the neutral-format parser, the `fsio` read side, the **Claude skill emitter** (one skill per home layer — making the `Library<Validated>` typestate load-bearing), the **`fsio` write-side overwrite guard** (pre-flight marker check aborts the whole write rather than clobber unversioned content), and the **CLI** (`clap` derive; an unknown `--targets` value is a parse error before any write, because `Target` only names buildable emitters), the **Cursor emitter** (one `.mdc` per rule; `globs`/`alwaysApply` derived from `Home` via the shared `LoadSemantics` helper), and the **Copilot / `AGENTS.md` / project-`CLAUDE.md` emitters** (concatenated instruction files, home-rank ordered; `CLAUDE.md` is project-layer only). **All five emitters ship** — `relearn build` compiles the whole set by default and has been smoke-tested end to end (7 files from a 2-rule library). **Emission idempotence and neutral-format round-trip are now property-tested** (`tests/properties.rs`), and a `rule::to_document` serializer makes the pipeline lossless. **Phase A is complete** — every v0.1 guarantee names a real enforcing artifact (the last, "emitters reject an unvalidated library", closed by a trybuild compile-fail pin). **Phase B is advancing**: the advisory linter (`relearn lint`) ships overlapping-scope, home-slug-collision, dangling-reference, and retired-reference detection — reporting only, never destructive, with `Info` findings that inform without failing CI. Contradiction detection is handled as a **process-control** (a periodic Claude review pass; first pass 2026-08-13, clean). **Emit-status filtering ships** (decision 2026-08-13): atticked rules are suppressed from every emitter so withdrawn guidance never leaks into an active instruction file, and graduated rules are emitted annotated with the stronger control that also holds them — the policy lives in one exhaustive match (`Status::emittability`), applied once (`emit::emittable`), and is property-tested whole-space. **`relearn verify` ships** (2026-08-13): a read-only CI gate that classifies every generated file `Ok`/`Missing`/`Unversioned`/`HandEdited`/`Stale` against a fresh emission and exits non-zero on any drift — the detection complement to the write-side clobber guard (closes the Phase A hand-edit-detection item). **Phase B is functionally complete**: its one remaining item, cold-surface, is genuinely blocked on phase-C runtime data (skill-invocation counts live in stochos-lab, not this repo) and is tracked under Phase C where its data lives. 116 unit + 1 compile-fail + 14 lint + 6 property + 7 verify integration tests, all passing. The contradiction row stays `exposed` **by design** (no deterministic artifact to name, and faking one would be dishonest) and mirrored in TODO.md until their enforcing artifact ships, in the same change — never later.*

*Update (2026-08-22): **the gate is now invoked.** Two things were true of the `relearn verify` row and neither was in this ledger — CI never ran it, and the emitted tree was untracked, so there was nothing on a clean checkout for it to verify. Both are closed: the tree is committed (23 files) and `ci.yml` runs `verify` bare on both OSes. In the same change the `claude-rules` target moved off the repository-root `CLAUDE.md` — the collision Paper 3 §8 reports as still open — to `.claude/rules/<home-slug>.md`, with the path derived from `Home`; **bare `relearn verify` now exits 0 on a clean tree**, so the default invocation is the operative one. Fixing it surfaced a second, unrecorded defect: the emitter had been pooling every project home into one file, so this repo's project layer carried stochos-lab's rules. Both are locked by whole-space property tests, not only by pins.*

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

### A rule records when it failed again (recurrence)
What: a rule carries zero or more `[[recurrence]]` tables — `date` plus `incident` — recording later occurrences of the error class it already covers. Recurrence is the one observation that says whether a rule is working; without it a rule that has bitten three times is indistinguishable from one written once and never seen again. A recurrence is deliberately **not** a second element of an `incidents` list: the triggering incident's date is the rule's `created` and a recurrence has none, so it carries its own, and collapsing the two would make `incidents[0]` mean something no other element means. Absent recurrences parse as an empty vector and render nothing, so the format change needed **no migration** — none of the forty-six committed rules was edited.
**Enforced by:** the `rule::Recurrence` type (private fields, `Date` + `Incident` witnesses, so a dateless or empty-text recurrence is unconstructible) + `Rule::new`'s ninth **required** argument (dropping a rule's recurrence history is a compile error at the site that drops it, not a silent loss) + `rule::parse` (`#[serde(default)]` on the field, `deny_unknown_fields` on the table; a malformed one stops the build, tests `recurrences_parse_as_an_array_of_tables`, `a_recurrence_with_a_bad_date_is_a_range_error_naming_the_field`, `a_recurrence_with_an_empty_incident_is_rejected`, `an_unknown_field_in_a_recurrence_is_rejected`, `a_rule_without_recurrences_parses_to_none`) + `rule::serialize` (tables emitted last and only when non-empty — `a_rule_with_no_recurrences_renders_no_recurrence_table`, `recurrence_tables_are_emitted_after_the_scalar_fields`, `round_trips_recurrences`) + `rule::def` unit pins (`latest_recurrence_is_the_maximum_date_not_the_last_entry`, `recurrences_are_kept_in_file_order`).

**Counter-metric — SHIPPED 2026-09-06**, see *"The counter-metric ships"* below. This row was recorded as `NOTHING YET — exposed` when recurrence landed, and closed the same week rather than left standing. P5: recurrences are only ever recorded by someone willing to write down that their own rule failed, so the count falls through under-reporting exactly as easily as through prevention. The counter is the corpus's *inert* fraction — never recurred **and** never mined from a real failure — which needed a second modelled fact (`origin`) before it could be computed at all. It now can be: `origin` is mandatory on every rule and `relearn lint` prints both numbers together, never the recurrence count alone.

### No rule file needs editing when the format grows
What: parsing then re-serializing any committed rule reproduces the file **byte for byte**. This is what makes "this format change needs no migration" a fact rather than a hope: a field that changed the rendering of a rule not using it would rewrite the whole library on the next build, and nothing else in the suite would notice — the property tests generate rules rather than reading these, and `verify` checks the *emitted* tree, not the sources it was emitted from.
**Enforced by:** `tests/corpus.rs::every_committed_rule_round_trips_byte_identically` (over the real `rules/` directory, 46 files) + `every_committed_rule_parses` (a parse regression and a serializer regression are separable faults) + the property test `tests/properties.rs::neutral_round_trip_preserves_the_rule` (generated rules, recurrences included). Line endings for `rules/**` are pinned `text eol=lf` in `.gitattributes`, because the comparison is byte-exact and a CRLF checkout would fail it on Windows only — `[R:pin-eol-for-executable-text]`, the same fix already applied to the emitted tree.

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
What: produces one skill **per home layer** — `skills/<home-slug>/SKILL.md` with valid YAML front-matter (`name`, `description`). Not one skill per rule (P6; decision 2026-08-13). The `description` value is YAML-double-quoted, so a `:` stays valid front-matter. Since 2026-09-19 the only string reaching that field that can carry one is a **project home's path**, which on Windows begins `C:` — titles no longer reach it and a tag cannot contain a colon.

### A skill `description` is a bounded trigger, not an inventory
What: `description` is a lead sentence saying *when* the layer applies (derived from `Home`), followed by as many of the home's rules as fit **1024 characters**, with any remainder counted (`; +7 more`). **The subject of that list was a rule's title until 2026-09-19 and is its tag body now** — see *"A skill description names every rule in its home, and no longer truncates"* below, which is where the change and its budget check are recorded. What this row states is the *shape*, unchanged since 2026-09-06: a lead sentence, a bounded list, a counted remainder. What the later fix changed is that on this corpus the bound no longer binds. Error classes are excluded: they are reviewer-facing prose about the failure, useless to a matcher, and they are what made the field overflow. The full text of every rule remains in the file body, so the cap bounds the trigger and not the content.

*Why this is a correctness row and not a style one (fixed 2026-09-06).* `description` is the only string Claude reads when deciding whether to load a skill, and it is capped at install time. The previous format interpolated every rule's title **and error class**, producing **6924** characters for `global` and **4322** for `domain-rust` against a 1024 cap — so the two skills that matter were **uninstallable**, and would have been useless matchers if they had installed. Nothing caught it because nothing measured it: the emitter's unit tests build two-rule libraries, where the old format is comfortably short and stays short forever. The bound only binds at the size the real library reached.

**Enforced by:** `emit::claude::SkillDescription` (private field; the only constructor truncates, so an over-cap description is unrepresentable rather than a bug to catch — `[R:prefer-by-construction]`) + `tests/corpus.rs::every_emitted_skill_description_fits_the_frontmatter_cap`, which asserts the bound over the **real corpus** rather than a generated one, because that is the only size at which it binds + `emit::claude` unit pins `a_short_description_keeps_every_title_and_names_no_remainder`, `an_over_long_list_is_truncated_within_the_cap_and_counts_the_remainder`, `a_single_oversized_title_does_not_overflow_the_cap` (the first entry alone exceeding the cap must not overflow — these three keep the word *title* in their names because they build synthetic libraries where the entry is one, and they hold the residual truncation path the real corpus no longer reaches), `the_lead_sentence_says_when_the_skill_applies`, `description_leads_with_when_the_skill_applies`, `description_covers_each_rule_in_the_home` (which now also asserts error classes are *absent*).

**Enforced by (skill emitter):** `emit::claude::emit` (groups by `HomeSlug`, one `OutputFile` per home, rules sorted by tag) + `emit::claude` tests (`one_skill_per_home_layer`, `home_skill_aggregates_its_rules_sorted_by_tag`, `front_matter_names_the_home_slug`, `description_is_quoted_so_a_colon_in_a_home_label_stays_valid_yaml`).

### Cursor rules emitter
What: produces `.cursor/rules/<tag-body>.mdc` (one per rule; filename is the colon-free tag body) with front-matter `description`, `globs`, `alwaysApply`, where `globs`/`alwaysApply` are **derived from `Home`** via the shared `LoadSemantics` helper — not carried on the rule (P2; decision 2026-08-13). A global/project rule is `alwaysApply: true`; a known-language domain auto-attaches on that language's globs; an unknown domain is agent-requested (no globs, not always-on), never blanket-applied.
**Enforced by:** `emit::cursor::emit` + `emit::LoadSemantics::for_home` + the shared domain→glob table; `emit::cursor` tests (`one_mdc_per_rule_named_by_tag_body`, `rust_domain_rule_auto_attaches_on_rs_globs`, `global_rule_always_applies_with_no_globs`, `front_matter_is_delimited_and_ordered`, `emission_is_deterministic`) and `emit` `LoadSemantics` tests (`known_domain_scopes_to_language_globs`, `unknown_domain_is_on_request`, `global_scope_is_always`, `project_scope_is_always`, `an_empty_glob_list_cannot_be_built`, `domain_matching_is_case_insensitive_and_aliased`); the colon-free filename by `RuleTag::body` + `rule::tag` test `body_drops_the_prefix_and_is_a_safe_stem`.

### Copilot instructions emitter
What: produces a single `.github/copilot-instructions.md` holding every rule, ordered by home rank (global → domain → project), then home slug, then tag. An empty library emits no file.
**Enforced by:** `emit::copilot::emit` (+ shared `emit::home_rank`) + `emit::copilot` tests (`single_file_at_the_github_path`, `all_rules_present_ordered_by_home_then_tag`, `empty_library_emits_nothing`, `emission_is_deterministic`) + `emit` test `home_rank_orders_general_before_specific`.

### AGENTS.md emitter
What: produces a single `AGENTS.md` at the repository root holding every rule, same home-rank ordering. An empty library emits no file.
**Enforced by:** `emit::agents::emit` (+ shared `emit::home_rank`) + `emit::agents` tests (`single_file_at_the_repo_root`, `all_rules_present_ordered_by_home_then_tag`, `empty_library_emits_nothing`, `emission_is_deterministic`).

### Claude rules-layer emitter
What: produces **one file per home** at `.claude/rules/<home-slug>.md`, containing only that home's rules, ordered by tag. When there are no rules for the layer it emits no file. The output path is **derived from `Home`** via `HomeSlug` (shape `[a-z0-9-]+` by construction), not a constant — so the emitter cannot name the hand-authored repository-root `CLAUDE.md`, and cannot pool two projects' rules into one file (P2 at the emission layer). *Changed 2026-08-22; the previous constant `CLAUDE.md` path is the collision Paper 3 §8 uses as its worked example — see the ARCHITECTURE decisions log.*

**Which homes reach it is decided by what the target can say** (2026-08-22). This layer has a **two-state** load model — front-matter absent means resident in every session, a `paths:` list means it attaches when a matching file is read — while `LoadSemantics` has three. A known-language domain is emitted **scoped**, with `paths:` carrying that language's globs, so the Rust discipline loads when Rust is touched and costs nothing otherwise. An unknown-language domain is `OnRequest`, which this target has no spelling for, and is **skipped** rather than written unscoped: an unscoped file is not a narrower rule, it is one loaded *always*. `Global` is kept out too — it is `Always`, but already has an always-resident home, and a second copy here would put one rule in two Claude files.
**Enforced by:** `emit::claude_rules::emit` (groups by `HomeSlug`, path built from `RULES_LAYER_DIR` + the slug; the `LoadSemantics` match decides membership) + `emit::LoadSemantics` / `emit::Globs`, whose fallible constructor makes an empty glob list unrepresentable + the property tests `tests/properties.rs::no_emitter_ever_targets_the_repo_root_claude_md` (whole-space: **no** emitter ever produces a file at `CLAUDE.md`), `each_project_layer_file_carries_only_its_own_homes_rules` (whole-space: every rule in a layer file belongs to the home the file is named for, and no `Global` rule reaches the layer), `no_rules_file_ever_carries_an_empty_paths_list` and `unknown_domains_never_reach_the_rules_layer` (both whole-space, added 2026-08-22) + `emit::claude_rules` unit pins (`project_and_domain_homes_each_get_a_file_but_global_does_not`, `a_known_domain_layer_carries_its_language_globs`, `a_project_layer_carries_no_front_matter`, `an_unknown_domain_is_not_emitted_to_this_layer`, `file_is_under_the_owned_dir_named_by_home_slug`, `never_targets_the_repo_root_claude_md` — including an adversarial `Home::project("CLAUDE.md")` and a traversal-shaped path, `each_project_home_gets_its_own_file_carrying_only_its_rules`, `no_rules_for_this_layer_emits_nothing`, `emission_is_deterministic`) + `emit` unit pins (`an_empty_glob_list_cannot_be_built`, `known_domain_scopes_to_language_globs`, `unknown_domain_is_on_request`, `global_scope_is_always`, `project_scope_is_always`) + `fsio::tests::a_hand_authored_root_claude_md_is_never_claimed` (the ownership half: `verify` does not classify the charter at all). **Invocation:** `.github/workflows/ci.yml` runs `cargo run -- verify` bare on both OSes. Verified through the binary on the real seed, **re-run 2026-08-24 at corpus 22**: `relearn build` (default targets) → `wrote 31 file(s)`, exit 0, including `.claude/rules/domain-rust.md` carrying `paths: ["**/*.rs"]`; `relearn verify` (**default arguments, no flags**) → `ok: 31 generated file(s) up to date`, exit 0. *(The counts are dated because they move with the corpus; an undated count here is a claim nothing checks — the reason `README.md` declines to state the corpus size at all. `[R:doc-currency]`)*

### Emission respects rule status — withdrawn guidance never leaks
What: every emitter draws from `emit::emittable(library)`: `active` and `graduated` rules are written; `attic` (retired) rules are **suppressed**, so a withdrawn rule is never written into an active instruction file (skill, `.mdc`, `AGENTS.md`, Copilot, or project `CLAUDE.md`) — not even into its provenance. A `graduated` rule *is* emitted, **annotated** with a `> Also enforced by <destination>.` line under its heading, because the instruction layer tunes generation before the fact whereas the graduated-to control catches after (decision 2026-08-13). The full library (atticked rules included) stays visible to `lint`, which needs retired rules present to flag references to them.
**Enforced by:** `Status::emittability` (the exhaustive-match policy — a new status variant cannot compile until its emit disposition is decided) + `emit::emittable` (applies it once for all five emitters) + `emit::enforcement_note` (the annotation); the property tests `tests/properties.rs::atticked_rules_never_leak_into_any_emitter` (no atticked tag reaches any emitter's `sources`, over generated mixed-status libraries) and `active_and_graduated_rules_all_reach_copilot` (the suppression is exactly `attic`, no over-suppression); unit pins `rule::status::active_and_graduated_emit_but_attic_is_suppressed`, `emit::emittable_keeps_active_and_graduated_and_drops_attic`, `emit::enforcement_note_only_annotates_graduated_rules`, `emit::copilot::atticked_rule_is_excluded_and_graduated_is_annotated`, `emit::claude::a_home_with_only_atticked_rules_produces_no_skill`, `emit::claude::a_graduated_rule_is_annotated_in_its_skill`.

### Emission annotates a rule that has recurred
What: a rule carrying recurrences is emitted with `> Has recurred <n> time(s) since it was written; most recently <date>.` under its heading, in every one of the five formats. Same category, same place, same reason as the graduation note: an assistant reading the corpus should weight a rule that has bitten three times above one written once and never seen again, and the instruction layer is where that weighting happens. The two notes are independent — a graduated rule that has also recurred carries both. The date is the **maximum** of the recurrences, not the last listed, because file order is not required to be chronological. A rule with no recurrences emits exactly what it emitted before the field existed.
**Enforced by:** `emit::recurrence_note` (the one renderer) + the property test `tests/properties.rs::a_recurrence_is_annotated_in_every_emitted_format`, which checks the union of all five emitters' output over generated libraries and **counts** the notes per file (one per recurred source, no more) — a unit test of the renderer proves nothing about whether an emitter calls it, and there are five independent splice sites; unit pins `emit::recurrence_note_only_annotates_rules_that_have_recurred`, `emit::recurrence_note_counts_all_and_dates_the_most_recent`, `emit::graduation_and_recurrence_notes_are_independent`.
### A status can say a rule is only *partly* held
What: `Status::Partial { by, uncovered, date }` records the state in which a rule has real controls over part of its class and prose alone over the rest. It **reports like `Active`** — a recurrence is still an unheld recurrence, because naming a control for half a class does not relieve the prose of the other half — and **annotates like `Graduated`**, emitting `> Partly enforced by <controls>; <uncovered> is held by this instruction alone.` in all five formats. What it never does is claim the whole class: `recurrence-after-graduation`, the one recurrence finding that is an `Error` and fails CI, reads `Status::whole_class_claim`, which is `None` for `Partial`. Recording the truth must never turn a warning into a build failure. `uncovered` is **mandatory**: a partial graduation that will not say what is missing reads exactly like a full one, the reader stops looking, and the uncovered half is protected by nothing while appearing held — `[R:guarantee-needs-a-reader]` wearing a status. Added 2026-09-16; `[R:guarantee-needs-a-reader]` is the first rule to carry it.
**Enforced by:** the type — `uncovered: Uncovered` is a field of the variant, so "partly held, but I will not say what is missing" is unconstructible, and `Uncovered::parse` rejects empty. The two policies are **exhaustive matches** on `Status`, so a future variant cannot compile until both are decided: `Status::prose_coverage` (is a recurrence *unheld*, and is a citation *retired*) and `Status::whole_class_claim` (does a named control claim all of it). Both replaced call-site `matches!(status, Status::Active)` expressions, which would have answered wrongly for a new variant **silently** — the reason the policies exist rather than the predicate living in `lint`. Unit pins `rule::status::partial_requires_the_uncovered_part`, `partial_requires_a_control`, `partial_is_emitted`, `partial_prose_still_holds`, `partial_makes_no_whole_class_claim`, `graduated_claims_the_whole_class`, `active_and_atticked_claim_nothing`, `emit::a_partial_rule_is_annotated_with_both_halves`; integration pins `tests/lint.rs::a_partial_rule_that_recurred_is_still_an_unheld_recurrence`, `a_partial_rule_that_recurred_after_its_date_is_not_a_lying_artefact`, `a_graduated_rule_that_recurred_after_its_date_still_is_one` (the companion — the distinction must be the status, not a weakened finding), `citing_a_partial_rule_is_not_a_retired_reference`.

### Every enum decision that matters is exhaustive
What: the questions code asks about `Status`, `Authority`, `Origin` and `Home` are answered by **policy methods that match exhaustively**, never by a `matches!`, an `if let`, or a wildcard arm at the call site. `Status::emittability`, `Status::prose_coverage`, `Status::whole_class_claim`, `Status::is_withdrawn`, `Authority::provenance`, `Authority::is_editable`, `Origin::recurrence_role`, `Home::federation`. The reason is that **adding an enum variant does not break a `matches!`** — the new variant simply takes the `false` branch, or falls into `other`, and nothing anywhere reports it. Swept 2026-09-16 after `Status::Partial` landed and was found to be silently excluded from one lint finding and silently flagged by another. Nine production decisions were converted; the five that would have answered *permissively* were: a withheld citation reported as merely "not published yet", an unknown authority becoming contributable, withdrawn guidance being cached anyway, an unknown authority joining a retirement poke, and a new `Origin` silently counting as *inert* — that last in the one number whose entire purpose is honesty. A genuine identity test (`VerifyStatus::is_clean`) keeps its `matches!`; the distinction is policy versus identity.
**Enforced by:** each policy is a `match` with no catch-all arm — so the compiler, which is the literal answer for this entry, fails the build at the site that must decide as soon as a variant is added. Verified by **probe** rather than asserted: adding a variant to `Origin` fails compilation at `Origin::recurrence_role`, and to `Authority` at `Authority::provenance` — neither of which would have errored before the sweep. Unit pins `rule::authority::provenance_separates_what_we_wrote_from_what_arrived`, `rule::origin::every_origin_declares_its_role_in_the_statistics`, `rule::status::only_the_attic_is_withdrawn`. **Not fully enforced:** nothing prevents a *future* `matches!` on a domain enum being written; that is a review question, and `TODO.md` carries it.

### A destination is a typed list of controls, parsed at the perimeter
What: `Controls` is a non-empty list of `Control { kind, name, covers }`, parsed once from the destination a rule file writes and rendered back by `Display`. Three rules name **two** controls, joined by `" + "` with a parenthesised note saying which half each covers; that convention was load-bearing and enforced by nothing — `to = "we added a test"` parsed, emitted, and reported no control kind, and `report` re-split the raw string on whitespace, which silently truncated any name containing a space (`gate:verify.sh emoji_ban`). Now a malformed destination **stops the build**, like any other bad field. The on-disk format is deliberately unchanged: the neutral format stays hand-editable and diff-friendly, and what travels between installs is the same text it always was, so no rule file moved and the federation wire format is untouched. Added 2026-09-16.
**Enforced by:** `Controls::parse` is the only constructor and `Status::{graduated, partial}` take a `&str` through it, so an unparsed destination cannot reach a `Status`; the vocabulary is closed by `ControlKind` with no free-text variant, which is also what keeps a control's *name* off the federation's report. `tests/control.rs::every_corpus_destination_round_trips_byte_for_byte` renders all nine real destinations back and requires equality — the whole safety argument for changing the representation without changing the format, and the reason `verify` stayed green on all three output roots **without a rebuild**. Plus `a_name_containing_a_space_survives`, `a_name_may_contain_parentheses_that_are_not_a_note`, `one_bad_entry_fails_the_whole_destination`, and the refusals (`prose_without_a_kind_is_refused`, `an_unknown_kind_is_refused`, `a_kind_naming_no_artefact_is_refused`, `an_empty_coverage_note_is_refused`); `report::an_unknown_control_prefix_is_refused_rather_than_silently_unreported` pins the inverted guarantee.

### A domain enum is never decided by pattern-matching at a call site
What: `tests/exhaustiveness.rs` reads `src/` and fails if any production line decides one of ten domain enums — `Status`, `Home`, `Authority`, `Origin`, `ControlKind` and the five policy enums that answer them — with `matches!`, `if let`, or `let … else`. The 2026-09-16 sweep converted nine such decisions into exhaustive policy methods, so the compiler blocks a *new variant* at each; this blocks a *new call site*, which the compiler cannot see. Two exemptions, both deliberate: **test code** (asserting a specific variant is correct) and **the enum's own module** (whoever adds a variant is editing that file and is looking at the variant list; the danger is the decision taken three modules away). Added 2026-09-16, and it immediately found a tenth — `Rule::is_publishable` decided `Federation` with a `matches!`, safe only by the accident of which variant happened to be tested. Now an exhaustive match, so a third `Federation` variant is a compile error where the publishing decision is actually taken.
**Enforced by:** `tests/exhaustiveness.rs::no_call_site_decides_a_domain_enum_by_pattern_matching`, run by bare `cargo test` — a test rather than a script under `scripts/`, because CI would have to remember to call a script and a test is wired by construction. **Probed, not assumed:** reintroducing the exact original defect (`matches!(rule.status(), Status::Active)` in `lint.rs`) fails it, and so does the `if let Authority::Cached` form in `contribute.rs`. Three of its own preconditions are checked rather than trusted — that it found a source tree at all (`files.len() > 10`, since a gate scanning nothing passes), that no file carries a second `#[cfg(test)]` marker (the truncation would silently stop scanning production code), and that a match is a whole identifier (`VerifyStatus::` contains `Status::`, and the first draft reported `fsio.rs` for it). **Known gap, stated in the test's own header:** a wildcard arm inside a `match` — `_ =>`, or the named `other =>` that a grep for `_ =>` misses — is *not* caught, because knowing what is being matched needs a parser. One such arm existed in `poke.rs` and only the compiler found it. A floor, not a ceiling; carried in TODO.md.


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

### `relearn build --home` / `verify --home` — emit one home layer
What: restricts `build` and `verify` to the rules homed in one layer (its slug). Exists for emitting into a **user** scope such as `~/.claude/rules/`, where the domain layer is wanted and the project layer is not — project homes are `LoadSemantics::Always`, so a project layer written to a user scope would load unscoped in every session in every repository, promoting project rules to machine-wide. A slug no rule is in is an **error** naming the known slugs, never an empty emission: emitting nothing looks like success, and the paired `verify --home` over an empty expected set would pass trivially — a gate reporting success while checking nothing. `build` and `verify` share one `restrict_to_home`, as they share `emit_selected`, so they cannot disagree about which rules are in scope.
**Enforced by:** `cli::restrict_to_home` + `library::Library::<Validated>::filter` (no re-validation: the only cross-rule invariant is tag uniqueness, and a subset of a set with unique tags still has unique tags, so the witness carries over) + `CliError::UnknownHome`; `tests/home_filter.rs` (`restricting_keeps_only_the_named_home`, `the_rules_layer_for_one_domain_emits_exactly_one_file`, `without_the_filter_the_project_layer_is_emitted_as_well` — the control showing the previous test measures the filter and not the fixture, `a_filter_matching_nothing_yields_an_empty_library`, `a_filtered_library_is_still_a_validated_library_for_every_emitter`) + `cli` pins (`build_with_home_emits_only_that_layer`, `build_with_an_unknown_home_is_an_error_naming_the_known_ones` — which also asserts nothing is written on the failure path). **Invocation:** verified through the binary — `relearn build --out <tmp> --targets claude-rules --home domain-rust` → `wrote 1 file(s)`, exit 0, only `.claude/rules/domain-rust.md`; `relearn verify … --home domain-rust` → `ok: 1 generated file(s) up to date`, exit 0; `--home domain-cobol` → `error: no rule is homed in domain-cobol (known: domain-rust, global, project-relearn, project-stochos-lab)`, exit 1.

### `relearn list --home`
What: lists rules as `tag  [home-slug]  title`, optionally filtered to one home layer by its slug.
**Enforced by:** `cli::list` (filters on `HomeSlug::of(rule.home())`) + `cli` test `list_without_a_filter_lists_all_and_a_missing_dir_is_an_error`.

### `relearn lint`
What: reports advisory findings; writes nothing and never deletes. `--deny <warning|error>` sets the lowest severity that makes the run fail — **default `warning`**, so the bare command behaves exactly as it always has. Findings below the threshold are **printed and made non-fatal, never hidden**: suppressing one would be a check whose verdict never reaches the reader, which is the failure `[R:verdict-survives-the-channel]` names. The summary line names the threshold (`1 finding(s), 0 fatal at --deny error`) so a green log says *why* a printed warning did not fail. Exit 0 on a clean library at any level.

*Why the flag exists (2026-09-06).* Every finding until then was a structural defect fixable by editing the library, so "any actionable finding fails the run" was the whole policy. `UnheldRecurrence` is the first that is legitimately long-lived — it says a rule needs promoting to a stronger control, and building that control may be work in **another repository**. Blocking CI on it leaves `main` permanently red, and an always-red check is a muted check (`[R:xplat-fixtures]`).

**Invocation:** `.github/workflows/ci.yml` step *"lint (rule library; errors fail, warnings reported)"* runs `cargo run --quiet -- lint --deny error` on both `ubuntu-latest` and `windows-latest`. Before 2026-09-06 this row named no invocation at all — the check existed and CI never ran it, which is `[R:wired-artifact]`, and it was found by the change that added the first finding capable of failing it.

`--upstream <clone>` adds the federation's pokes, which are printed after the verdict and can never change it — see *The poke* below.

**Enforced by:** `cli::lint_rules` (`load_rules` → `validate` → `lint::lint`, no write path; `ExitCode::FAILURE` only at or above `DenyLevel::threshold()`) + `cli::DenyLevel` (a `ValueEnum` with no `info` variant, so an unselectable level is unrepresentable; an unknown value is a clap parse error before the loader reads anything) + `tests/cli_lint.rs`, which drives the **real binary** via `CARGO_BIN_EXE_relearn` because the exit code is only observable outside the process: `a_clean_library_succeeds_at_every_deny_level`, `a_warning_fails_the_run_by_default`, `deny_warning_is_the_default_spelled_out`, `deny_error_reports_a_warning_without_failing` (asserts the finding is still *printed*), `an_unknown_deny_level_is_rejected_before_anything_is_read`.

### `relearn verify` — the committed generated tree is in sync
What: reads only, writes nothing, and checks that the files `build` would emit from the current rules match what is on disk under `--out`. Each expected file is classified `Ok` / `Missing` / `Unversioned` (a non-generated file shadows the path) / `HandEdited` (body no longer hashes to its own header's recorded `sha256`) / `Stale` (self-consistent but not what the current rules emit); and any marker-bearing file in a relearn-owned location that the current rules no longer emit — its rule or home was deleted — is reported `Orphan`. Exits `FAILURE` if any file drifted, `SUCCESS` (exit 0) when all match — the CI gate that a checkout's generated files were not hand-edited, are not stale, and left no orphan behind. `build` and `verify` emit through the **same** `cli::emit_selected`, so they can never disagree about the expected set. **Ownership policy:** orphan detection scans only the subtrees relearn emits to (`skills/`, `.cursor/rules/`, `.claude/rules/`, and the fixed `AGENTS.md`/`.github/copilot-instructions.md`) and flags only files carrying the generated-by marker, so a hand-authored file — even inside an owned dir — is never claimed. The repository-root `CLAUDE.md` is **not** an owned path: it is the hand-authored charter, and `verify` does not classify it.

**Invocation (this row records it deliberately — see below).** `.github/workflows/ci.yml`, step *"verify (generated tree in sync with rules)"*, running `cargo run --quiet -- verify` — **bare**, no `--targets` and no `--out`, so the command CI takes is the default one. The emitted tree is committed (53 files as of 2026-08-25) so a clean checkout has something to verify, and `.gitattributes` pins those paths to LF so the body hash survives a Windows checkout. Runs on `ubuntu-latest` and `windows-latest`.

> **Correction, 2026-08-22.** This row previously called `verify` "the CI gate" and was wrong on both halves: CI ran only `fmt`/`clippy`/`test`, and `git ls-files` returned nothing under any emitted path, so a clean clone contained no generated files for the gate to check. The artefact was real, correctly named and well tested; its default invocation ran nowhere, on a tree that could not have satisfied it. This is precisely the limit Paper 3 §9 names — *the enforced-by column records enforcement, not invocation* — occurring in the repository the paper uses as its case. The fix is the **Invocation** paragraph above: the column now names the artefact *and* the command that runs it, and that command is the one CI takes. Ledger rows added since should be read with that distinction in mind: naming an artefact is not naming an invocation.
**Enforced by:** `cli::verify` (`load_rules` → `validate` → `emit_selected` → `fsio::verify_all`; `ExitCode::FAILURE` iff any report is not `Ok`) + `fsio::verify_all` / `fsio::classify` / `fsio::parse_generated` (the integrity + freshness classification; `render_with_header` hashes the **body only** so a hand edit is detectable) + `fsio::scan_orphans` / `fsio::collect_files` (the owned-location, marker-scoped orphan pass) + `tests/verify.rs` (`freshly_built_files_verify_clean`, `a_missing_file_is_reported`, `a_hand_edited_body_is_detected`, `an_unversioned_file_at_a_target_path_is_reported`, `a_file_that_no_longer_matches_the_rules_is_stale`, `an_orphan_generated_file_is_reported`, `an_unmarked_file_in_an_owned_dir_is_not_an_orphan`) + `fsio` unit pins (`parse_generated_recovers_the_exact_body_and_hash`, `parse_generated_handles_a_body_without_a_trailing_newline`, `parse_generated_rejects_a_file_without_the_marker`, `verify_reports_ok_for_a_freshly_written_file_and_missing_when_absent`, `an_orphaned_fixed_target_is_reported`, `a_marked_file_outside_owned_locations_is_not_scanned`) + `cli` unit pins (`verify_succeeds_on_a_fresh_build_and_fails_after_drift`, `verify_fails_when_a_generated_file_is_missing`). Verified through the binary on the real seed: clean build → exit 0; hand-edited `AGENTS.md` → `hand-edited`, exit 1; a deleted rule → its `.cursor/rules/*.mdc` reported `orphan` (+ the concatenated files `stale`), exit 1.

### `copilot-pack/` — a downloadable drop-in for another repository
What: a committed second emission of the copilot target at `copilot-pack/.github/copilot-instructions.md`, plus a hand-authored `copilot-pack/README.md` with installation steps. It exists so the folder can be taken out of this repository whole and dropped into an unrelated one, in an environment that has no Rust toolchain and cannot run `relearn build`. The instruction file is **generated**, not transcribed — it is produced by the same `emit::copilot::emit` as the root artefact and carries the same generated-by marker, so a hand edit or a stale copy is detectable rather than merely discouraged. Added 2026-08-24.

**Invocation.** `.github/workflows/ci.yml`, step *"verify (copilot-pack in sync with rules)"*, running `cargo run --quiet -- verify --targets copilot --out copilot-pack` on both OSes. The flags are load-bearing here and not a smell: the pack is a **second output root**, invisible to the bare `verify` above because it lies outside every relearn-owned path — which is exactly why it needs its own invocation rather than trusting the default to reach it. `.gitattributes` pins `copilot-pack/**` to LF for the same reason the root tree is pinned: a CRLF checkout would rewrite the body and the hash would report a hand edit nobody made.

**Known limit, recorded rather than hidden:** `copilot-pack/README.md` is hand-authored and names rule counts, a line count, and the Rust and repository-discipline rule tags by name — and it went stale exactly as predicted on 2026-08-25, when a third rule graduated and its "Two rules describe an enforcement..." paragraph still said two. It went stale again by 2026-09-12, further in every direction at once (46 rules against 52, 538 lines against 746, three graduated against eight, four project rules against seven), and was refreshed by hand that day. **Closed 2026-09-13** by `tests/pack_counts.rs` — see *A pack README's counts are read, in both directions* below: the rule and line counts now fail CI when they drift, in both directions. Two occurrences were what a prose mitigation bought; the third would have been free.

What is **still** hand-authored and unchecked in that README is the prose around the numbers: the named rule tags, the "Two rules describe an enforcement…" paragraph, and the per-home alternatives in the install prompt. Those are sentences rather than counts, and nothing reads them. Carried in TODO.md.
**Enforced by:** `emit::copilot::emit` (same emitter, same determinism and status-filtering property tests as the root artefact) + `fsio::verify_all` reached through the CI step named above. A rule added, changed, or retired without rebuilding the pack fails that step.

### `claude-pack/` — the library as installable Claude Skills
What: a committed second emission of the `claude` target at `claude-pack/skills/<home-slug>/SKILL.md` — one skill per home layer, and **the count of them is not restated here**: it lives in `claude-pack/README.md`, which `tests/pack_counts.rs` reads. This row said *"four skills: `global` 24 rules, `domain-rust` 18…"* until 2026-09-13, by which time there were five and twenty-seven; a count in prose is a claim nothing checks, so the number now has one home and that home is checked. Plus a hand-authored `claude-pack/README.md` with install steps for a Claude Code project (`.claude/skills/`), for every project on a machine (`~/.claude/skills/`), and for claude.ai. It exists so the folder can be taken out of this repository whole and installed somewhere with no Rust toolchain. The skills are **generated**, not transcribed — same `emit::claude::emit` as the root artefact, same generated-by marker, so a hand edit or a stale copy is detectable rather than merely discouraged. Added 2026-09-06.

**Invocation.** `.github/workflows/ci.yml`, step *"verify (claude-pack in sync with rules)"*, running `cargo run --quiet -- verify --targets claude --out claude-pack` on both OSes — a second output root, invisible to the bare `verify`, so it needs its own invocation for exactly the reason `copilot-pack` does. `.gitattributes` pins `claude-pack/**` to LF, because `verify` hashes the body and a CRLF checkout would report a hand edit nobody made (`[R:pin-eol-for-executable-text]`).

**Prerequisite, and why this row could not have existed before today:** a skill is only installable if its `description` fits the cap — see *"A skill `description` is a bounded trigger"* above. Until 2026-09-06 the two skills anyone would actually want were 6.8× and 4.2× over it, so a pack built from them would have been a folder of files that could not be installed.

**Known limit, recorded rather than hidden:** `claude-pack/README.md` is hand-authored and names per-home rule counts and per-skill line counts. Nothing checks them against the library, so they can go stale without failing CI — the same exposure already recorded for `copilot-pack/README.md` and carried in TODO.md. The counts were verified two independent ways when written (`relearn list --home <slug>` and the `##` heading count of each emitted skill), which is evidence about the day they were written and nothing later; they went stale by 2026-09-12 (a fourth home added, `global` 24 against 27) and were refreshed by hand that day, which is the same non-fix the copilot row records. Each number now names the command that measures the real one.

**Enforced by:** `emit::claude::emit` (same emitter, same determinism and status-filtering property tests as the root artefact) + `fsio::verify_all` reached through the CI step named above.

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

### The counter-metric ships: `origin` and the two-number tally
What: every rule declares `origin = "mined" | "codified"` — produced by the error loop, or written down from standing practice. `relearn lint` always prints `N rule(s): X recurred, Y inert (codified and never fired)`, never the recurrence count alone. **P5: the metric does not ship without its counter.** Recurrence is only ever incremented by someone willing to record that their own rule failed, so it falls through under-reporting exactly as easily as through prevention; *inert* — never recurred **and** never mined — is the fraction that says the library is full of rules that change nothing. Corpus at 2026-09-06: **46 rules, 1 recurred, 19 inert** — a dated snapshot, not a live figure; `relearn lint` prints the current one on every run, which is why it is the only place this repository keeps it.

The field is **mandatory**, not defaulted: a default classifies unlabelled rules silently, which is the under-reporting the counter exists to detect, and a metric computed from guesses is worse than none because it looks like a measurement. All 46 files were migrated in one pass, classified by **the author's own signal** — the 19 whose `incident` says "Codification-dated" — never by the migrator's judgement. (The tempting second signal, the word "ported" in 32 rules, was measured and rejected: it describes migration between homes, not origin, and would have misclassified about thirteen.)
**Enforced by:** `rule::Origin` (a two-variant enum; an unknown value is `ParseError::Origin` naming the value and the alternatives) + `Rule::new`'s required argument + `Rule::is_inert` + `lint::tally` / `lint::Tally` (private fields, accessors) + `tests/corpus.rs::every_committed_rule_declares_an_origin`, which also asserts the literal `origin = ` line is present so a future `#[serde(default)]` added for convenience fails here rather than quietly filling it in + `tests/lint.rs::the_tally_counts_recurred_and_inert_separately` (a mined rule that never recurred is **not** inert — conflating the two is the whole error the number exists to prevent) + `rule::origin` unit pins.

### A recurrence after graduation is an error
What: `Status::Graduated` carries a `date`, and a recurrence strictly **after** it raises `Finding::RecurrenceAfterGraduation` at `Severity::Error` — the one recurrence finding that fails CI. The named stronger control was claimed to hold the rule and demonstrably did not, so every emitted layer's `> Also enforced by {to}` line is false: a lying artefact in a live artefact (`[R:repair-the-lying-artefact]`), not a rule merely wanting promotion. A recurrence **before** the graduation is deliberately not a finding — it is very often the incident that prompted the graduation, and flagging it would punish exactly the response the library wants.
**Enforced by:** `rule::Status::Graduated { to, date }` (the date is required; a graduated status without one does not parse) + `lint::recurrence_after_graduation` + `tests/lint.rs::a_recurrence_after_graduation_is_an_error` — which asserts the finding is raised **exactly once**, a pin added 2026-09-13 after `lint` was found to have called this check twice since 2026-08-16 and pushed both results, printing every such finding twice. Invisible: this corpus holds no graduated rule that has recurred, and every test asked only whether the finding was *present*. Probed both ways — restoring the duplicate call fails the test naming both copies — plus `a_recurrence_before_graduation_is_not_a_finding`, and `a_recurrence_on_the_graduation_date_is_not_after_it` — the boundary is pinned because an off-by-one manufactures an `Error` out of the day the control landed. The three graduated rules were dated from git history (each hook's first commit, 2026-07-21), not from a plausible guess.

### Unheld-recurrence detection — a rule prose is demonstrably failing to hold
What: an `Active` rule — one held by prose alone — that records at least one recurrence is flagged `Warning`. The message states what the finding *means*, not what it found: prose is the only thing holding this rule, and prose has already been shown to fail, so promote it to a control that can hold it and record that with `Status::Graduated`. **No count threshold** — the first recurrence already proves the point, two is not more actionable than one, and a cut-off would be a magic number this codebase does not use. `Warning`, not `Error`: `Error` here means the *emitted tree* would be wrong (two rules colliding on one file); this is a fault in the **library**, and a build that emits correctly must not fail on it. A *graduated* rule that has recurred is deliberately **not** this finding — that is the sharper one (a named stronger control that demonstrably did not hold) and it needs a graduation **date** that `Status::Graduated { to }` does not carry; separate change, own argument, carried in TODO.md.
**Enforced by:** `lint::unheld_recurrences` + `tests/lint.rs::an_active_rule_that_has_recurred_is_flagged`, `one_recurrence_is_enough_and_the_latest_date_is_the_maximum`, `an_unheld_recurrence_is_a_warning_not_an_error`, `a_rule_that_has_never_recurred_is_not_flagged`, `a_graduated_rule_that_has_recurred_is_not_this_finding`.

### Scope near-duplicate detection — a vocabulary without a curator

What: a scope declared by exactly **one** rule, sitting a slip away from one declared by **more**, is flagged `Warning` naming both spellings and the rule to open. `applies_to` is free text bounded only in shape, so the set of audiences in use is whatever the corpus declares — one identity space rather than a controlled list somebody owns. `ScopeTag` kills the malformed cases at parse time; what survives is the well-formed near-miss, and that one is **silent**: nothing fails, the rule simply serves an audience nobody asks for.

**Enforced by:** `lint::scope_near_duplicates` + `lint::Finding::ScopeNearDuplicate` + `tests/lint.rs::a_scope_used_once_near_one_used_by_many_is_flagged`, `a_one_character_slip_in_a_long_audience_is_flagged`, `two_real_audiences_two_edits_apart_are_not_flagged`, `short_audiences_are_not_near_misses_of_each_other`, `two_scopes_each_used_once_are_not_a_near_duplicate`, `a_scope_near_duplicate_is_a_warning`.

**The design says edit distance 2, and that alone is wrong in a way its own examples show.** `rust` and `ruby` are two edits apart and are two languages; `go` and `js` are two apart because **any** two two-letter scopes are, by arithmetic rather than by error. A check firing on those is the false positive that gets the whole thing muted, which is the outcome §12.5 was written to avoid. So distance is read **relative to length** — two edits require at least five characters, one edit at least three — which admits `low-latencv` against `low-latency` and refuses both pairs above. The ceiling is the design's; the floor is what it needed and did not state, recorded in §12.5 itself.

**Built only once it could fail.** It was deferred through every federation phase because the corpus declared no scopes at all, and a drift detector over an empty vocabulary arrives already green — an artefact that cannot fail is one nobody notices is broken. Two low-latency rules gave it a vocabulary on 2026-09-14.

**Observed through the binary (2026-09-14)** over an eleven-rule library with three real audiences, two typos and two short names: `rusty` and `low-latencv` both flagged with the rule to open; `ruby` against `rust` and `js` against `go` both silent. Against the real corpus it is silent, correctly — two scoped rules cannot produce a one-against-many.

**Levenshtein is twenty lines here, not a dependency.** A crate would need pricing in the decisions log (`[R:price-every-dependency]`) for a function whose behaviour is fully specified by four test cases.

### Contradiction detection
What: two rules that semantically contradict each other.
**Enforced by: NOTHING YET — exposed, and held meanwhile by a process-control** rather than a code artifact: a periodic *Claude review pass* over the corpus (the rule bodies are readable markdown; Claude judges consistency, delegated through the subscription per doctrine). Deliberately **not** a keyword heuristic in the linter: a heuristic dressed up as certainty would be worse than the honest absence. *Review 2026-08-24 (corpus 22, after `[R:repair-the-lying-artefact]` landed; previous pass 2026-08-22 at 19): **clean.** Contradicts nothing. Three near neighbours, all separable — `[R:verify-through-production-path]` is the *verifying* half to this one's *reporting* half (exercise the real channel vs. that channel must tell the truth about what it produced); `[R:prefer-by-construction]` supplies the preferred fix shape (derive the claim, do not assert it) but is not about false claims; `[R:doc-currency]` is the closest and is **complementary, not overlapping** — it says update every checked-in description when reality moves, this one says the description is not where the fix lands when an executable artefact is the thing doing the misleading. The two rules that landed after the previous pass (`xplat-fixtures`, `pin-eol-for-executable-text`) were inside this pass' scope and are clean. Recorded in TODO.md, Seed content.* *Review 2026-08-22 (corpus 19, after the `rust-typedd` port; previous pass 2026-08-20 at 15): **clean.** No contradictions, no problematic overlap, homes consistent under the homing principle settled that day (TODO.md, Seed content). Four relationships noted, none a contradiction — `newtype-liberally`↔`parse-dont-validate`, `typestate-for-protocols`↔`make-illegal-states-unrepresentable`, `design-types-first`↔`prefer-by-construction`, and `no-weak-model-for-judgment`↔`measure-cost-per-task` (a capability floor beside an economic rule; each body names the other and states the distinction, so neither reads as a restatement).* *Review 2026-08-13 (re-run after the corpus grew to fourteen rules): no contradictions, no problematic overlap, homes consistent (recorded in TODO.md). One relationship noted, not a contradiction: `R:prefer-by-construction` (design a mistake out rather than guard against it) and `R:generate-guards-unversioned` (a runtime guard protecting hand-authored files) are the hierarchy-of-controls in action — the guard is the honest fallback exactly where by-construction is unavailable, since "a human authored this file" cannot be made unrepresentable at the filesystem level. This row is `exposed` by design — there is no deterministic code artifact to name, and inventing one would be the dishonesty the tool exists to prevent.*

### Competing-home detection — a rule that also lives outside the library
What: a rule stated both in `rules/` and in a hand-authored instruction file elsewhere (a skill, a personal `CLAUDE.md`) has two homes, which is the P2 violation this tool exists to prevent.
**Enforced by: NOTHING YET — exposed.** `lint::lint` takes `&Library<Validated>` and reads only the rule library, so anything outside it is structurally invisible. Found 2026-08-22: four rules — `parse-dont-validate`, `parse-wide-then-range-check`, `no-anyhow-in-libraries`, `no-unwrap-in-production` — were stated both here and in `~/.claude/skills/rust-typedd/SKILL.md`, and the linter reported a clean library throughout. They were found by reading. Not closed in that change because the fix requires deciding whether `lint` may read anything outside the rule library, which cuts against the standing invariant that the linter and emitters are pure functions of validated rules; a half-measure (scan a configured list of foreign paths for `[R:...]` tags) would report presence, not duplication, and would flag every legitimate *citation* of a rule as a competing home. Carried in TODO.md.

### Cold-surface / uncited report
What: rules that nothing exercises (candidates for the attic cut-list).
**Enforced by: NOTHING YET — relocated to Phase C** *(not a Phase B gap: "cold" needs runtime invocation data, which lives in the stochos-lab observability layers — Phase C — not in this repo's rule text. Reporting "uncited" alone would be noise, since a standalone rule is legitimately uncited. Tracked under Phase C in TODO.md until that data feed exists; not buildable here without faking a signal.)*

---

## Repository hygiene

### A private name cannot be published in a quotation

What: this repository stores other repositories' incidents **verbatim** — a rule's
`incident` field, and TODO.md's narration of the same material — and it is public. A
name that is private in the source repository is therefore one quotation away from being
published, and the source's own detector cannot help, because a detector scans its own
tree. `scripts/no-banned-names.sh` walks this tree and reports **location and count**,
never the term, and exits **2 — not 0 — when it has no list**, so a disarmed gate is
distinguishable from a clean one.

**Enforced by:** `.githooks/pre-push` → `scripts/no-banned-names.sh`, invoked by
`git push` once a clone has run `git config core.hooksPath .githooks`. Probed in all four
directions on 2026-09-06 before being trusted: clean tree → 0; a probe list banning a
word this repository is full of → 1, with counts and paths and a `<file name> in <dir>/`
line where the match was the filename itself; no list → 2; a list with a salt and no
terms → 2.

**CONFIGURATION-DEPENDENT, and recorded as such rather than claimed as a CI gate.** The
term list is **not** committed here and cannot be. In the source repository (private) a
committed list of salted digests is correct and means a fresh clone can never arrive
disarmed. This repository is public, and one protected name normalises to four
characters — about 1.7 million candidates against a published salt — so committing the
list would disclose what it detects, which is `[R:detector-excludes-own-definitions]`
pointing the other way. The list lives at `$BANNED_TERMS_FILE`, or
`~/.claude/usage/banned-terms.sha256`. Consequently CI **cannot** run this gate and is
not wired to it: a fork has no list, and failing every fork for a reason that is none of
its business is worse than the gap. The residual exposure is that the gate is armed only
where a maintainer installed the hook, and TODO.md carries it.

**The rule it holds:** `[R:names-travel-with-the-quote]`, mined from this repository on
2026-09-06 — an employer-owned product name found in `rules/repair-the-lying-artefact.md`
and twice in `TODO.md`, present in all 52 commit trees and in TODO.md since the first
commit. Redacted in the same change; the history rewrite is carried in TODO.md.

### Every dependency is priced in the decisions log, and so is every refusal

What: `[R:price-every-dependency]`, codified 2026-09-12 — a dependency is a decision that arrives as one line in a manifest, so it gets an entry in the ARCHITECTURE decisions log (what it is for, why this one, what was refused — including writing it by hand — whether the default features were taken, whether it is dev-only), and a decision *not* to take one gets an entry too. The manifest comment points at the entry; it does not restate it.

**Enforced by:** `tests/dependencies.rs::every_dependency_is_priced_in_the_decisions_log` — it walks every `Cargo.toml` in the repository (`target/` excluded), collects runtime, dev, build and per-target dependency keys, and fails naming any that no decisions-log row prices. Three further tests hold the check itself upright: `no_grandfathered_dependency_is_already_priced` and `no_grandfathered_dependency_has_left_the_manifest` force the exemption list to shrink and never rot, and `the_decisions_log_is_where_this_gate_thinks_it_is` fails if the log section is renamed or emptied — without it, the priced set would go quietly to zero while every dependency was still grandfathered, and the gate would pass while measuring nothing (`[R:guarantee-needs-a-reader]`).

**Invocation.** `cargo test`, which `.github/workflows/ci.yml` already runs on `ubuntu-latest` and `windows-latest`. A test rather than a shell script deliberately: it is wired by construction and cannot arrive disarmed the way a script needing an installed hook or a remembered CI step can. `scripts/verify-dependencies.sh` is a hand-invocable front door to the same test — `0` passed, `101` failed, `2` disarmed because cargo is absent, each observed rather than assumed — and it carries **no check of its own**: it explains the marker format, forwards its arguments, and does not filter cargo's output. The enforcement is the test; the script is a door onto it, and a second implementation there would be a second home for the rule.

**What it proves, and what it does not.** It proves a dependency is *named* by a row that declares itself to be about dependencies — a row whose **Decision** cell opens with `Dependency:` or `Dependencies:`, the crate in backticks. It cannot read the row and judge whether the price is real: whether the rejected list is honest, whether the default features were weighed, whether writing the thing by hand was considered. The marker must open the cell rather than appear anywhere in the row, so that a row *about* the convention cannot price a crate by mentioning it — `[R:detector-excludes-own-definitions]`, which this check would have failed on its first day, and which the 2026-09-12 decisions entry now exercises: that row backticks three crate names and prices none of them.

**Probed in eight directions on 2026-09-12 before being trusted:** clean tree → pass; a new unpriced dependency → fail, naming it; the same dependency priced → pass; a marker row placed outside the log section → fail; a crate named in a non-marker row → fail; a grandfathered crate given an entry → the shrink test fails; a grandfathered crate removed from the manifest → the stale test fails; the log heading renamed → the locator test fails.

**The other half is held by nothing, and no gate can hold it.** A dependency *refused* leaves no manifest line, no lockfile churn, no artifact of any kind, so there is nothing for a check to compare the log against — the reason that half is the one that goes missing. It is P4: the human reader is the only detector. The rule therefore stays `active` rather than graduating to this test, because `Status::Graduated { to }` is single-destination and the emitted *"Also enforced by"* line would be a false claim about the refused half. The same argument keeps `[R:verdict-survives-the-channel]` honest; TODO.md carries both.

**Measured when the rule was written (this repository, 2026-09-12):** nine dependencies — six runtime (`toml`, `serde`, `clap`, `thiserror`, `anyhow`, `sha2`) and three dev (`proptest`, `tempfile`, `trybuild`) — against a decisions log in which three of the nine names occur at all and exactly one, `toml`, occurs as a crate choice, and that entry prices a *format* decision for which the crate's maintenance is the stated reason. No dev-dependency appears anywhere in it. The existing entries were deliberately **not** retrofitted: reconstructing a rejected list months later produces the flattering one, which is the failure `[R:decisions-log-records-rejected-alternatives]` already names.

### The aggregate applies the k-floor, prints its confounds, and is never authoritative

What: `relearn aggregate --clone <path> --generated YYYY-MM` recomputes `aggregate.toml` from the reports in a cloned aggregate repository. **A repository, not a service** — reports arrive as pull requests, a scheduled job runs this over them and commits the result. That is the entire infrastructure, and it is why publication is deliberate by construction, why every byte that ever crossed is auditable in public history, and why review is free.

**The k-floor is applied here, and below it a rule does not appear at all — tag included.** Publishing `R:x` while withholding its count points at the same person as publishing the count. `K_ANONYMITY_FLOOR` is re-exported from `report`, so the producer and the aggregate read **one constant** rather than two policies that could drift. The floor counts **distinct installs**, not reports: one install repeating a report is the obvious way to fake a population.

**Enforced by:** `tests/aggregate.rs::below_the_floor_a_rule_does_not_appear_at_all` (asserts the tag is absent from the rendered document at every n from 1 to k−1, not merely that the row is missing) + `at_the_floor_the_rule_publishes` (so the test above measures the floor rather than a renderer that never works) + `one_install_cannot_reach_the_floor_by_repetition` + `the_suppressed_count_is_published_but_never_the_tags` — a reader is told **how much** was withheld so an aggregate cannot read as complete, and never **what**.

**Both confounds print beside the numbers, unconditionally.** `CONFOUNDS` is a `const [&str; 2]` written by `to_toml` with no condition attached: cross-install recurrence measures frequency *and* diligence inseparably, and since the catalogue was dropped the aggregate counts only classes somebody published a rule for — a bias toward cheap incidents, not mere sparseness. **Enforced by:** `both_confounds_print_beside_the_numbers` and `an_empty_aggregate_still_carries_its_confounds`, the second because an empty document is where dropping them is most tempting and most misleading. The way they get dropped is by being droppable, so they are not a flag.

**The aggregate is never authoritative.** No build behaves differently for having seen one. **Enforced by:** `no_emitter_or_build_path_reads_the_aggregate`, which reads the source of all five emitters plus `emit.rs`, `library.rs` and `fsio.rs` for `crate::aggregate`, `aggregate::` and `Aggregate`, comments stripped. **Probed both ways (2026-09-13):** adding `use crate::aggregate::Aggregate;` to an emitter makes it fail naming the file and the marker; removing it makes it pass. Its first version matched the bare word `aggregate` and failed on a test named `home_skill_aggregates_its_rules_sorted_by_tag` — a detector that cannot tell an English word from a dependency gets muted (`[R:detector-excludes-own-definitions]`), so it now matches what a dependency actually looks like.

**Reports come from strangers, so parsing is strict.** An unknown schema version, an unknown bucket spelling and a malformed pseudonym are each refused rather than skipped: an aggregate that ignored what it could not parse would publish a count quietly missing whoever wrote it. **Enforced by:** `an_unknown_schema_version_is_refused`, `a_malformed_install_id_is_refused`, and `an_unknown_bucket_is_refused`.

**Observed through the binary (2026-09-13):** six reports in a clone, five naming one rule and one naming another → the five-install rule published with `installs = 5` and its bucket distribution, `suppressed = 1`, and **the suppressed tag absent from the whole document**; both confound sentences at the head of the file; `installs = 6` distinct.

**What it does not do.** It does not fetch — there is no network path in this tool and `tests/solo_mode.rs` is what keeps the "for convenience" version from appearing. It does not schedule itself: the job belongs to the aggregate repository's CI, which does not exist here, and this row claims a command rather than a cron entry.

### The ledger's own enforcing artefacts are read

What: this file makes seventy `Enforced by:` claims, and until 2026-09-13 **nothing checked that any of them named something real**. A test renamed in a refactor leaves its row behind, still naming it, still reading as a guarantee somebody holds — `[R:guarantee-needs-a-reader]` and `[R:wired-artifact]` at once: a claim with no reader, and a claim whose evidence is a string anybody can typo.

**Two directions, each owning its denominator.** Every **path-qualified citation** — one naming its file and its function, as the row below does — must resolve: the file exists and defines that function, with line comments stripped first so a test named only in a comment cannot satisfy a claim that it *exists*. And every **`Enforced by:` row** must name something in backticks within sixty characters of the marker, or say `NOTHING YET` — so a row cannot claim enforcement in prose alone.

*(This paragraph carried an invented example of the citation form until the gate read it as a citation and failed on the missing file. An illustration of the form is indistinguishable from a use of it, so the row now points at a real one instead — `[R:detector-excludes-own-definitions]`, caught by the artefact it was being written for, on its first full run.)*

**Enforced by:** `tests/ledger.rs::every_artefact_the_ledger_names_by_path_exists` and `every_enforced_by_row_names_an_artefact_or_declares_itself_exposed`. **Invocation:** `cargo test`, on both OSes in CI.

**Scope, stated because it is narrower than it looks (`[R:measure-the-claim-not-a-subset]`).** Only the path-qualified form is resolved — thirty-two citations across twelve test files. Bare `module::item` citations (`lint::tally`, `Rule::serves`) are **not**: resolving them means searching the tree, and the same shape is used for `env::var` and `ExitCode::FAILURE`, which are the standard library's. A checker carrying an exclusion list of foreign names grows one entry per release until somebody mutes it. The path-qualified form carries its own file, so there is nothing to search and nothing to exclude — and it is the form the load-bearing citations use.

**Both floors are deliberate**: fewer than twenty citations, or fewer than twenty rows, fails rather than passes. A parser that has stopped recognising the ledger would otherwise report success having measured nothing. The floors sit well below the real counts so that adding rows never requires editing this check — a check that must be edited to stay true is one that gets edited away.

**Probed three ways (2026-09-13), and the second version of one direction was the honest one.** Renaming a cited test → fails naming the row, the file and the function. Pointing a citation at `tests/linter.rs` → fails naming the missing file. Replacing a citation with *"careful review at commit time"* → **passed at first**, because the first version asked only whether a backtick appeared anywhere after the marker, and every row satisfies that by accident in a long paragraph. Measuring the real ledger fixed it: fifty-eight rows open with their citation and the rest reach it within thirty characters, so a sixty-character window catches the prose claim while rewording no honest row.

**Found by building it:** the contradiction-detection row claimed enforcement by *"a **process-control**, not a code artifact"* — the one row naming neither artefact nor gap, while this file's own header had said since 2026-08-13 that it *"stays `exposed` by design"*. It now says `NOTHING YET — exposed, and held meanwhile by a process-control`, which is what the header always claimed it said.

### A pack README's counts are read, in both directions

What: `copilot-pack/README.md` states a rule count and a line count for the instruction file it ships; `claude-pack/README.md` states a rule count for each skill layer. Those numbers are hand-written about generated files, and until 2026-09-13 **nothing read them** — the federation programme named it as the standing exposure twice, and it is `[R:guarantee-needs-a-reader]` in the repository whose own corpus carries that rule. *(The values are deliberately not repeated in this row. They have one home, and it is the one the test reads; restating them here would be a second copy that nothing checks — which is how the row above came to say "four skills, 24 rules" for a week.)*

**Two directions, because one of them misses what actually drifts.** Every **claim** must resolve to a file in its pack and match what that file holds. And every **file** in a pack must be the subject of at least one claim — the direction that catches a sixth home layer arriving, `build` writing its `SKILL.md`, `verify` staying green because the pack matches the rules, and the README still listing five. Every claim would still be correct; the inventory would be a lie by omission.

**Enforced by:** `tests/pack_counts.rs::every_count_a_pack_readme_claims_is_the_one_its_file_holds` and `every_file_in_a_pack_is_counted_by_its_readme`. **Invocation:** `cargo test`, which CI runs on `ubuntu-latest` and `windows-latest` — a test rather than a script, for the same reason as the dependency and solo-mode gates: it is wired by construction and cannot arrive disarmed.

**Scope, stated because `[R:measure-the-claim-not-a-subset]` is exactly what goes wrong here.** A *rule* is a `## ` heading ending in its `[R:tag]` — the form every emitter writes, which no prose heading can accidentally match; a *line* is `str::lines()`, which strips a trailing `\r` so the count is identical on a CRLF checkout (`[R:xplat-fixtures]`). A README stating no counts at all **fails**, because a parser that has stopped recognising the inventory would otherwise pass silently. File paths are resolved by suffix match rather than by parsing the ASCII tree: the tree is decoration, and a check that read it would break on a redrawn box character while the numbers it guards stayed wrong.

**What it does not check:** that a pack matches the rules. `relearn verify --targets copilot --out copilot-pack` holds that and CI runs it as a second output root. The two compose — README ↔ pack ↔ rules — and **each owns its own denominator**, so neither needs to know how the other measures.

**Probed both ways (2026-09-13):** raising the copilot README's line count by one fails naming the file, the line and the true count; deleting the `project-design-architecture-tool` line from the claude README leaves every remaining claim correct and fails the *second* test naming the uncounted file. Restored, both green.

**And it caught a real one the same day, in the next commit.** Recording a recurrence on `[R:doc-currency]` added two lines to `copilot-instructions.md`; the README still claimed the old count, and this test failed with the file, the line and both numbers. That is the whole point of it — the drift it caught was introduced by the change that shipped beside it, which is exactly how the two occurrences before the gate happened.

### The poke: one reactive trigger on, broadcast capped, and it can never fail a run

What: `relearn lint --upstream <clone>` prints what the corpus knows that this install might want to. Five triggers — §6's four, and §12.6's upstream retirement: **class-covered** (a rule fired here and upstream already has one for that class — *reactive*), **cache-behind** (a cached rule whose upstream revision has moved), **cache-retired** (a rule this install holds a copy of, retired upstream), **contributed** (an upstream rule serving this install's audience that it does not hold), **high-recurrence** (a rule many installs report and this one does not hold). Without `--upstream` there are no pokes and **no warning about their absence** — a solo install is the product, and a nag is a requirement with better manners.

**The reactive one is what makes this a collective memory rather than a mailing list**, and it is the only one that is not capped: it follows evidence recorded *here*, at the moment somebody has just demonstrated they needed it. Everything else is broadcast, which is how a notification channel teaches people to ignore it — Paper 3 §9 already documents the failure on exposed rows. So broadcast is capped per run, the cap is a number the operator passes (`--poke-cap`, default 3) rather than a judgement buried in code, and two of the four broadcast triggers are off until named — the two that are on are both about a copy this install already holds, so neither can fire against an install that caches nothing. **Enforced by:** `poke::Trigger::on_by_default` — the design's table in one exhaustive match and nowhere else — plus `poke::Reach`, which makes "which pokes does the cap reach" a property of the trigger rather than a `bool` at the cap's call site + `tests/poke.rs::only_the_default_triggers_fire_when_none_is_named`, which supplies data for **every** trigger and asserts the off-by-default ones stay silent anyway (then asserts all five fire when named, so the silence is a decision rather than an absence of data) + `the_cap_withholds_broadcast_pokes_and_publishes_how_many` + `the_cap_never_withholds_a_reactive_poke` + `pokes_are_ordered_by_rank_then_tag` (`[R:order-by-explicit-rank]`: the `Trigger` declaration order is the rank, never a sort of the rendered text).

**An upstream retirement warns and can never do more than warn (§12.6).** A local attic suppresses emission; an upstream one must not, because deleting an instruction a team relies on *because a stranger retired it* is a correction lost with no reader — P1, the exact failure this tool exists to prevent, and the local install may hold evidence the upstream author does not. The poke names the three human resolutions — pull, adopt, drop — and `adopt` first among them, since a retirement installs refuse is the population telling an author something no single install can know. **Enforced by:** the fact that only a **local** `Status` reaches `Status::emittability`, so the suppression is unreachable rather than declined + `tests/poke.rs::an_upstream_retirement_never_suppresses_the_local_rule`, which asserts it over the **real emitters** rather than over the argument + `a_rule_you_own_is_not_retired_by_a_stranger` (a rule with no upstream cannot be retired by one) + `an_upstream_graduation_is_not_a_retirement` (a graduated rule is still emitted; only tag-level death is an attic, which is why §12.6 needed no new `Status` variant) + `a_rule_retired_upstream_pokes_and_says_when_and_why`.

**Its shape is a decision, not a transcription.** §12.6 specifies a lint `Warning`, written before the poke existed. A `Warning` that can fire only when `--upstream` is given is federation failing a run at the default `--deny warning` — which the single-install guarantee forbids — so it ships as the fifth *poke* instead, on by default, capped like every broadcast. Recorded in the decisions log and in `docs/federated-relearn.md` §12.6 itself, so the design does not go on specifying a shape the code deliberately does not have.

**A poke is news, not a finding, and it can never change a verdict.** It carries no severity, `--deny` does not reach it, and it is printed after the exit code has been decided. A federated signal that can fail CI has made federation required, which the single-install guarantee forbids outright. **Enforced by:** `tests/poke.rs::a_poke_reaches_the_reader_and_never_changes_the_exit_code`, which runs the **real binary** twice — with and without a clone — and asserts the two exit codes are equal and that the poke text reached stdout (`[R:verify-through-production-path]`; the exit code is only observable outside the process) + `a_failing_lint_still_fails_when_it_is_poked`, the other half, so the first is not satisfied by a lint that never fails + `without_a_clone_nothing_is_poked_and_nothing_is_said_about_it`.

**`lint` reports; `build` emits.** No build behaves differently for having been poked. **Enforced by:** `tests/poke.rs::no_emitter_or_build_path_reads_the_poke`, the same source-level scan as the aggregate's, over all five emitters plus `emit.rs`, `library.rs` and `fsio.rs`, comments stripped (`[R:detector-excludes-own-definitions]`). The flag exists only on `lint`, so there is no `build --upstream` to scan for in the first place.

**Withheld pokes publish their count**, for the same reason the aggregate publishes its suppressed count: a throttled run must not read as a quiet one. And the k-floor reaches the poke by construction — the population trigger reads the aggregate's **rows**, so a rule below the floor is invisible to it with no second policy to keep in step. "High recurrence" is the published top bucket (`Bucket::Many`), not a tuned integer. **Enforced by:** `a_rule_below_the_floor_is_invisible_to_the_poke` (asserted at k−1 *and* at k, so it measures the floor rather than a trigger that never fires) + `high_recurrence_means_the_top_bucket`.

**A poke flag with no clone is refused** rather than silently inert, and a mistyped trigger fails **before** the lint runs rather than after it has printed. **Enforced by:** `CliError::PokeWithoutUpstream`, `CliError::PokeTrigger` + `tests/poke.rs::a_poke_flag_without_a_clone_is_refused` + `poke::tests::an_unknown_trigger_lists_the_ones_that_exist`.

**Observed through the binary against the real 52-rule corpus (2026-09-13):** a hand-built clone holding an upstream rule with the error class of `R:verdict-survives-the-channel` → `poke [reactive]: R:verdict-survives-the-channel has fired 1 time(s) here, and upstream already has R:read-the-verdict for the same class`, with the lint's own exit code (1, from two unheld recurrences) unchanged. With the upstream rule carrying the *same tag* as the local one, nothing is poked — holding it is the answer the poke would give. `--poke-cap 1` with all four triggers → one reactive, one broadcast, `2 more broadcast poke(s) withheld by --poke-cap 1`. `--poke reactive` → `error: --poke: unknown poke trigger "reactive" (expected one of: class-covered, cache-behind, contributed, high-recurrence)`, before any finding is printed.

**The fifth trigger observed the same way (2026-09-13):** a rule in the clone retired with `status = { kind = "attic", reason = "cold surface, challenge-tested", date = "2026-09-02" }` and a local cache of that tag added to the real corpus → `poke [broadcast]: upstream retired R:an-upstream-only-rule in 2026-09-02 (cold surface, challenge-tested) — your copy still emits, and will keep emitting: pull the retirement, `adopt` it if you hold evidence they do not, or drop it`, printed beside the reactive poke under the **default** trigger set, with the exit code unchanged.

**What it does not do.** It does not fetch: the clone is a path on the command line, read with the same `load_rules` the local corpus uses, and the recurrence signal is **recomputed** from `reports/*.toml` rather than read from a committed `aggregate.toml` — nothing verifies that a published aggregate matches the reports beside it, and a recompute needs no second parser to drift from. It installs **no `SessionStart` hook**: the snippet lives in `docs/session-start-poke.md` and stays there, because anything under `~/.claude` edits the layer loaded into every session on the machine. And "new to you" means "absent here" and nothing else — there is no record of what an install has already been shown, because keeping one would be per-machine state.

### A published rule states which revision it is, in the field that already means that

What: `authority = { kind = "local", version = N }`. A cache records which revision it holds; a home now records which revision it **is** — the same field answering the same question, so a cached file never carries the number twice. It is what makes "is this cache stale?" answerable at all: without a revision upstream there is no comparison point, and the poke says nothing rather than guessing. An unnumbered local rule renders **no authority line at all**, which is why none of the 52 committed rules needed editing.

**Enforced by:** `Authority::Local { version: Option<Version> }` — `None` is a real state (a rule nobody has published has no revision) rather than a zero that would tell every cache it is stale forever + `Authority::is_behind`, an exhaustive match in which a home is behind nothing, deliberately **not** reading the revision through `Authority::version` (which answers the other question) + `tests/authority.rs::a_published_rule_states_which_revision_it_is`, `an_unnumbered_local_rule_renders_no_authority_line`, `a_cache_can_always_say_whether_it_is_behind` (extended to assert a *numbered* home is still behind nothing) + `tests/poke.rs::a_local_rule_is_never_behind_anything`, `an_upstream_rule_with_no_revision_pokes_nobody` + `rule::parse` tests `a_local_authority_keeps_its_revision`, `a_local_revision_out_of_range_says_so` + `tests/corpus.rs::every_committed_rule_round_trips_byte_identically`, which is what makes "no rule file needed editing" a fact about the real fifty-two.

**A copy's provenance under `kind = "local"` is refused, not ignored.** `from`, `pulled` and `adopted` describe a copy; a file claiming to be this install's own *and* naming where it was pulled from has said two contradictory things, and dropping the field silently reads back as a fact the parser accepted. **Enforced by:** `ParseError::UnexpectedField` + `rule::parse::a_local_authority_carrying_a_copys_provenance_is_refused`, which checks all three fields. Before this change the parser read `authority.version` for a local rule and threw it away.

### A recurrence report is anonymous, and the pseudonym never touches the machine

What: `relearn report --aggregate <clone> --generated YYYY-MM` writes this install's recurrence signal into a cloned aggregate repository. Five fields per observation — an upstream tag, a **bucketed** count, a month, a status kind, a control kind — and the field list is defined as much by what is absent: no title, no incident, no body, no path, no name, no repository, no language, **no day-level date anywhere**. Prints what would go; writes only on `--confirm`; transmits nothing.

**The absences are structural, not remembered.** `Observation` has five fields, so a sixth cannot be added by editing a renderer. `Month` has no day field, so no formatting slip or later edit can reintroduce one — `Month::of(Date)` is the only way a date reaches a report and it drops the day by construction.

**Enforced by:** `report::Observation` and `report::Month` (the types are the field list) + `tests/report.rs::no_day_level_date_appears_anywhere_in_a_report`, which scans **the whole rendered document** for any `N-N-N` token rather than the fields somebody remembered to check — the programme names this as real and easy to miss + `no_authored_text_of_any_kind_reaches_a_report` (six private strings, including a home path and a person's name, asserted absent) + `report::Bucket` with `counts_publish_as_buckets_with_published_boundaries` + `K_ANONYMITY_FLOOR` as a public constant with `the_k_anonymity_floor_is_a_readable_constant`, so both the producer and the aggregate read one number rather than two policies.

**Four conditions decide who is reported, each a different question:** the home is publishable (**A2's exclusion in its second flow** — an org-homed rule never appears here, exactly as it can never be contributed), the authority names an upstream (**B1's `Authority` answering "does this tag have a shared identity?"** — a local-only tag is a private name, and publishing one is the leak this flow is shaped around), the rule is not mandated, and it has actually recurred (a zero for every cached rule would publish the shape of the local corpus). **Enforced by:** `a_local_only_rule_is_never_reported`, `an_org_homed_rule_never_appears_in_a_report`, `a_mandated_rule_is_never_reported`, `a_rule_that_has_not_recurred_is_not_an_observation`.

**The control kind is sealed and never carries the control's name.** `gate:internal-payments-lint` publishes `gate`. **Enforced by:** `report::Control` (five variants, no free-text escape) + `a_control_kind_never_carries_the_controls_name`, which asserts both that the kind is published and that the name is absent from the document.

**The pseudonym lives in the clone, never on the machine** — Phase 0.2's decision, implemented. `reports/<install-id>.toml` **is** the id, recovered from the file stem, so nothing is read from a home directory, an environment variable or a dotfile; `tests/solo_mode.rs` needs no exemption and its `the_tool_reads_no_ambient_state` covers `home_dir`, `dirs::`, `XDG_`, `env::var` and (since B1) the clock. `--install` is required **only** the first time; afterwards the clone supplies it, so a mistyped id cannot silently fork one install's history into two. A conflicting `--install` is refused, and a clone holding several reports is an error rather than a guess.

**Observed through the binary (2026-09-13):** an empty clone → refused, naming the one-time `--install`; first run with `--install` → written, and the local-only rule absent from a two-rule library; **second run with no `--install` at all** → the id read from the clone, the same file **replaced** rather than a second one added (one file on disk); `--install deadbeef` against that clone → refused rather than switching pseudonyms.

**What it does not do.** It does not transmit, and it does not apply the k-floor — that is the aggregate's job (C1), and this row claims neither. And the report file carries no day-level date, but **the git commit that publishes it does**; the command says so in its own output, because no artefact here can prevent it.

### The raw incident cannot reach a contribution, because the contribution never holds it

What: `relearn contribute --tag <tag> --terms <list> --version <n>` prepares one rule to leave the machine. **`--version` is required as of 2026-09-13**, and it is what makes the receiving side possible at all: a cache records the revision it holds, so a rule published without one could never be told it is stale and `pull` refuses it outright. It is given on the command line rather than read from the rule file because publication is the act that assigns it — a fork's own authority records the revision it was *forked at*, which is not the revision it is being published as, and publishing a changed rule under a revision that already exists upstream would make every cache of it wrong in the one direction nobody could detect. A rule's `incident` is a verbatim quotation from a private working session; so is every `[[recurrence]]` incident, and there are more of those. **`Contribution` is a projection that borrows only the publishable fields and never borrows those at all** — not stripped, never held. What travels is `published_incident`, a separate authored field of its own type, so putting a raw incident where a published one belongs is a compile error rather than a call-site mistake. Nothing derives one from the other.

**Enforced by:** the `Contribution<'a>` type itself (private fields; `incident` and `recurrences` are absent from the struct, so no renderer can reach them) + `Contribution::of`, the only constructor, where all four refusals happen once + `tests/contribution.rs::the_raw_incident_is_nowhere_in_what_would_leave` (asserts the whole raw string *and* four fragments of it are absent, and that the published one is present) + `recurrence_incidents_never_travel` + `a_contribution_parses_as_a_rule_whose_incident_is_the_published_one`, which pins that upstream receives a rule its own tools can read.

**Four refusals, each a refusal rather than a transformation:** a withheld home (**A2's federation exclusion, finally consumed** — an org layer, or a project home naming a path), no published incident, a cache (it is already upstream's; adopt it first if you changed it), and **a mandate** — whose provenance is an approver by name, and which stripped of its approval would claim a sign-off it no longer records. **Enforced by:** `a_withheld_home_cannot_be_contributed`, `a_rule_with_no_published_incident_cannot_be_contributed`, `a_cache_cannot_be_contributed_but_a_fork_can`, `a_mandated_rule_cannot_be_contributed`.

**Two steps, and the default writes nothing.** `contribute` prints exactly what would leave and writes only on `--confirm`. The printed banner says what the matcher cannot do. **Writes a file; transmits nothing** — publication is a pull request a person opens, which is why §9's aggregate is a repository, and `tests/solo_mode.rs` is what keeps that true.

**What no artefact here proves, and this is the important sentence.** A published incident can identify a person, a customer or a repository without using any word a matcher knows. §12.2 puts that judgement on the contributor, and nothing in this row changes it: the code makes the raw field unreachable, refuses what it can recognise, and puts the exact text in front of a human. It does not read for meaning and must never be described as if it does.

### The scrub runs where the human is, and a disarmed matcher is not a clean one

What: before printing or writing anything, `contribute` runs the banned-terms matcher over the two fields a contributor authors — the published incident and the body — and a hit stops everything. The matcher is the one from `scripts/no-banned-names.sh`, reimplemented over a string rather than a tree (§12.2 names that split); the **term-list format is the contract between the two implementations**, and the duplication is recorded in the decisions log rather than pretended away.

**Enforced by:** `scrub::TermList::parse` (the only constructor; refuses a list with no salt or no terms) + `scrub::Hit`, which carries a location and a length and whose `Display` emits neither the match nor its context + `--terms` being a **required** argument, so "ran with no list" is unconstructible rather than an exit code to remember — stronger than the script's exit 2 + `tests/scrub.rs` (`a_term_is_found_however_it_is_spaced_or_cased` covering the joining pass, `a_term_fused_to_a_neighbour_is_found` covering the sliding pass, `a_finding_reports_location_and_length_but_never_the_match`, `an_unusable_term_list_is_refused_not_treated_as_empty`) + `src/scrub.rs` unit tests for normalisation and hex decoding. **The test lists are synthetic** — salt and digests built in the test — so this suite never needs the real list and can never print a real name.

**Observed through the binary (2026-09-13):** a rule whose raw incident named a person and a home path → the contribution printed with neither, the published account in its place, nothing written; a protected name inserted into the published incident → `error: 'published_incident' contains a protected name — a protected name appears at character 129 (length 5)`, exit 1, **the term itself absent from the diagnostic**; `--confirm` → written, and `grep` for the two private strings in the written file returns 0; a term list with no salt → refused, exit 1, naming the disarmed state rather than passing clean.

### A contribution warns about citations the destination does not carry

What: `contribute` reads its destination for a second question and reports every tag the rule cites that is not published there, distinguishing *not published there yet* from *can never be published — its home never leaves a machine*. Printed before `--confirm`, like everything else that flow shows.

**The publishable unit is a rule plus its citation closure, and this is what says so at the moment it matters.** Publishing a subset of a corpus whose rules cite each other hands every subscriber a library with dangling references — and `lint` makes those `Warning`s, fatal at its default threshold, so a new subscriber's **first command fails**. That is the worst possible first impression of a shared corpus.

**A warning, not a refusal**, and the distinction the warning draws is why: a rule may legitimately cite one whose home never leaves a machine, and refusing would make it permanently unpublishable. One case resolves when somebody publishes something; the other never does, so telling them apart is the whole value.

**Enforced by:** `contribute::dangling_citations` + `contribute::Citation` + `tests/contribution.rs::a_citation_the_destination_lacks_is_reported`, `a_citation_of_a_withheld_rule_says_it_can_never_be_published`, `a_citation_the_destination_carries_is_quiet` (a rule citing itself is not a dangling reference), `a_citation_of_a_tag_nobody_holds_is_still_reported`. The citation scan is `lint::cited_tags`, made public rather than copied — it reads the **body only**, because provenance legitimately names retired and foreign tags, and a second scanner would drift from that reasoning.

**Found by the second install, not by a fixture (2026-09-13).** Three rules were published; the subscriber's first `lint` failed on two dangling references, because the closure of those three was five. Probed both ways afterwards: publishing into an empty corpus prints the warning naming `R:wired-artifact`; publishing into the corpus that now carries the closure prints nothing.

### A republication goes forwards, and no rule-file write is unguarded

What: publishing a rule the shared corpus already holds must carry a **greater** revision than the one already there. Republishing at or below it makes every existing cache read as *newer than upstream* — and nothing anywhere notices, because `cache-behind` decides staleness by comparing exactly the two numbers that have been corrupted. A first publication supersedes nothing, so any revision starts it.

**Enforced by:** `contribute::SupersedingVersion`, a witness minted from the proposed revision and whatever the destination already publishes. `Contribution::to_document` takes **only** that type, so a backwards republication is not a mistake a call site can make — it is a document that cannot be built. `contribute::NotSuperseding` carries **both** numbers, because a contributor told only "refused" has to go and look up the one the tool has just read. + `tests/contribution.rs::a_first_publication_supersedes_nothing`, `a_republication_below_or_equal_to_what_is_published_is_refused` (every revision from 1 to the published one, and the message names both), `a_contribution_can_only_be_rendered_with_a_superseding_version`.

**`contribute` read nothing about its destination before this**, and that is how the second half of this row was found. It wrote with a **bare `fs::write`, no guard at all**, from B2 until 2026-09-13: publishing over an existing rule of the same tag destroyed it silently, and publishing over a *different* rule at the same path did too. The omission survived because `contribute` was the only rule-file writer that could not be compared against a sibling — the moment `write_cache` arrived there were two that guarded and one that did not. All three now go through one `guarded_write`, so a future writer inherits the refusal instead of reimplementing it. **Enforced by:** `fsio::write_contribution` + the shared `fsio::guarded_write`.

**Observed through the binary against a shared directory (2026-09-13):** against a drive publishing revision 4, `--version 2` → *"revision 2 does not supersede the 4 already published"*; `--version 4` → the same, naming 4 twice; `--version 5` → written, and the drive moved to 5. With a **different** rule's file sitting at that path, `--version 9` → *"already exists and is not rule R:no-sentinel-values — refusing to overwrite it"*, and the other rule was still there afterwards.

**What it does not check:** that the *content* changed. Publishing revision 6 of a byte-identical rule is allowed — the revision means "this is the sixth thing I have published under this tag", not "the sixth distinct one", and a content check would make a no-op republication an error rather than a waste.

### `pull --all` — the status check to run before starting work, and the prune

What: `relearn pull --all --upstream <dir> --from <name> --on <date>` works over the whole shared corpus instead of one tag. **Without `--confirm` it is a report**, which is what makes it the thing to run before starting a project: what this install would take, what it would refresh, what it would drop, and everything it would leave alone. `--scope` narrows it; `--prune` also removes caches that are gone or retired upstream. Every rule in both corpora appears exactly once.

**Asking for all of them is still asking.** §10 forbids automatic sync in either direction, and nothing here runs on a schedule or decides on its own that a rule has become relevant — `--all` is a person naming a corpus instead of a tag, printing first and writing on confirmation. `--scope` is §10's own answer to a corpus too large to compile, through the same `Rule::serves` every other narrowing uses; an unscoped upstream rule is taken regardless, which is A1's safety default.

**The half a bulk command usually hides is what it did *not* do.** `Plan::skipped` names every untouched rule with its reason — already current, yours, a fork you took, no upstream revision, a withheld home, an audience you did not ask for, unwanted but kept. A reader given only the actions would have to diff two libraries by hand, which is what they ran the command to avoid.

**`--prune` removes only caches, and that is the entire safety argument.** A cache is *regenerable*: dropping one loses nothing a later pull cannot restore. A rule this install owns and a fork it took are **source**, and nothing regenerates either — so `DroppableCache`, the third witness, refuses both, and `fsio::remove_cache` cannot be called with anything else. Removal is opt-in and an unwanted cache is still *reported* under the default, so the status check tells you what a prune would take without taking it. This is §12.6's third resolution — pull, adopt, **drop** — reached deliberately; an upstream retirement still only warns, and what changed is that the warning now has a command to act on.

**Enforced by:** `rule::DroppableCache` (constructor refuses `Local` with `YoursToKeep` and `Adopted` with `ADeliberateFork`) + `pull::Plan` + `pull::Prune` (an enum, so a call site reads `Prune::Drop` rather than `true`) + `fsio::remove_cache` + `tests/pull.rs`: `a_plan_takes_what_is_new_and_refreshes_what_is_behind`, `a_plan_names_everything_it_leaves_alone`, `an_audience_bounds_what_a_bulk_pull_takes`, `only_a_cache_can_be_dropped`, `a_cache_is_unwanted_when_it_is_gone_or_retired_upstream`, `nothing_is_dropped_unless_pruning_was_asked_for`, and through the real binary `pull_all_takes_refreshes_prunes_and_then_converges` — whose **second run** is the assertion that matters, because a bulk operation that is not idempotent is one nobody can run twice without reading the output first — plus `pull_with_neither_a_tag_nor_all_is_refused`.

**Found while building it:** a retired upstream rule that the install already held was reported twice in one run, once as *"already at upstream's revision"* and once as *"unwanted, and kept"* — the same rule current and unwanted at once. The attic check had been conditioned on whether the rule was held; it is not, and the held case is now decided entirely on the held side.

**Observed through the binary (2026-09-13)** against a drive of five rules and an install holding four: the report named 2 to take, 1 to refresh, 2 to drop and 2 left alone, with `R:mine` kept as *"yours, not a cache"*; `--scope rust` moved `R:javan` to *"serves an audience you did not ask for"*; `--prune --confirm` wrote three files and removed two; `R:mine` was still there; `check` validated the result; and the **second** `--prune --confirm` printed `Nothing to do.`

### `relearn pull` — the receiving half, and the only thing that creates a cache

What: `relearn pull --rules <dir> --upstream <path> --tag R:x --from <name> --on <date>` takes a copy of an upstream rule into this install as a cache. Until 2026-09-13 this install could **publish** a rule and could not **receive** one: `Authority::Cached` existed, `adopt` existed, three poke triggers read caches — and nothing anywhere created one, so a cache could only be made by hand-writing an `authority` table. All of that was dormant.

**Nothing is fetched and nothing is synced.** `--upstream` is a path to a directory both installs can see — a common drive — given on the command line like every other input, so `tests/solo_mode.rs` is untouched and the same corpus compiles identically anywhere. A rule arrives because somebody asked for it by tag, which is §10's *"no silent adoption"* and *"no automatic sync, in any version, in either direction"*.

**A cache keeps the home it arrived with**, which is the whole point: it compiles into this install's layer exactly like a local rule. `Home` says which layer, `Authority` says who maintains it, and they are independent. `--from` **names** the upstream and never locates it — the path says where the corpus sits today, and a rule file that travels must not bake in one machine's layout.

**Enforced by:** `rule::PulledRule` — the second witness, see below — plus `fsio::write_cache` + `tests/pull.rs`: `a_pull_mints_a_cache_carrying_its_provenance`, `a_cache_keeps_the_home_it_arrived_with`, `an_unnumbered_upstream_rule_cannot_be_pulled`, `a_withheld_home_cannot_arrive_any_more_than_it_can_leave`, `re_pulling_over_a_cache_updates_it`, `pulling_over_your_own_rule_is_refused`, `pulling_over_a_deliberate_fork_is_refused`, `pull_writes_nothing_without_confirm_and_then_writes_the_cache` (through the real binary, which then runs `check` over the corpus the cache landed in), `pulling_a_tag_the_drive_does_not_have_is_an_error`.

**Four refusals, asked once at the only place a cache can come into existence.** A **withheld home** never arrives, for the same reason it never leaves — a project home names somebody else's filesystem path, an org layer is an organisation's own, and `Home::federation` decides both. An **unnumbered** upstream rule is refused outright rather than cached: a copy of it could never be told it is stale, so `cache-behind` would be dead for it forever, and the refusal belongs at the door rather than months later. Pulling over **your own rule** would replace hand-authored source with a stranger's; pulling over a **deliberate fork** would discard both the change and the provenance of what it was forked from. Re-pulling over a *cache* is the update path and works — that is what makes a cache a cache.

**The second witness is the type-level point, and the reason this is not a flag.** `fsio::write_rule` takes an `EditableRule` whose constructor *refuses a cache* — exactly what a pull must write. A `bool` to skip that refusal would make "edit a cache in place" reachable by passing `true`, which is the state the cached-rule row below spent a phase making unconstructible. So there are two write paths taking two witnesses, and **neither can be minted for the other's subject**: `write_rule` can only ever write source this install owns, `write_cache` only ever a copy it does not. **Enforced by:** `tests/pull.rs::a_pulled_rule_is_never_editable_and_an_editable_rule_is_never_a_cache`, beside the existing compile-fail pin for the first half.

**A pulled cache carries no recurrences and no published incident.** Upstream's history of its own rule is theirs, and copying their count would fabricate local evidence for a rule this install has not seen fire once. The published incident is what upstream wrote *in order to publish*; a cache has nothing to publish.

**Observed end to end through the binary (2026-09-13), two installs and a shared directory.** Install A authored a `published_incident` on a global rule and ran `contribute --version 3 --out <drive>/rules --confirm` → the drive received the rule carrying `authority = { kind = "local", version = 3 }` and the **published** incident in place of the private quotation. A project-homed rule tried first was refused: *"this rule's home never leaves the machine"*. Install B ran `pull --from shared-drive --on 2026-09-13 --confirm` → `authority = { kind = "cached", from = "shared-drive", version = 3, pulled = "2026-09-13" }`, and `build --targets claude` over B's corpus emitted it into B's global skill like any local rule. `adopt` then turned it into a fork, after which `pull` refused: *"a fork you took deliberately — pulling would discard both your change and the provenance"*. A published revision 4 on the drive made an install still holding the cache print `poke [broadcast]: your cache of R:no-sentinel-values is at revision 3 and upstream publishes 4`. **That is the first time any of the federation machinery has fired against something that actually happened rather than a fixture.**

### A cached rule cannot be edited in place, and `adopt` is the loud fork

What: `Authority::Local | Adopted { .. } | Cached { from, version, pulled }` records whether this install is a rule's home, holds a cache of one whose home is elsewhere, or holds a fork it took deliberately. **Editing a cache is a silent fork** — the edit succeeds, P2 is gone, and nothing records that the copy and its source have diverged — so `fsio::write_rule`, the only path that writes a rule file, takes an `EditableRule` witness whose only constructor refuses `Cached`. A second write path cannot reach the filesystem without minting the witness, and minting it is where the question is asked. `relearn adopt --tag <tag> --on <date>` converts a cache into an `Adopted` rule: editable, and it **remembers** what it was forked from, at which upstream version, and when.

**Enforced by:** `EditableRule::of` (private field, the only constructor, refuses `Cached`) + `fsio::write_rule` taking it + `tests/compile_fail/write_rule_rejects_a_bare_rule.rs`, which pins that passing a bare `&Rule` does not compile — **by its error** (`E0308`, `expected &EditableRule<'_>, found &Rule`), not merely by failing + `tests/authority.rs` (`a_cached_rule_cannot_be_minted_editable`, `a_local_rule_and_an_adopted_one_are_both_editable`, `writing_a_cached_rule_is_impossible_and_nothing_reaches_disk`, which also writes a *local* rule successfully so the refusal is measured rather than a writer that never works, `a_written_rule_round_trips`, `writing_refuses_to_clobber_a_file_that_is_not_the_same_rule`) + `Authority::is_editable`, one exhaustive match that is the only definition of the word, which `Rule::is_editable` delegates to rather than restating.

**Three variants, not two.** A cache that became plain `Local` on adoption would be indistinguishable from a rule authored here — the silent fork arriving through the command that exists to make forking deliberate. `adopting_records_what_it_was_forked_from_and_when` and `only_a_cached_rule_can_be_adopted` pin both halves; adopting twice is refused because it would overwrite the first fork's provenance with a second that never happened.

**The total order is the requirement, not a nicety.** `Version` is a monotonic `u32`, so two caches of one rule are comparable for **every** pair and `is_behind` always has an answer. `versions_are_totally_ordered` asserts trichotomy explicitly; `a_cache_can_always_say_whether_it_is_behind` asserts the question it exists for. A content hash would say two copies differ and never which is newer — not the question a staleness check asks.

**Emitters treat a cache exactly like a local rule**, which is the point of caching one. **Enforced by:** `tests/properties.rs::authority_never_reaches_an_emitted_path_or_body` — for any generated rule, replacing its authority with `Local` changes not one byte in any of the five emitters — and by `arb_authority` now feeding every other property in that file.

**Observed through the binary (2026-09-13):** a cached rule adopted → `authority = { kind = "adopted", from = "relearn-upstream", version = 3, pulled = "2026-09-10", adopted = "2026-09-13" }` in the file; adopting it again → `error: this rule was already adopted from 'relearn-upstream' at version 3`, exit 1; an unknown tag and an impossible date each refused by name, exit 1.

**What it does not prove.** Nothing *creates* a cached rule yet — pulling is a later phase, so today a cache arrives by hand-writing the `authority` table, and `adopt` is the only thing that consumes one. The staleness question `is_behind` answers has no upstream to ask about until then.

### An org-homed rule can never leave the machine

What: `Home::Org { name }` carries an organisation's own engineering principles — the layer a corporation fills privately — and **no rule homed there may ever be contributed upstream or appear in a recurrence report.** Not by a filter in the publishing path, which lives in one code path while a second one forgets it: by `Home::federation`, an exhaustive match with no catch-all arm, so **adding a home variant without deciding its federation behaviour is a compile error**. `Rule::is_publishable` reads that single answer, so no second predicate can drift from it. Established deliberately *before* `contribute` or `report` exist: an exclusion added after the publishing code is one somebody has to remember to apply to it.

**Enforced by:** `Home::federation` (exhaustive, no wildcard) + `Federation`, a two-valued enum rather than a `bool`, because this is the one answer in the codebase where getting it backwards publishes something private + `tests/compile_fail/home_match_must_be_exhaustive.rs`, which pins the mechanism as a negative — a match written before `Org` existed no longer compiles, with the expected error (`E0004`, naming `Org`) asserted rather than merely "it failed" + `tests/federation_exclusion.rs::an_org_home_is_withheld_and_the_other_homes_are_not` (stated over **every** home kind, so it measures the enum and not the one variant it was written for), `a_rule_is_publishable_exactly_when_its_home_is`, and `the_federation_match_has_no_catch_all_arm`.

**The wildcard check is the half a compiler cannot hold.** `_ => Publishable` compiles forever and silently publishes every variant added after it, so that test reads `home.rs` itself, with comments stripped so the prose explaining the rule cannot trip it (`[R:detector-excludes-own-definitions]`). **Probed 2026-09-13:** replacing the two explicit arms with `_ =>` makes it fail naming the arm; restoring them makes it pass.

**Invocation.** `cargo test`, on both OSes in CI.

**What it does not prove.** That a home variant added in 2027 is classified *correctly* — only that it must be classified at all. And **nothing consumes the answer yet**: `contribute` and `report` are later phases, so today this is a decision with an enforcing artefact and no caller. `Project` is also `Withheld` — its home carries a filesystem path, which is a private identifier, and a stranger's project home is meaningless upstream — a decision the exhaustive match forced this phase to take and which B2 should confirm; `TODO.md` carries it.

### A mandated rule carries its approval, and is held out of the recurrence statistics

What: `Origin::Mandated(Approval)` beside `Mined | Codified`, for control-framework requirements — which are not corrections: they have no incident and were never mined. Their provenance is the sign-off, so `approval = { by, date, control }` is **required when and only when** the origin is mandated. Both halves are refusals, not warnings: a mandate with no signer is the unenforced guarantee the field exists to prevent, and an approval on a mined rule is dropped nowhere — it stops the build, because a silently ignored approval reads to the next person as a rule that was signed off when it was not.

**Enforced by:** the `Origin::Mandated(Approval)` **payload** — the approval is the variant's data rather than an `Option<Approval>` field beside it, so "a mandate with no approval" and "an approval on a mined rule" are both unconstructible rather than checked + `Origin::parse(kind, approval)`, one perimeter that sees both halves and returns `OriginError::MandateWithoutApproval` / `ApprovalWithoutMandate` + `rule::origin` tests (`a_mandate_without_an_approval_is_refused`, `an_approval_without_a_mandate_is_refused_and_names_the_origin`, `as_str_round_trips_through_parse` over all three variants) + `rule::parse` tests end to end (`a_mandate_parses_with_its_approval_table`, `a_mandate_without_an_approval_stops_the_build`, `an_approval_without_a_mandate_stops_the_build`, `an_approval_with_a_bad_date_names_its_own_field`, `an_unknown_field_in_an_approval_is_rejected`) + `rule::serialize` tests (`round_trips_a_mandate_with_its_approval`, `a_rule_that_is_not_mandated_renders_no_approval_line`, `the_approval_line_follows_origin`).

**Held out of both counter-metric numbers, and the hold-out is reported.** `Rule::counts_toward_recurrence_statistics` is the single definition; `lint::tally` reads it for both filters rather than testing the origin twice. A mandate cannot be recurrence evidence (nothing was mined) and must not count as *inert* either — inert means "authored and never fired", a judgement about something meant to be evidence, and a control-framework requirement never was. `Tally::mandated` and `Tally::evidential` expose the hold-out and the honest denominator; `relearn lint` prints them **only when a mandate exists**, because a permanently-zero column is one readers learn to skip. **Enforced by:** `tests/federation_exclusion.rs::a_mandated_rule_is_neither_recurrence_evidence_nor_inert` and `rule::origin::a_mandate_is_neither_mined_nor_inert_material`.

**And the tool still claims no regulatory alignment.** Recording the approver is what lets a claim follow from a record; the tool asserting one itself would be the unenforced guarantee `[R:guarantee-needs-a-reader]` names — in a regulated environment a liability rather than a feature.

### A rule declares the audiences it serves, and an unscoped rule serves all of them

What: a rule may carry `applies_to = ["rust", "java"]` — the audiences that should *load* it, distinct from the one `home` that *owns* it. `build`, `verify` and `list` take a repeatable `--scope`, which narrows by audience and composes with `--home`. The table: with no `--scope` every rule is emitted, scoped ones included; with `--scope rust` an unscoped rule and a rust-scoped rule are both emitted; with `--scope java` the rust-scoped rule is withheld and **the unscoped rule is not**; with both, either matches. **The default is the safety decision, not a convenience:** adding `applies_to` to one rule must never be able to remove a *different* rule from an existing build, because a silently dropped rule is a lost correction. Scope decides *whether* a rule is included and announces itself in the body — `Home` alone decides *where* it lands, so no scope can fabricate a path.

**Enforced by:** `ScopeTag` (private field, constructible only via `ScopeTag::parse`: trimmed, non-blank, bounded, lowercase kebab, so a malformed audience is unrepresentable) + `Rule::serves`, which holds both defaults in one place no caller can reimplement + `tests/scope_filter.rs` (the four rows as four assertions, plus `scope_never_changes_the_emitted_path`, `scoping_a_rule_changes_exactly_one_announced_line`, `home_and_scope_narrow_independently`) + `tests/properties.rs::narrowing_never_touches_an_unscoped_rule` — **the whole-space form of the safety default**: for any library and any audience, every unscoped rule survives, and every emitter's output for those rules is byte-identical to its output with no narrowing at all + `no_audience_emits_exactly_what_an_unnarrowed_build_emits` (no `--scope` is not the empty audience) + `scope_never_reaches_an_emitted_path` (stripping every scope moves no file, from any emitter) + `scope_reaches_a_body_only_through_the_audience_note` (and changes no byte but the announcement) + `applies_to_survives_the_round_trip_and_empty_renders_nothing` + `rule::parse` tests (`a_rule_without_applies_to_parses_as_unscoped`, `applies_to_parses_as_an_array_of_scopes_in_file_order`, `a_malformed_scope_stops_the_build_naming_the_field`, `a_repeated_scope_is_a_parse_error_naming_it`, `scopes_are_trimmed_before_the_duplicate_check`) + `rule::serialize` tests (`an_unscoped_rule_renders_no_applies_to_line`, `round_trips_applies_to_in_file_order`, `the_applies_to_line_follows_home`) + `tests/corpus.rs::every_committed_rule_round_trips_byte_identically`, which is what makes "no rule file needed editing" a fact over the real fifty-two rather than over generated ones.

**A duplicate scope in one rule is a parse error**, for the same reason a duplicate rule tag is: one written intent with two behaviours, the second inert. Checked after trimming, so `["rust", " rust "]` is caught.

### A scoped rule announces its audience in every emitted format

What: a rule carrying `applies_to` emits `> Written for the rust and java audiences.` under its
heading — one audience in the singular, three or more comma-separated before the last, in the
order the author declared. An **unscoped rule announces nothing**: it is emitted under every
audience, so there is no audience to name, and annotating it "applies to all" would be a claim
the rule does not make. Decided 2026-09-14; until then a scoped rule and an unscoped one emitted
byte-identically, and a reader of `skills/domain-low-latency/SKILL.md` could not tell that the
rule in front of them had been written for someone in particular.

Third of a family. `enforcement_note` says how firmly a rule is held, `recurrence_note` says
whether it has bitten, `audience_note` says whether it is yours — and it is spliced first,
because that is the reader's first question.

**Enforced by:** `emit::audience_note` (one definition, `Option<String>`, so "unscoped" is
`None` rather than an empty string a splice site could render as a blank blockquote) +
`tests/properties.rs::an_audience_is_announced_in_every_emitted_format` — **the wiredness
artifact**: one function, five independent splice sites, so it counts announcements per file
across all five emitters rather than testing the function, exactly as the recurrence note is
held + `scope_reaches_a_body_only_through_the_audience_note`, which deletes the note from a
scoped build and requires the remainder to be byte-identical to an unscoped one, so the
announcement is provably the *whole* of the difference + `scope_never_reaches_an_emitted_path`,
the surviving half of the old byte-identity property + `emit` unit tests
(`an_unscoped_rule_has_no_audience_note`, `one_audience_is_named_in_the_singular`,
`two_audiences_are_joined_with_and`, `three_or_more_audiences_are_comma_separated_before_the_last`,
`the_declared_order_is_the_announced_order`, `the_audience_note_is_independent_of_the_other_two`)
+ `tests/scope_filter.rs::the_announcement_names_every_audience_in_house_prose` and
`an_unscoped_rule_announces_nothing`, which pin the sentence a human actually reads.


### A codified rule may name the artefact it was written down from

What: `Origin::Codified` carries an optional `SourceArtefact` — `source = "..."` beside `origin`,
exactly as `approval` sits beside a mandate. It is refused on any other origin
(`OriginError::SourceWithoutCodification`), and refused blank. The absent case stays free: all
fifty-seven committed rules parse and emit unchanged, and `relearn verify` reported 69 files up to
date across the change.

Until 2026-09-18 the variant deliberately carried no payload, on the recorded ground that "a
practice written down from standing doctrine is meaningful without naming a document". That was
sound for the corpus it was written against — every codified rule had been ported out of the
author's own always-loaded instruction file, and the `incident` prose already said so. It stopped
being sound the first time a practice was codified from **someone else's** published artefact,
where the document is not a footnote to the provenance but *is* the provenance
(`[R:source-practice-from-its-artefact]`). Optional, not mandatory, is therefore the whole design:
the mandate's approval is required because a mandate with no signer is an unenforced guarantee,
and a codified rule with no named artefact is not the analogous defect.

`source` is also the **third authored field that travels on `contribute`**, and adding it exposed
that the banned-terms scan list lived at the CLI call site and named two. The enumeration moved to
`Contribution::authored_texts`, beside the projection that decides what travels
(`[R:names-travel-with-the-quote]`).

**Enforced by:** the type — `Codified(Option<SourceArtefact>)` makes "a source on a mined rule"
and "a blank source" both unconstructible, and `Origin::parse` takes the origin, the approval and
the source at **one perimeter** with no catch-all over the triple, so a fourth origin cannot
inherit either payload's policy by default + `tests/codified_source.rs` (7 pins:
`a_codified_rule_carries_the_artefact_it_was_written_down_from`,
`a_codified_rule_without_a_source_still_parses`, `the_source_survives_a_round_trip`,
`a_rule_without_a_source_renders_no_source_line`, `a_source_on_a_mined_rule_stops_the_build`,
`a_source_on_a_mandated_rule_stops_the_build`, `a_blank_source_stops_the_build`) +
`tests/properties.rs::neutral_round_trip_preserves_the_rule`, whose `arb_origin` now generates
**both** codified shapes — they share the spelling `codified`, so a round-trip that only saw the
payload-free one would pass while the serializer dropped every source in the corpus +
`tests/contribution.rs::every_authored_field_that_travels_is_offered_to_the_scan` and
`a_contribution_with_no_source_offers_no_source_to_the_scan` (an empty string would scan clean and
read, in any report, as a field that was checked).

**Closed 2026-09-19 — the scan now covers every authored field.** `title` and `error_class` were
left out on the ground that widening would newly refuse contributions that pass today, and a gate
that refuses previously-legal input gets muted. That traded a real exposure for a hypothetical
one: a contribution is a deliberate act behind `--confirm` and an explicit term list, so refusing
one is the gate working, and nothing about a `title` stops a product name being in it. Scanning
four fields and reporting clean was the more expensive mistake, because the report named what had
been checked and not what had not. `tag` and `home` remain out and the reason is recorded on
`authored_texts`: a tag is a published identifier and a project `home` never reaches a
contribution at all, being withheld by `Home::federation`.

### A sourced rule announces its artefact in every emitted format

What: a codified rule naming an artefact emits `> Written down from <artefact>.` under its
heading, in all five formats. A rule naming none announces nothing, so output is unchanged for
every rule written before the field existed.

Fourth of the family, and **last by rank on purpose** (`[R:order-by-explicit-rank]`).
`audience_note` says whether the rule is yours, `enforcement_note` how firmly it is held,
`recurrence_note` whether it has bitten — the reader's questions in the order they are asked.
Provenance answers none of them: it is what a reader follows to check the rule against the thing
that defines it, asked last and least often.

*Recorded because it cuts against the feature:* a citation is a pointer into a reader's — or a
model's — existing knowledge of the source, which helps where the rule agrees with it and
**misleads where the rule deliberately departs from it**. `[R:transient-state-is-not-a-terminal-state]`
is the live example: it forbids sharing a return value between "drained" and "producer mid-push",
which Vyukov's own `mpscq_pop` does. The note names the artefact; it never implies the rule agrees
with it.

**Enforced by:** `emit::source_note` (one definition, `Option<String>`, so "no artefact" is `None`
rather than an empty string a splice site could render as a blank blockquote) +
`tests/properties.rs::an_artefact_is_announced_in_every_emitted_format` — **the wiredness
artifact**: one function, five independent splice sites, and it **counts** announcements per file
rather than asserting presence, because a concatenated file holds many rules and a file-wide
negative is false the moment one of them names an artefact (proptest produced that counterexample
on the eighth case, against the first version of this property) +
`a_source_reaches_a_body_only_through_the_source_note`, which deletes the note from a sourced
build and requires the remainder to be byte-identical to a sourceless one, so the announcement is
provably the *whole* of the difference. Verified through the binary as well as in-process: a
62-rule library with three sourced rules emitted three announcements each into the skill,
`AGENTS.md` and `copilot-instructions.md`, and one into the per-rule `.mdc`.

### `relearn new` — author a rule file from its fields

What: `relearn new --tag … --title … --error-class … --home … --on … --incident-file … --body-file …`
writes one rule file and refuses to touch anything that already exists. `--home` takes the
authorable form (`global`, `domain=<name>`, `org=<name>`, `project=<path>`), the optional payloads
are refused on the origins that have no use for them, `--scope` is repeatable, and `--dry-run`
renders the document to stdout and writes nothing. Prose comes from a literal or a file, and is
**required** either way: a rule whose body or incident defaulted to a placeholder parses, emits and
says nothing, which is the silently useless rule the provenance requirement exists to prevent.

Two things it deliberately does not take. **Status**, because a rule that is graduated or atticked
on the day it is written is not a rule anyone learned anything from — it is always `active`. And
**`published_incident`**, which is authored when a rule is contributed and not before; a rule file
is hand-editable by design, so adding it later is the ordinary path rather than a gap.

`--home` is **not** the slug `list --home` takes, and that is the interesting constraint.
`HomeSlug` is a one-way filesystem identity — it turns a project path's separators into dashes, so
`project-c-repo-sub` names a home it cannot reconstruct. A slug can select an existing home; it
cannot construct one, and a parser that pretended otherwise would silently invent a different
project path.

**Nothing here is a second way to write a rule file.** The witness is minted the same way, the
document is rendered by the same serializer, and the clobber refusal is the shared one — so a rule
this command produces is byte-identical to the same rule typed by hand, and `check` is the arbiter
of both.

**Enforced by:** `cli::new_rule` + `cli::parse_home_spec` (exhaustive over the four `Home` kinds
with no catch-all, so a fifth variant becomes a compile error at the one place that would otherwise
not notice) + ten unit pins in `cli`, of which the load-bearing one is
`what_new_writes_is_a_rule_the_library_validates` — not "a file appeared" but "the library accepts
it", because `check` is the arbiter for a hand-typed rule and must be the arbiter for this one +
`new_never_replaces_an_existing_rule_even_with_itself`,
`an_occupied_target_is_refused_and_left_untouched`, `a_dry_run_writes_nothing`,
`every_home_kind_is_authorable_and_a_slug_is_not_a_home`,
`a_source_is_written_for_a_codified_rule_and_refused_otherwise`, `a_half_given_approval_is_refused`,
`a_missing_body_is_refused_rather_than_defaulted`, `prose_reads_from_a_file`,
`a_repeated_scope_is_refused`. Smoke-tested through the real argv path, which is where the
create-never-replaces defect was found.

**The create-never-replaces refusal is `new`'s own, and is the one thing it adds to the shared
write path.** `fsio::write_rule`'s guard refuses a target occupied by a *different* rule and
permits overwriting the *same* one — exactly right for `adopt`, whose whole job is rewriting an
existing rule's authority in place, and exactly wrong for a create. Widening the shared guard would
have broken `adopt`, so the second question is asked in `new_rule` against
`fsio::rule_path`, which is now one definition rather than two copies of a path derivation.

### A truncated description drops titles by an explicit rank, never by the alphabet

What: a skill `description` has a hard 1024-character cap, so past roughly a dozen rules per home
it cannot name them all. Which titles survive is now decided by a stated rank —
recurrence count descending first, then `Status::instruction_reliance`, then tag as the
determinism tiebreak — instead of by the caller's tag order, which is the alphabet.

**The first two keys were the other way round until 2026-09-19.** Reliance led, on the reasoning
that the rank answers "what does it cost for this title to be absent", which is a question about
what *else* holds the rule. Defensible alone, inconsistent with everything around it: `lint` fails
CI on an unheld recurrence, the session-start hook leads with it, and the framework treats it as
the number that says whether a rule is working. The live casualty was
`[R:guarantee-needs-a-reader]` — `partial`, one recurrence, dropped from `global`'s description
while never-fired rules were kept. It is back, and `global` now leads with its recurred rules.

**A dropped title is a rule the matcher cannot fire on**, and for a home reachable only by
description (`domain-low-latency` and every non-language domain, which `LoadSemantics` makes
`OnRequest`) that is a rule that may never load at all. Measured on a sixteen-rule home before the
fix: the three titles dropped were the three whose tags sort last, and nothing about `p`, `t` or
`v` says "least worth loading". `[R:order-by-explicit-rank]`

The intent the rank states: **keep the titles whose absence costs most.** A rule held by prose
alone loses everything if its layer does not load; a graduated rule still has the named control
that claims its whole class, and loses only the tuning the instruction would have added before the
fact. On the real corpus this moved `global`'s three graduated rules out of the description and the
recurred ones to the front — the same fifteen slots, spent on the rules that have nothing else
holding them.

The rank orders the **description only**. The skill body stays in tag order, where a reader finds a
rule by name and a diff stays readable.

**Enforced by:** `Status::instruction_reliance` — a third policy over `Status`, matched
exhaustively with no catch-all, so a new status variant cannot compile until it decides how much it
relies on the instruction layer. Deliberately not a reuse of `Status::prose_coverage`, which is
two-valued and puts `Active` and `Partial` in one bucket: right for its own question ("did the
prose fail?"), wrong for this one, because a partly-held rule has controls a wholly-unheld rule
does not + `emit::claude::description_order` (the one place the rank is applied) + unit pins
`a_rule_prose_alone_holds_outranks_a_graduated_one_whatever_the_tag_says`,
`a_recurred_rule_outranks_one_that_has_never_fired`,
`equal_rank_falls_back_to_the_tag_so_emission_stays_deterministic`,
`the_rank_reorders_the_description_and_never_the_body`.

**Verified by probe, not asserted** (`[R:wired-artifact]`): the rank was temporarily reverted to
the old tag sort and all three ordering pins failed, then restored and they passed. A test that
passes under the behaviour it is supposed to forbid locks nothing.

~~**NOTHING YET — exposed:**~~ **Closed the same day** — by *"A skill description names every rule
in its home, and no longer truncates"* below. Was: the rank decides *which* titles are dropped, not
*that* titles are dropped; `global` still drops fifteen of thirty and `domain-low-latency` will
truncate again as it grows. A cap that cannot hold an inventory is a question about what the field
should contain, and is carried in `TODO.md`. It was answered by changing what the field contains —
tag bodies, not titles — and `tests/description_reaches_every_rule.rs` is now the budget check that
fails the day a home stops fitting.

**The rank did not become redundant**, which is why this row keeps its enforcing artefacts rather
than being struck through whole: it still orders the field, and it still decides the drops in the
residual case, on a corpus that has outgrown the budget twice already.

### The corpus carries a Java discipline and a mechanism-level low-latency discipline

What: `rules/` grew 57 → 83 on 2026-09-19. `domain-java` is a **new home** with 13 rules;
`domain-low-latency` went 2 → 15. The two domains were written together and are deliberately
separate, because the test that assigns a home when a rule is both language-specific and
performance-specific is **does the error class require a latency requirement to exist, or require
the runtime to exist** — and those pick out different rules.

The Java set is the language discipline the corpus had never carried: nullability, the
equals/hashCode contract, sealed alternatives with exhaustive switch, references escaping a class,
safe publication under the JMM, exception discipline, collection views, resource closing,
inheritance, boxed identity, `Serializable` as a second constructor, and one rule that exists
because **this corpus creates the hazard** — `[R:a-wrapper-type-is-not-free-here]` keeps
`[R:newtype-liberally]`'s design argument and refuses its cost argument, because a newtype is
zero-cost in Rust by construction and in Java only when escape analysis says so.

The low-latency set is mechanism and measurement: contended paths, the two-store publication window
from both ends, concurrency regimes, backpressure, memory ordering, false sharing, allocation and
residency, syscalls, and three rules about instruments that report confidently wrong numbers
(coordinated omission, safepoint-biased profilers, warm-versus-cold measurement).

`domain-java` matters for a reason beyond its contents: `java` is in `emit::domain_globs`, so the
home is `LoadSemantics::WhenReading` and emits `.claude/rules/domain-java.md` carrying
`paths: ["**/*.java"]` — it attaches on reading any Java file. `domain-low-latency` is not a
language, gets no globs, and remains `OnRequest`.

**Enforced by:** `relearn check` (83 validated) + `relearn lint` (7 findings, **identical to the
57-rule baseline** — the 26 add no overlapping scope, no home-slug collision, no dangling or
retired reference) + `tests/corpus.rs::every_committed_rule_round_trips_byte_identically` over all
83 + `relearn verify` on all three output roots (97 / 1 / 7 files) +
`tests/pack_counts.rs`, which caught both pack READMEs on landing: the copilot table still said 57
rules, and the claude inventory had no line for the new `domain-java` skill at all.

**Description headroom, stated because it is the constraint that shaped the set:**
`domain-java` sits at 884/1024 characters with no truncation, and 13 rules was chosen to keep it
there. `domain-low-latency` at 15 rules **does** truncate, dropping 2. That is the open
description-as-inventory question in `TODO.md`, not a defect in these rules.

### A skill description names every rule in its home, and no longer truncates

What: the `description` lists each rule's **tag body with hyphens turned into spaces** —
`no coordinated omission`, `a view is not a copy` — rather than its title. Every home now fits
inside the 1024-character cap with room, and the remainder counter is gone from the corpus.

| Home | Rules | Description | Dropped before | Dropped now |
|---|---:|---:|---:|---:|
| `global` | 30 | 884 | **15** | 0 |
| `domain-rust` | 18 | 508 | 0 | 0 |
| `domain-low-latency` | 15 | 612 | **2** | 0 |
| `domain-java` | 13 | 457 | 0 | 0 |

**The field had two jobs fighting over one budget** — be a trigger, and be an inventory — and the
inventory job was never its own: the skill body already lists every rule, and a matcher loads a
whole home or none of it. Listing titles cost sixty-odd characters each, so `global`'s thirty came
to 1824 against a 1024 budget; the same thirty tag bodies cost 749. The saving is not compression:
a tag is a field the author wrote, unique and stable, already a keyword phrase with no articles and
no qualifying clause. Turning the hyphens into spaces is what makes a matcher see separate word
tokens and a human see a phrase.

This is what the truncation **rank** could not do. Ordering decides *which* rules are dropped when
the field overflows, and it was necessary — `[R:order-by-explicit-rank]`, since the alphabet was
choosing — but at 1824 characters against 1024 no ordering makes the field hold them. The rank
remains, for the residual case.

**Enforced by:** `tests/description_reaches_every_rule.rs` over the **real corpus** — a *budget*
check rather than a formatting one, asserting the outcome that matters (no rule is unmatchable)
and failing on the day a home grows past what the field can hold, which is the warning that was
missing + `no_skill_description_has_had_to_truncate`, the symptom half, naming the home + the
`emit::claude` unit pins, which hold the formatting (`description_covers_each_rule_in_the_home`
now asserts the subject is present **and the title is absent**, so the saving is real rather than
additive).

**Two tests had their premise changed and were rewritten rather than relaxed.**
`description_is_quoted_so_a_colon_in_a_title_stays_valid_yaml` put a colon in a *title* to prove
the YAML quoting was load-bearing; titles no longer reach the field and a tag cannot contain a
colon, so left alone it would have passed while asserting nothing. The colon can still arrive by
one route — a **project path**, which `description_lead` interpolates and which on Windows begins
`C:` — so it now exercises that channel (`[R:verify-through-production-path]`) under the name
`description_is_quoted_so_a_colon_in_a_home_label_stays_valid_yaml`, renamed with the premise
because a test whose name states the wrong channel is the next reader's wrong belief. And the truncation
detector first matched the word `" more"`, which reported `global` as truncated when all thirty
rules were present, because `five-files-no-more` renders as `five files no more`: a detector whose
pattern occurs in the data it counts (`[R:detector-excludes-own-definitions]`). It matches
`+<digit>` now.

### Every rule sample in the documentation is a rule the parser accepts

What: `tests/doc_samples.rs` extracts every `+++`-delimited block from `README.md`,
`ARCHITECTURE.md` and the two worked examples, gives it a body where the document elides one, and
runs it through `rule::parse_document`. A sample that does not parse fails the build, naming the
file, the line and the parser's own diagnostic.

**Why it did not exist, which is the general shape rather than an oversight.** A sample is prose
to everyone who reads it and a rule to nobody, so no gate had a reason to open it: `verify`
compares generated files against the library and never reads a hand-authored document, and
`check` reads `rules/` and nothing else. A documentation example sits in the one place both are
blind to — and it is the first thing a new reader copies. On 2026-09-19 the front matter in
`README.md` and `docs/worked-example.md` were both extracted and both rejected with
`missing field origin`: the public face had been showing a document the tool refuses, on both
counts, for months, while `pack_counts` was catching a stale line count in the same session.

**Enforced by:** `tests/doc_samples.rs::every_rule_sample_in_the_documentation_parses` +
`every_listed_document_exists_and_carries_a_sample`, because `DOCUMENTED` is a list of paths and a
path can be renamed out from under it — without that, deleting a document would make this gate
quieter rather than louder + `the_extractor_finds_a_sample_whatever_the_line_endings`, since these
documents are **not** LF-pinned in `.gitattributes` (only `rules/**` and the emitted tree are), so
a CRLF checkout yields `+++\r`, matches no delimiter, and the gate would find nothing and pass on
the platform it was written on (`[R:xplat-fixtures]`) + a count assertion, because a gate that
finds nothing passes and that is the failure this whole file is about (`[R:wired-artifact]`).

**Verified by probe, not asserted:** `origin` was deleted from the README sample, the gate failed
naming `README.md:150` and the parser's `missing field origin`, and it passed again on restore.

### No published recurrence bucket denotes a single count

What: `Bucket` is `1-4 | 5-9 | 10+`. The `1` bucket is gone — the variant removed rather than
re-spelled, so the state cannot be reconstructed — and `aggregate::BUCKETS` drops to three, which
means an incoming report spelling `1` or `2-4` is `UnknownBucket` and is **refused**.

**Why it was a leak and not a preference.** The design specified `1 | 2-4 | 5-9 | 10+`; the
federation programme's own risk list warned, of that same boundary, that *"a bucket of 1–1 is not a
bucket"*. The design won where the two disagreed. The k-anonymity floor does not reach it: that
floor protects the **aggregate**, while a raw report is a file in a public git repository where
`recurrences = "1"` is an exact count for one install, republished every month it appears — the
fingerprint the coarsening exists to prevent, at the only size where a fingerprint is worth having.

**Enforced by:** `tests/report.rs::no_bucket_publishes_an_exact_count`, which states the property
rather than the boundaries — `Bucket::of(1) == Bucket::of(4)` (indistinguishable once published)
and `Bucket::of(4) != Bucket::of(5)` (the coarsening still carries signal), plus an assertion that
no bucket's spelling parses as an integer, taken over counts rather than over a list of spellings
written in the test, so a future variant cannot slip past it + `counts_publish_as_buckets_with_published_boundaries`
(the table) + `the_smallest_bucket_absorbs_zero` (zero is unreachable from `Report::of`, which drops
a rule with no recurrences, but `of` is total over `usize` so the arm is pinned).

**A test that had to be reworked rather than renamed.**
`tests/aggregate.rs::a_row_carries_the_distribution_and_the_control_kinds_seen` proved a row keeps
a *distribution*, using installs in `1` and `2-4`. Both merge into `1-4`, so renaming the fixture
would have collapsed every install into one bucket and left the test asserting nothing about
distribution. It now spans `1-4` and `5-9` and asserts both.

**Scope of the change, stated:** `src/report.rs`, `src/aggregate.rs`, three test files, and the two
places the boundary is written down in prose — `docs/federated-relearn.md` §5 and §12.3, and the
programme's risk list, which now records the item as closed rather than open. Nothing had been
published, so nothing needed migrating.

### `copilot-paths` — path-scoped Copilot instructions, one file per home

What: `relearn build --targets copilot-paths --out <dir>` writes
`.github/instructions/<home-slug>.instructions.md` per home, each carrying an `applyTo:` glob
derived from `LoadSemantics` — `**/*.rs` for `domain-rust`, `**/*.java` for `domain-java`, `**` for
global and project layers. Copilot attaches each file only to the files it is about, so eighteen
Rust rules stop occupying context in a repository with no Rust in it. That is the P6 argument that
motivated `--home`, applied to the one target that had no way to express it.

Reuse rather than new modelling: the per-`Home` glob table already existed for Cursor's `globs` and
the Claude rules layer's `paths:`.

**Every home gets a file, and that is the deliberate difference from `emit::claude_rules`.** That
emitter *skips* a home it cannot express — an unknown-language domain is `OnRequest`, a
`.claude/rules/` file cannot say "on request", and skipping is safe there because the Claude
**skill** target carries those rules by description instead. Copilot has no second channel in this
layer: skipping `domain-low-latency` would leave fifteen rules reachable through no Copilot file at
all, which is a correction lost. So an unscopable home declares `applyTo: "**"` — the narrowest the
format can say — **and states in its own header that it is unscoped and why**, because these files
are copied into a repository one at a time and the reader is the one choosing. A file nobody copies
costs nothing; a rule that reaches no file cannot be copied at all.

**Deliberately not in the default target set, and not committed.** It is the *alternative* shape to
`copilot`, not an addition: installing both states every rule twice, and this repository's own
`.github/` is read by Copilot, so committing both would double relearn's own instructions. `copilot`
stays the default; this is selected explicitly, like the packs.

**Enforced by:** `tests/copilot_paths.rs` over the **real corpus** —
`every_emittable_rule_reaches_exactly_one_file` is the load-bearing one and compares against
`emit::copilot`'s own source set, so the two Copilot shapes must carry **exactly** the same rules
(a rule in one and not the other is a correction that reaches Copilot only if you happened to
install the right shape) + `one_file_per_home_named_by_its_slug` +
`apply_to_is_the_homes_load_semantics`, which derives the expectation from `LoadSemantics` rather
than from a table written in the test + `an_unscopable_home_declares_that_it_is_unscoped` +
`an_empty_library_emits_no_file`. Smoke-tested through the real argv path: seven files, and
`domain-rust` carries `applyTo: "**/*.rs"` while `domain-low-latency` carries `**` and says so.

Because it is not committed, `verify` cannot grade it — the integration test is what does, and for
this target it is the stronger check: it asserts no rule is lost, which a byte comparison of a
committed tree would not.
### An audience no rule declares is an error for `build` and `verify`, and an empty listing for `list`

What: `relearn build --scope rsut` fails naming the declared scopes rather than emitting. The reason is **sharper than the unknown-home one and is recorded as such**: an unknown home produces an empty emission, which at least looks wrong; an unknown scope produces a tree that is *quietly missing every scoped rule* while every unscoped rule is still present — output that looks like success. `list --scope` mirrors `list --home` instead: printing nothing *is* an answer for a listing, and is not one for a gate.

**Enforced by:** `CliError::UnknownScope` + `restrict_to_scopes`, shared by `build` and `verify` so the two can never disagree about what is in scope (the same reason they share `restrict_to_home` and `emit_selected`) + `cli` tests `build_with_an_unknown_scope_is_an_error_naming_the_declared_ones` (which also asserts **nothing is written** before the rejection), `an_unknown_scope_against_an_unscoped_library_says_so` (the empty-declared-set case gets its own message rather than an empty parenthesis), `a_malformed_scope_is_rejected_before_the_lookup`, and `list_with_an_undeclared_scope_prints_nothing_and_succeeds`.

**Observed through the production path, not only the dispatch functions (2026-09-13):** `relearn build --scope rust` against the real corpus → `error: no rule declares scope rust (no rule in this library declares 'applies_to')`, exit 1, nothing written; `relearn build --scope Rust` → `error: --scope: scope must be [a-z0-9-] with first char [a-z0-9] (got "Rust")`, exit 1; `relearn build` with no `--scope` → 63 files, and `relearn verify` green on the committed tree.

### Solo mode: one engineer, one machine, offline, no account

What: `relearn` is a compiler a single engineer points at their own rules to emit their own instruction layers — Claude skills, Cursor `.mdc`, Copilot, `AGENTS.md`, project `CLAUDE.md`. It requires no account, no configuration file, no network, and no per-machine state: the input is the path given on the command line and the output is the path given on the command line. `docs/federated-relearn.md` designs a corpus that installs contribute to, cache from and report into, and **every one of those features is a standing reason to acquire a client, an account or a config directory**. Federation is a feature of the corpus, never a dependency of the compiler; §1's single-install guarantee states the four invariants and this row is the gate under them.

**Enforced by:** `tests/solo_mode.rs` — `the_tool_opens_no_socket` (no `std::net`, `TcpStream`, `TcpListener`, `UdpSocket`, `SocketAddr`, `ToSocketAddrs` anywhere in `src/`), `the_tool_spawns_no_process` (no `Command::new` — shelling out to `git` or `curl` is the back door to both the network and to ambient state), `the_tool_reads_no_ambient_state` (no `home_dir`, `dirs::`, `XDG_`, `env::var`, so the same corpus compiles identically on two machines), and `no_network_crate_is_in_the_dependency_tree` (a deny-list of networking, TLS and async-runtime crates checked against `Cargo.lock`, so a transitive arrival is caught too). Comments are stripped before matching, so a doc comment that *names* a marker while explaining we do not use it cannot turn the gate permanently red — `[R:detector-excludes-own-definitions]`.

**Invocation.** `cargo test`, which CI runs on `ubuntu-latest` and `windows-latest`. A test rather than a script, for the same reason as the dependency gate: it is wired by construction and cannot arrive disarmed.

**What it proves, and what it does not.** It proves nothing in this codebase *calls* a socket, a child process or the environment — which is what matters, since an uncalled socket cannot open — and that no network stack is in the tree. It does **not** prove emission is unchanged; `relearn verify` holds that and is the artifact to run beside this one. Two of the four invariants in §1 are not held here at all: *no configuration is required, and its absence is never a filter* waits on `Audience::Everything` and the emission round-trip, because there is no configuration yet to assert about, and *every federated field is optional* waits on the first such field. Both are carried in TODO.md Phase J rather than claimed here.

**Both halves were observed, not assumed (2026-09-13):** clean tree → 4 passed; a probe file under `src/` containing `std::net::TcpStream` → `the_tool_opens_no_socket` fails naming the file, line and marker. The two "measuring nothing" paths are refused explicitly: an empty source list and an unparseable `Cargo.lock` each fail rather than pass.

### The corpus names the fourth position in the signal family

What: `rules/` grew 83 → 84 with `[R:signal-needs-a-consequence]` — *give a repeated signal a
consequence, or stop emitting it.* `global`, `mined`, `active`.

**Why a new rule and not a recurrence,** which is the decision worth recording. It was first
proposed as a recurrence of `[R:guarantee-needs-a-reader]` and rejected on inspection: there,
nothing reads the state a sentence asserts; here a check read it correctly and a person did not
act. The corpus already held three positions in that family and the fourth was empty —
`guarantee-needs-a-reader` (nothing checks the claim), `wired-artifact` (something checks it and
accepts evidence anything could produce), `verdict-survives-the-channel` (the verdict is destroyed
in transit), `reconcile-wiring-at-start` (the control goes dark without announcing it). This is the
case where every link holds and the chain still ends, because its last link was a person with
nothing at stake. Forcing it onto the nearest neighbour would have inflated the one number the
framework treats as evidence, which is the failure mode of recording recurrences at all.

**Enforced by: NOTHING YET — exposed.** `status = active`, so the instruction layer is the only
thing holding it, and the rule's own doctrine says an unheld class must say so rather than name a
control that does not exist. What *is* mechanical: `relearn check` parses it (84 validated) and
`relearn lint` will report it as an unheld recurrence the first time it fires. The obvious
graduation — a check that every hook or script emitting a warning either exits non-zero or is
named in a reviewed register — is carried in `TODO.md`, unbuilt, and lives in stochos-lab rather
than here because that is where the signals are.

**Found by the gate on landing:** `tests/pack_counts.rs` refused the change twice over — the
copilot pack README claimed 83 rules against 84 and 2109 lines against 2125, and the claude pack
claimed 30 global rules against 31. Both pack READMEs also carried the sentence *"nothing checks
them"*, false since 2026-09-13 for every count the test reads; repaired in the same change, with
the two that genuinely remain unchecked now named as such. `[R:repair-the-lying-artefact]`

### `list --format tsv`: the corpus answerable as data

What: `relearn list --format tsv` emits one tab-separated record per rule — `tag`, `home`,
`created`, `origin`, `status`, `controls`, `recurrences`, `last_recurrence` — under a
`#`-prefixed header naming every column, so a consumer finds a field by name and a column
added later shifts nobody's `cut -f`. `lines` remains the default and is unchanged.

**Why it exists.** stochos-lab's rules ledger was maintaining its own hand-written list of
which rules exist, inside `scripts/mine-rules.sh`. It had drifted to **23 of 84** with
nothing failing, and five rules that a real control already holds — `no-unwrap-in-production`,
`no-anyhow-in-libraries`, `role-is-an-edge-property`, `seeded-data-needs-a-migration`,
`verify-the-glyph-exists` — were invisible to the quarterly review that decides what gets
retired. A second list of the corpus is the P2 violation this tool exists to prevent, and the
only way to retire it is to make the corpus answerable as data. `[R:search-before-you-build]`

**No free prose travels, and that is a decision rather than an omission.** `incident` is a
verbatim quotation from a private working session, so piping it into another repository's
generated artefact carries names past the gate holding them (`[R:names-travel-with-the-quote]`).
`title` is excluded on the same principle of carrying only what the consumer needs as *data*.
The human précis beside a ledger row stays hand-authored in that ledger, where its author can
see it.

**Enforced by:** `ListFormat` (a two-variant enum matched exhaustively, so a third format
cannot compile until it decides what it prints) + `cli::status_columns` (exhaustive over
`Status` with no catch-all — `Partial` reports its controls *without* claiming the whole class,
which a two-state reading would flatten into `graduated` and tell a reviewer the class is
covered) + `cli::tsv_field`, which refuses a value containing a tab, newline or carriage
return rather than escaping or stripping it: TSV has no escape, so such a value invents a
column and the consumer reads every later field shifted with nothing failing
(`[R:parse-dont-validate]`) + `cli::tsv_row`, a pure function of the rule so the format is
pinned without capturing stdout.

**Unit pins:** `the_header_names_exactly_as_many_columns_as_a_row_has_fields` (the header is
the consumer's contract; a one-column drift is silent and total),
`an_active_rule_names_no_controls_and_no_recurrence` (an absent date is empty, never a fake
one), `a_graduated_rule_carries_the_control_that_claims_its_class`,
`a_partial_rule_reports_its_controls_without_claiming_the_whole_class`,
`last_recurrence_is_the_latest_date_not_the_first_written` (fixture ordered oldest-last on
purpose, so `.first()` or file order would pass on tidy input and be wrong here),
`a_control_holding_a_tab_is_refused_rather_than_written`,
`the_tsv_header_is_printed_even_when_no_rule_matches` — an empty table and no output are
different facts and only one is a bug in the consumer.

**Observed through the production path (2026-09-19):** `relearn list --format tsv` against the
real corpus → 84 records under one header, the five Layer-1 rules each carrying the control
that holds them.

## Deliberately out of scope for v0.1

- **Recurrence / outcome instrumentation** — phase C; lives in the stochos-lab ledger, not here.
- **Importing existing rules from Cursor/Copilot formats** — reverse direction; only if a real need appears (P7: verify the target exists).
