# TODO.md — relearn

*Contains every `NOTHING YET — exposed` gap from FEATURES.md until closed, plus discovered work.*

## Phase 0 — scaffold
- [x] Five project files authored (CLAUDE, ARCHITECTURE, FEATURES, TODO, README)
- [x] Relocated to `C:\Users\ganak\Documents\relearn` (the live Windows dev area, beside win-health-mcp/mesh-watchdog/[redacted]-Architecture). NB: the named "dev root with Ferridis/sysmand/linux-health-mcp" was `D:\linux-george`, a stale Linux-home backup — see 2026-08-13 note.
- [x] `cargo init` (bin + lib, edition 2024); module skeleton per ARCHITECTURE; `git init`; **private** GitHub repo `george-andrikopoulos/relearn`
- [x] CI: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` on push + PR

## Phase A — portability (v0.1)

### Types first (before any emitter)
- [x] `RuleTag` newtype, `parse` only, shape `R:[a-z0-9][a-z0-9-]*`
- [x] `Home` sum type: `Global | Domain{name} | Project{path}` (+ `DomainName`/`ProjectPath` non-empty newtypes)
- [x] `Status` sum type carrying payloads: `Active | Graduated{to} | Attic{reason,date}` (+ `Destination`/`Reason` newtypes)
- [x] `Incident`, `ErrorClass` newtypes (non-empty by construction, over a shared `nonempty` perimeter)
- [x] Date field: parse wide, then range-check (dogfood `[R:parse-wide-then-range-check]`) — `Date::parse` into `i64` then narrow; property-tested
- [x] `Library<Unvalidated>` / `Library<Validated>` typestate; `validate` is the only path to `Validated` (`emit` will accept only the latter)

### Parsing
- [x] TOML front-matter reader (`+++` delimited) → `Rule` (`rule::parse_document`; dates are quoted strings so `Date::parse` keeps the range check)
- [x] Parse failure names the file and field, never skips (`ParseError` + `fsio::LoadError::Parse`)
- [x] Duplicate-tag detection at library level (`Library::validate` → `ValidationError::DuplicateTag`)

### Emitters (pure functions returning `Vec<OutputFile>`)
- [x] Shared emit types: `OutputFile` (path + contents), `RelativePath` (portable forward-slash, `pub(crate)` construction), `HomeSlug` (`Home` → `[a-z0-9-]+`, also the skill `name`) — `emit.rs`
- [x] `emit` shared scope helper: `Home` → (globs, alwaysApply) via a domain→file-pattern table (`rust → **/*.rs`, …); read by `cursor` (and later `copilot`) so scope translation is one place (`emit::Scope::for_home`)
- [x] `claude` — one skill **per home layer**: `skills/<home-slug>/SKILL.md`, description aggregating the home's rules (YAML-quoted); makes the `Library<Validated>` typestate load-bearing (`emit::claude::emit`)
- [x] `cursor` — `.cursor/rules/<tag-body>.mdc`; `globs`/`alwaysApply` from `Scope::for_home`, not the rule; lenient (unquoted) front-matter, colon-free filename via `RuleTag::body` (`emit::cursor::emit`)
- [x] `copilot` — `.github/copilot-instructions.md` (all rules, home-rank ordered; `emit::copilot::emit`)
- [x] `agents` — `AGENTS.md` (all rules, home-rank ordered; `emit::agents::emit`)
- [x] `claude_md` — project-layer `CLAUDE.md` (project-home rules only; no file when none; `emit::claude_md::emit`)
- [x] Shared `emit::home_rank` (general→specific home ordering for the concatenated emitters)

### fsio + guards
- [x] Read side: load `rules/*.md` (sorted) into `Library<Unvalidated>`, file-named diagnostics (`fsio::load_rules`)
- [x] Generated-by header + content hash on every emitted file (`fsio::write_all` appends marker + version + source tags + `sha256`)
- [x] Overwrite guard: pre-flight marker check, all-or-nothing abort; refuse to clobber a file lacking the header (`fsio::write_all` → `WriteError::WouldClobberUnversioned`; `[R:generate-guards-unversioned]`)
- [x] Detect a hand-edited generated file (recompute body `sha256`, compare to the header's) — **done via `relearn verify`** (`fsio::verify_all`): classifies each generated file `Ok`/`Missing`/`Unversioned`/`HandEdited`/`Stale` and exits non-zero on drift. Integrity (body-hash vs own header) checked before freshness (vs a fresh emission), so a hand edit is never mislabelled stale. The marker guard *prevents* clobbering; this *detects* drift — complementary. `build`/`verify` share `cli::emit_selected`.
- [ ] **Orphan detection in `verify`** (follow-up): a generated file on disk whose rule was deleted still lingers; `verify` currently only checks the *expected* set, so it won't flag an orphan. Needs a directory-walk policy (which subtree relearn "owns") to avoid false positives — deferred until that policy is decided.

### CLI
- [x] `check`, `build --targets`, `list --home` (`cli::run` → `check`/`build`/`list`; `CliError` wraps `LoadError`/`ValidationError`/`WriteError`; `main` is a thin `ExitCode` shell)
- [x] Unknown target rejected before any write (`Target` value-enum names only buildable emitters, so an unknown `--targets` value is a `clap` parse error)

### Tests (per `rust-typedd` hierarchy — types first, then properties, then pins)
- [x] proptest: emission idempotent over generated rule sets (`tests/properties.rs::emission_is_idempotent`, all 5 emitters)
- [x] proptest: round-trip preserves tag, home, status, body (`tests/properties.rs::neutral_round_trip_preserves_the_rule`, via the new `rule::to_document` serializer)
- [x] compile-fail test: unvalidated library cannot reach `emit` (`trybuild` pin `tests/compile_fail/emit_rejects_unvalidated_library.rs` — checks the type mismatch, not merely that it fails; **closes Phase A**)
- [ ] unit pins: one per bug found, added with the fix
- [ ] **Discovered 2026-08-13 (TDP scan):** the `parse`-style constructors returning `Result` lack `#[must_use]` (18 sites across `rule::{text,date,home,tag,status,parse}`, `library::validate`, `fsio::{load_rules,write_all}`). A caller who ignores the `Result` silently drops a validation failure — exactly what `#[must_use]` prevents (George's rule: must_use on every `Result`-returning fn). Codebase-wide, pre-existing; a separate mechanical sweep, not folded into the emit-status change. (New emit-status fns already carry it.)

### Seed content
- [x] Migrate a first real rule set into the neutral format (`rules/*.md`) — **the first real test of format adequacy**. The schema (tag / title / error_class / home / created / status / incident / body) held every rule with no missing field.
- [x] Grow the corpus to **10 rules** (added `parse-dont-validate`, `no-unwrap-in-production`, `no-anyhow-in-libraries`, `make-illegal-states-unrepresentable`). Now exercises: `graduated` status on real data (`no-unwrap`/`no-anyhow` graduated to the actual `no-unwrap-in-src`/`no-anyhow-in-lib` hooks); multiple rules per non-global home (4 in domain-rust); and real cross-references (`parse-dont-validate`→`parse-wide`, `illegal-states`→`no-sentinel-values`, both resolving to active — the reference checker runs clean on real data). Homes: 5 global, 4 domain-rust, 1 project-relearn. `check`/`lint`/`build` all green (16 emitted files); contradiction re-review clean.
- [x] Grow the corpus to **14 rules** (added `prefer-by-construction`, `generate-guards-unversioned`, `verify-tracked-after-move`, `no-secrets-in-config-repo`). All four carry `[R:...]` tags that already exist in George's live instruction corpus — `prefer-by-construction` (global) and the two stochos-lab tags from that repo's CLAUDE.md, plus `generate-guards-unversioned` which relearn's own FEATURES/TODO already cite. Adds a **second project home** (`project-stochos-lab`) and one resolving cross-reference (`prefer-by-construction`→`make-illegal-states-unrepresentable`). Homes: 6 global, 4 domain-rust, 2 project-relearn, 2 project-stochos-lab. `check`/`lint`/`build`/`verify` all green (21 emitted files); contradiction re-review clean.
- [ ] Provenance note: the first 6 rules are single-incident-dated; later batches mix single-incident with **codification-dated** provenance. This latest batch ports **already-tagged real rules** from George's live instruction layer (the tag string is not invented — it exists in his corpus); the `created` date is the codification-into-this-corpus date (2026-08-13), with the genuine source named in each `incident`. No fabricated incidents, no invented tags.
- [ ] Still no real **attic** rule in the corpus (only `graduated`) — the retired-reference *attic* path stays unit-test-only until a rule is genuinely retired.
- [ ] Port the remaining `[R:...]` rules as their incidents are on hand (more project-discipline / stochos-lab / Rust layers).

### Open design question surfaced by the graduated rules
- [x] Should `emit` **filter by status**? **Decided 2026-08-13 (George): emit `active` + `graduated`, suppress `attic`; graduated is emitted *annotated* with its destination.** Rationale: the instruction layer *tunes* generation before the fact, the graduated-to hook *catches* after — distinct controls, both raising first-time-right, so a graduated rule keeps its tuning job (annotated so a reader knows a stronger control also holds it); a retired rule must never leak into an active instruction file. Implemented as `Status::emittability` (exhaustive match → a new status must decide its policy) applied once in `emit::emittable`, with `emit::graduation_note` for the annotation. Property-tested whole-space (`atticked_rules_never_leak_into_any_emitter`, `active_and_graduated_rules_all_reach_copilot`) + unit pins; FEATURES row "Emission respects rule status". Verified through the binary: the two seed graduated rules render `> Also enforced by hook:…` in all four target formats. (Attic *suppression* is proven by property + unit, not the binary — no honest attic rule exists in the corpus to feed it, and fabricating one is out.)

## Phase B — linter (functionally complete; `lint` module + `relearn lint` + `relearn verify`, advisory/read-only)
*Only open item is cold-surface, which is blocked on Phase-C runtime data — relocated to Phase C below where its data lives.*
- [x] Overlapping-scope detection (same error class, case-insensitive) — `lint::overlapping_scope`
- [x] Home-slug collision detection: two *distinct* homes whose `HomeSlug` collides would silently share one skill file — flagged `Error`-severity (not merged, not deleted). `lint::home_slug_collisions`. Closes the 2026-08-13 TDP-scan edge.
- [x] Dangling references (rule cites an `R:...` tag absent from the library; `OR:`/`FOR:` in prose excluded; a tag cited in body **and** incident is one finding) — `lint::reference_checks`
- [x] Retired references (rule cites a rule that exists but is atticked/graduated) — flagged `Info`; `lint::reference_checks`
- [x] `relearn lint` CLI: advisory, writes nothing; exit non-zero only on `Warning`/`Error` (`Info` informs without failing CI); clean on the seed
- [x] Contradiction detection — **resolved as a process-control**, not a code check: a periodic Claude review pass over the corpus (semantic judgment, delegated per doctrine; a keyword heuristic would be dishonest). **First review 2026-08-13: no contradictions among the six seed rules, no problematic overlap, homes consistent. One scoping observation — `parse-wide` (domain rust) and `order-by-explicit-rank` (project relearn) express principles that generalize beyond their homes; promote only if the class recurs elsewhere.** Re-run each time the corpus changes materially.
- [x] `relearn verify` — read-only drift detection: each generated file classified `Ok`/`Missing`/`Unversioned`/`HandEdited`/`Stale`, non-zero exit on any drift (`cli::verify` → `fsio::verify_all`). The CI "committed generated tree is in sync with the rules" gate; complements the write-side clobber guard. (Was the Phase A hand-edit-detection item; landed here as it is a checking feature.)
- [→] Cold-surface / uncited report — **relocated to Phase C** (see below): "cold" needs runtime invocation data (stochos-lab observability), which does not live in this repo; "uncited" alone is noise (a standalone rule is legitimately uncited). Not buildable here without faking a signal.

## Phase D — public release (gated on arXiv ID)
- [ ] README public framing: versioned instruction artifacts + the governance loop
- [ ] Licence (Apache-2.0, matching Ferridis — includes the patent grant)
- [ ] Worked example: one incident → one rule → five emitted formats
- [ ] Point at the arXiv identifier; make the repo public

## Phase C — instrumentation (parallel; lives in stochos-lab, not here)
- [ ] Error-class recurrence — partly exists in the ledger
- [ ] First-time-right capture on AI-assisted work
- [ ] **Cold-surface / uncited report** (moved from Phase B): flag rules that runtime data shows are never invoked — candidates for the attic cut-list. Needs skill-invocation / hook-fire counts from the stochos-lab observability layers; once that feed exists, the report itself can live either here or as a relearn lint check fed by an exported dataset.
- [ ] *(Deferred, needs a field site: rework rate, time-to-competence — study-design items, not build items)*

## Resolved decisions (2026-08-13 — see ARCHITECTURE decisions log)
- [x] Claude skill emitter: **one skill per home layer**, not per rule. Bounds the always-resident metadata index (P6); matches the existing `project-discipline`/`rust-typedd` skills that bundle rules by domain. Future escape hatch: an optional `skill_group` field *only if* one home ever needs more than one skill.
- [x] Cursor `globs`/`alwaysApply`: **derived from `Home`** via a shared domain→pattern table in `emit`; no `cursor.*` fields on the rule (keeps the format neutral, P2). If finer scope than `Home` expresses is ever needed, enrich `Home` so every emitter benefits — never a Cursor-only field.
- [x] **Emit-status filtering: suppress `attic`, emit `active` + `graduated` (annotated).** Policy is one exhaustive match (`Status::emittability`) applied once (`emit::emittable`); annotation via `emit::graduation_note`. Graduated kept because instruction-layer tuning (before generation) is a control distinct from the graduated-to hook (catches after); attic suppressed because withdrawn guidance must never enter an active instruction file. Whole-space property tests + unit pins; FEATURES row "Emission respects rule status".
