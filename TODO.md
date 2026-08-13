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
- [ ] Detect a hand-edited generated file (recompute body `sha256`, compare to the header's) — deferred; the marker guard already prevents clobbering human files

### CLI
- [x] `check`, `build --targets`, `list --home` (`cli::run` → `check`/`build`/`list`; `CliError` wraps `LoadError`/`ValidationError`/`WriteError`; `main` is a thin `ExitCode` shell)
- [x] Unknown target rejected before any write (`Target` value-enum names only buildable emitters, so an unknown `--targets` value is a `clap` parse error)

### Tests (per `rust-typedd` hierarchy — types first, then properties, then pins)
- [x] proptest: emission idempotent over generated rule sets (`tests/properties.rs::emission_is_idempotent`, all 5 emitters)
- [x] proptest: round-trip preserves tag, home, status, body (`tests/properties.rs::neutral_round_trip_preserves_the_rule`, via the new `rule::to_document` serializer)
- [x] compile-fail test: unvalidated library cannot reach `emit` (`trybuild` pin `tests/compile_fail/emit_rejects_unvalidated_library.rs` — checks the type mismatch, not merely that it fails; **closes Phase A**)
- [ ] unit pins: one per bug found, added with the fix

### Seed content
- [x] Migrate a first real rule set into the neutral format (`rules/*.md`, 6 rules from dated incidents: `parse-wide-then-range-check`, `no-sentinel-values`, `verify-through-production-path`, `measure-cost-per-task`, `revision-integrity`, `order-by-explicit-rank`) — **the first real test of format adequacy**. Finding: the schema (tag / title / error_class / home / created / status / incident / body) held every rule across all three homes (4 global, 1 domain, 1 project) with no missing field; `check` + `build` produce 12 files, ordering and project-only `CLAUDE.md` correct.
- [ ] Only real rules with documented, dated incidents were ported (fabricating provenance is the sediment the tool fights). Still unexercised by *real* seed data: `graduated` / `attic` status and multiple rules sharing one non-global home — covered by unit tests, awaiting a real rule that carries them.
- [ ] Port the remaining `[R:...]` rules as their incidents are on hand (project-discipline, rust-typedd, stochos-lab layers)

## Phase B — linter (started; `lint` module + `relearn lint`, advisory only)
- [x] Overlapping-scope detection (same error class, case-insensitive) — `lint::overlapping_scope`
- [x] Home-slug collision detection: two *distinct* homes whose `HomeSlug` collides would silently share one skill file — flagged `Error`-severity (not merged, not deleted). `lint::home_slug_collisions`. Closes the 2026-08-13 TDP-scan edge.
- [x] Dangling references (rule cites an `R:...` tag absent from the library; `OR:`/`FOR:` in prose excluded; a tag cited in body **and** incident is one finding) — `lint::reference_checks`
- [x] Retired references (rule cites a rule that exists but is atticked/graduated) — flagged `Info`; `lint::reference_checks`
- [x] `relearn lint` CLI: advisory, writes nothing; exit non-zero only on `Warning`/`Error` (`Info` informs without failing CI); clean on the seed
- [x] Contradiction detection — **resolved as a process-control**, not a code check: a periodic Claude review pass over the corpus (semantic judgment, delegated per doctrine; a keyword heuristic would be dishonest). **First review 2026-08-13: no contradictions among the six seed rules, no problematic overlap, homes consistent. One scoping observation — `parse-wide` (domain rust) and `order-by-explicit-rank` (project relearn) express principles that generalize beyond their homes; promote only if the class recurs elsewhere.** Re-run each time the corpus changes materially.
- [ ] Cold-surface / uncited report — **deferred**: "cold" needs runtime invocation data (phase C, stochos-lab); "uncited" alone is noise (a standalone rule is legitimately uncited)

## Phase D — public release (gated on arXiv ID)
- [ ] README public framing: versioned instruction artifacts + the governance loop
- [ ] Licence (Apache-2.0, matching Ferridis — includes the patent grant)
- [ ] Worked example: one incident → one rule → five emitted formats
- [ ] Point at the arXiv identifier; make the repo public

## Phase C — instrumentation (parallel; lives in stochos-lab, not here)
- [ ] Error-class recurrence — partly exists in the ledger
- [ ] First-time-right capture on AI-assisted work
- [ ] *(Deferred, needs a field site: rework rate, time-to-competence — study-design items, not build items)*

## Resolved decisions (2026-08-13 — see ARCHITECTURE decisions log)
- [x] Claude skill emitter: **one skill per home layer**, not per rule. Bounds the always-resident metadata index (P6); matches the existing `project-discipline`/`rust-typedd` skills that bundle rules by domain. Future escape hatch: an optional `skill_group` field *only if* one home ever needs more than one skill.
- [x] Cursor `globs`/`alwaysApply`: **derived from `Home`** via a shared domain→pattern table in `emit`; no `cursor.*` fields on the rule (keeps the format neutral, P2). If finer scope than `Home` expresses is ever needed, enrich `Home` so every emitter benefits — never a Cursor-only field.
