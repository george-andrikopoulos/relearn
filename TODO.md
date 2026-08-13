# TODO.md — relearn

*Contains every `NOTHING YET — exposed` gap from FEATURES.md until closed, plus discovered work.*

## Phase 0 — scaffold
- [x] Five project files authored (CLAUDE, ARCHITECTURE, FEATURES, TODO, README)
- [x] Relocated to `C:\Users\ganak\Documents\relearn` (the live Windows dev area, beside win-health-mcp/mesh-watchdog/[redacted]-Architecture). NB: the named "dev root with Ferridis/sysmand/linux-health-mcp" was `D:\linux-george`, a stale Linux-home backup — see 2026-08-13 note.
- [x] `cargo init` (bin + lib, edition 2024); module skeleton per ARCHITECTURE; `git init`; **private** GitHub repo `george-andrikopoulos/relearn`
- [x] CI: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` on push + PR

## Phase A — portability (v0.1)

### Types first (before any emitter)
- [ ] `RuleTag` newtype, `parse` only, shape `R:[a-z0-9][a-z0-9-]*`
- [ ] `Home` sum type: `Global | Domain{name} | Project{path}`
- [ ] `Status` sum type carrying payloads: `Active | Graduated{to} | Attic{reason,date}`
- [ ] `Incident`, `ErrorClass` newtypes (non-empty by construction)
- [ ] Date field: parse wide, then range-check (dogfood `[R:parse-wide-then-range-check]`)
- [ ] `Library<Unvalidated>` / `Library<Validated>` typestate; `emit` accepts only the latter

### Parsing
- [ ] TOML front-matter reader (`+++` delimited) → `Rule`
- [ ] Parse failure aborts with file + field in the diagnostic; never skip
- [ ] Duplicate-tag detection at library level

### Emitters (pure functions returning `Vec<OutputFile>`)
- [ ] `claude` — `skills/<name>/SKILL.md`
- [ ] `cursor` — `.cursor/rules/<tag>.mdc`
- [ ] `copilot` — `.github/copilot-instructions.md`
- [ ] `agents` — `AGENTS.md`
- [ ] `claude_md` — project-layer `CLAUDE.md`

### fsio + guards
- [ ] Generated-by header + content hash on every emitted file
- [ ] Overwrite guard: refuse to clobber a file lacking the header (`[R:generate-guards-unversioned]`)

### CLI
- [ ] `check`, `build --targets`, `list --home`
- [ ] Unknown target rejected before any write

### Tests (per `rust-typedd` hierarchy — types first, then properties, then pins)
- [ ] proptest: emission idempotent over generated rule sets
- [ ] proptest: round-trip preserves tag, home, status, body
- [ ] compile-fail test: unvalidated library cannot reach `emit`
- [ ] unit pins: one per bug found, added with the fix

### Seed content
- [ ] Migrate the existing rule library (relearn, project-discipline, rust-typedd, the `[R:...]` rules in stochos-lab) into the neutral format — **the first real test of whether the format is adequate**

## Phase B — linter (after format settles)
- [ ] Contradiction detection between rules
- [ ] Overlapping-scope detection (same error class, two homes)
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

## Open questions
- [ ] Does the Claude skill emitter produce one skill per rule, or one skill per home layer? Per-rule may bloat the always-resident metadata index (P6 — context is finite). **Decide before writing the emitter.**
- [ ] Cursor `globs` and `alwaysApply` have no counterpart in the neutral format. Add target-specific optional fields, or derive from `Home`? Deriving is cleaner but may not fit every case.
