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
- [ ] compile-fail test: unvalidated library cannot reach `emit` (`trybuild`; the type already enforces it, this pins the negative)
- [ ] unit pins: one per bug found, added with the fix

### Seed content
- [x] Migrate a first real rule set into the neutral format (`rules/*.md`, 6 rules from dated incidents: `parse-wide-then-range-check`, `no-sentinel-values`, `verify-through-production-path`, `measure-cost-per-task`, `revision-integrity`, `order-by-explicit-rank`) — **the first real test of format adequacy**. Finding: the schema (tag / title / error_class / home / created / status / incident / body) held every rule across all three homes (4 global, 1 domain, 1 project) with no missing field; `check` + `build` produce 12 files, ordering and project-only `CLAUDE.md` correct.
- [ ] Only real rules with documented, dated incidents were ported (fabricating provenance is the sediment the tool fights). Still unexercised by *real* seed data: `graduated` / `attic` status and multiple rules sharing one non-global home — covered by unit tests, awaiting a real rule that carries them.
- [ ] Port the remaining `[R:...]` rules as their incidents are on hand (project-discipline, rust-typedd, stochos-lab layers)

## Phase B — linter (after format settles)
- [ ] Contradiction detection between rules
- [ ] Overlapping-scope detection (same error class, two homes)
- [ ] Home-slug collision detection: two distinct homes whose `HomeSlug` slugifies to the same token (e.g. pathological all-punctuation domain names → `domain-`) would silently share one skill file — a lost rule. Detect and reject; do not merge. (Edge surfaced by the 2026-08-13 TDP scan; realistically unreachable for alphanumeric hand-authored homes, so deferred, not built.)
- [ ] Dangling references (rule cites an artifact that no longer exists)
- [ ] Cold-surface + uncited report (input to the cut list, never auto-delete)

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
