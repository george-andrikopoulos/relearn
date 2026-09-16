# CLAUDE.md — relearn

*The recreation standard: from this file alone, this project could be rebuilt.*

## What this is

`relearn` is a Rust CLI that stores an engineering team's correction-derived rules **once**, in a neutral versioned form, and **compiles** them out to every AI assistant's native instruction layer — Claude skills, Cursor `.mdc` rules, GitHub Copilot instruction files, `AGENTS.md`, plain `CLAUDE.md`.

It is the reference implementation of the error loop described in *Tuning the Stochastic Machine* (George Andrikopoulos, 2026, arXiv:2608.19125) and of the Stochos framework's P1 (persist or perish) and P2 (one home per rule). The precision-measurement companion, *Grouping the Stochastic Machine* (arXiv:2608.19140), references this repository. The third in the series, *Aiming the Stochastic Machine: A Repository Discipline for First-Time-Right, and What Survived Measuring It* (Zenodo, 2026, [doi:10.5281/zenodo.22083202](https://doi.org/10.5281/zenodo.22083202)), specifies the four-file repository discipline this repo is built on and names this repository as its worked case (§8). Its study materials — pre-registration, blinded pack, both codings, `kappa.py`, results — are a separate deposit, [doi:10.5281/zenodo.22082967](https://doi.org/10.5281/zenodo.22082967).

**The problem it solves.** When an expert corrects an AI assistant, the correction dies with the session unless it is written into the instruction layer. But every assistant has its own instruction format, so a rule written for one does not travel — the correction persists but is *locked to a vendor*. `relearn` makes the rule the artifact and the vendor format a build target.

**The failure class upstream of that.** A correction is needed in the first place because the assistant acted on a requirement nobody gave it. Paper 3 names those **invented requirements**, and names their visible residue **skedasis** (σκεδάννυμι, to scatter): the fraction of a change that nobody requested — a ratio, not a size. Two hundred lines is not skedasis if the task needed two hundred; it is skedasis when the task needed three. A persisted rule is how an invented requirement is retired instead of re-invented next session.

## Core design decisions (and why)

1. **One rule = one file, neutral format.** `rules/<tag>.md`, TOML front-matter delimited by `+++`, imperative markdown body after. TOML because the parser is well maintained in Rust; front-matter because the body must stay human-editable and diff-friendly. Rules are the source of truth; every emitted format is a build artifact and is never hand-edited.
2. **Emitters are pure functions of validated rules.** No emitter may read anything the neutral rule does not carry. If a target needs data, the data belongs in the rule.
3. **Type-Driven Design, not test-first** (see `rust-typedd`). Illegal states are made unrepresentable; validation happens once at the perimeter and mints a witness type; interior code never re-checks. Property tests carry behavioural laws; unit tests are regression pins for past bugs.
4. **Provenance is mandatory, not optional.** A rule without a date, a triggering incident, and an error class does not parse. The framework's whole governance argument rests on provenance; making it optional would make the tool complicit in the failure it exists to prevent. **Recurrence is the second observation**, added 2026-09-06: a rule also records, as `[[recurrence]]` tables, each later occurrence of the error class it already covers. The triggering incident says why the rule exists; a recurrence says the rule failed — and recurrence is the only number that tells you whether a rule is working. It is a separate field from `incident` because a recurrence needs its own date where the first incident borrows the rule's `created`; it is optional, so no rule file needed editing when it landed.
5. **Emission is idempotent and destructive-safe.** Re-running the compiler must produce byte-identical output for unchanged rules, and must refuse to overwrite a target file it did not generate (see `[R:generate-guards-unversioned]`).

## Invariants that must never break

- A rule that fails to parse **stops the build** — it is never silently skipped. A silently dropped rule is a correction lost, which is the exact failure the project exists to prevent.
- No emitted artifact is a source. Anything under an emitter's output path may be regenerated at any time and must never be hand-edited.
- Every rule has exactly **one home** (`Home::Global | Org | Domain | Project`). Two homes for one rule is unrepresentable in the type system, not merely discouraged.
- **An `Org`-homed rule never leaves the machine**, and that is a match rather than a filter:
  `Home::federation` decides every variant with no catch-all arm, so adding a home without deciding
  its federation behaviour is a compile error. A filter in one publishing path compiles perfectly
  while a second path publishes everything. `Project` is withheld too — its home carries a
  filesystem path, which is a private identifier.
- **An aggregate is never authoritative, and it prints its own confounds.** No build behaves
  differently for having seen one — asserted by reading the emitters' source, because nothing else
  can see a build that quietly consults a downloaded file. The k-floor is applied where the counts
  are, from the same constant the producer reads, and below it a rule does not appear at all: the
  tag points at the same person as the count.
- **A recurrence report is anonymous, always.** `Observation` is the field list — five fields, so a
  sixth cannot be added in a renderer — and `Month` has no day field, so a day-level date cannot
  be reintroduced by an edit. Only an upstream tag is reportable: a local-only tag is a private
  name. The pseudonym lives in the cloned aggregate repository, never on the machine.
- **The raw `incident` cannot reach a contribution.** `Contribution` is a projection that never
  borrows `incident` or the recurrences — not stripped, never held — and what travels is
  `published_incident`, a separate authored field of its own type. Nothing derives one from the
  other: an automatic scrubber leaks what it did not recognise and stops people reading the
  output. `contribute` prints and writes nothing without `--confirm`, runs the banned-terms
  matcher over what a contributor authored, and reports a location and a length — never the match.
- **A rule travels in both directions, and each direction has its own witness.** `contribute`
  publishes one into a directory both installs can see — a common drive — at a **stated
  revision**, which is required because a cache records the revision it holds and a publication
  without one produces a copy that can never be told it is stale. `pull` is the receiving half
  and the only thing that creates a cache: a rule arrives because somebody asked for it by tag,
  never by sync, and it keeps the home it arrived with, so it compiles into the receiving
  install's layer exactly like a local rule. `Home` says which layer; `Authority` says who
  maintains it; the two are independent and that independence is what makes caching worth doing.
  `pull --all` does the same over a whole shared corpus and is **a report before it is an
  action**: without `--confirm` it is the status check to run before starting work, naming every
  rule in both corpora exactly once — including what it will not touch, and why.
- **Only a cache may be deleted, and that is the whole safety argument for `--prune`.** A cache is
  regenerable, so dropping one loses nothing a later pull cannot restore; a rule this install owns
  and a fork it took are source that nothing regenerates. `DroppableCache` refuses both, so
  `fsio::remove_cache` — the only path that deletes a rule file — cannot be reached with source.
  Dropping is opt-in, an unwanted cache is still reported under the default, and a drop of a rule
  **retired** upstream says before it happens that it has no inverse.
- **A cached rule is never edited in place.** `Authority::Local | Adopted | Cached`; `fsio::write_rule`
  takes an `EditableRule` witness whose only constructor refuses a cache, so a write path added
  later cannot reach the filesystem without asking. The one legitimate writer of a cache,
  `fsio::write_cache`, therefore takes a **second** witness (`PulledRule`) rather than a flag:
  neither witness can be minted for the other's subject, so "edit a cache in place" stays
  unconstructible rather than becoming reachable by passing `true`. Editing a cache is a silent fork — the edit
  succeeds and nothing records the divergence. `adopt` is the loud one, and it remembers what it
  forked from. `Version` is totally ordered so "is this cache stale?" always has an answer — and
  the other half of that question is `Authority::Local`'s optional revision, which is how a
  published rule says which revision it *is*. One field, one meaning, so a cached file never
  carries the number twice; absent, no cache of it can be told it is stale and nothing guesses.
- **A rule can be *partly* held, and the status must then say what is not held.** Controls
  routinely cover part of an error class and prose covers the rest; with only `Active` and
  `Graduated`, recording that honestly was impossible, and the dishonest option was worse than
  silence — `Graduated` prints "Also enforced by X" into five instruction layers for ground X
  never claimed, and makes the next recurrence there a CI failure. `Status::Partial` carries
  `uncovered` as a field of the variant, so a partial graduation that will not name the gap is
  unconstructible. It reports like `Active` and annotates like `Graduated`, and those two halves
  are decided by `prose_coverage` and `whole_class_claim` — exhaustive matches, so a new status
  variant cannot compile until both are answered. Recording the truth must never turn a warning
  into a build failure, or the honest status becomes the one nobody uses.
- **A poke is news, never a verdict.** The federation's signal is surfaced inside `lint`, carries
  no severity, and is printed after the exit code has been decided — a signal from strangers that
  could fail CI would have made federation required. One trigger is reactive (a rule fired *here*,
  and upstream covers that class) and is never capped; the other four are broadcast, capped at a
  number the operator passes, and two of them are off until named. An upstream **retirement**
  warns and can do no more: a local attic suppresses emission, an upstream one never does,
  because deleting an instruction a team relies on because a stranger retired it is a correction
  lost with no reader. No clone, no pokes, and no
  warning about their absence. `build` cannot reach the module at all, asserted by reading the
  emitters' source: `lint` reports and `build` emits.
- **A mandate carries its approval, and is not evidence.** `Origin::Mandated(Approval)` makes a
  mandate with no signer and an approval on a mined rule both unconstructible, and mandated rules
  are held out of the recurrence statistics entirely — they were never mined, and they are not
  *inert* either, because inert is a judgement about something that was meant to be evidence.
- **`applies_to` is not a second home.** Home answers *who owns and maintains this rule* and has
  one answer; `applies_to` answers *who should load it* and may have several — a low-latency rule
  that Rust and Java engineers both need lives in one file, with one tag and one incident, and is
  compiled into both language builds. A scope is an **audience** (`rust`, `java`, `embedded`),
  never a topic (`performance`, `security`) — a topic is what `error_class` already carries.
  Scope decides *whether* a rule is emitted and says so in the body; `Home` alone decides
  *where*, and `scope_never_reaches_an_emitted_path` is what keeps that true. A scoped rule
  announces its audience — `> Written for the rust and java audiences.`, the same blockquote
  shape as the graduation and recurrence notes — because a reader of
  `skills/domain-low-latency/SKILL.md` otherwise cannot tell that the rule in front of them
  was written for someone in particular (decided 2026-09-14). That announcement is the *only*
  channel from `applies_to` into emitted text, and
  `scope_reaches_a_body_only_through_the_audience_note` is what keeps *that* true: it deletes
  the note from a scoped build and requires what is left to be byte-identical to an unscoped
  one. **A rule declaring no
  `applies_to` is emitted under every audience**, so adding a scope to one rule can never remove a
  different rule from an existing build — the first invariant above, one layer up.
- Tags are unique across the library; a duplicate tag is a parse error.
- **One engineer with nothing else present can use it.** No account, no configuration file, no
  network, no ambient state: the input is the path given on the command line and the output is the
  path given on the command line, so the same corpus compiles identically on any machine. The
  federation design (`docs/federated-relearn.md`) is a feature of the *corpus* and never a
  dependency of the *compiler* — contribution and reporting are a person running `git`, which is
  why its aggregate is a repository rather than a service. `tests/solo_mode.rs` fails the build on
  a socket, a spawned process, an ambient read, or a networking crate anywhere in the tree.
- Every dependency is priced before it is used: a row in `ARCHITECTURE.md`'s decisions log whose **Decision** cell opens with `Dependency:` (or `Dependencies:`) and names the crate in backticks. `tests/dependencies.rs` fails the build otherwise, and the nine crates that predate the rule are exempted by name in a list that may only shrink. The other half — recording a dependency *refused* — has no artifact and therefore no gate; it is on the person who refused it. `[R:price-every-dependency]`

## Build, run, test

```bash
cargo build --release
cargo test                     # unit + property tests
cargo clippy -- -D warnings
cargo fmt --check

bash scripts/verify-dependencies.sh   # the dependency gate alone (a door onto `cargo test`)

relearn build --targets claude,cursor,copilot,agents   # compile rules to all targets
relearn check                                          # validate library, no output written
relearn list --home global                             # inspect
relearn lint --upstream <drive>                        # findings, then the federation's pokes

# The federation flows. Every one of them prints first and writes only on
# --confirm, none of them transmits anything, and <drive> is a directory both
# installs can see — given on the command line like every other input.
relearn pull --upstream <drive> --all --from <name> --on <date>   # the status check
relearn pull --upstream <drive> --all --prune --from <name> --on <date> --confirm
relearn contribute --tag R:x --terms <list> --version <n> --out <drive>/rules
relearn adopt --tag R:x --on <date>                    # the loud fork of a cache
relearn report --aggregate <clone> --generated YYYY-MM # anonymous recurrence counts
relearn aggregate --clone <clone> --generated YYYY-MM  # the scheduled recompute
```

## Definition of done — runs on EVERY feature or fix

Binding for this repository. A change is not complete until all five pass, in order:

1. **Code and its enforcing artifact ship together.** The type, property test, or unit test that locks the new behaviour is in the same change — never "tests later."
2. **Regression pass.** Re-read `FEATURES.md`; run the enforcing artifacts of every feature the change could plausibly touch. **A fix that breaks another documented feature is not a fix.** This is the control that keeps the ledger true: features are only protected from each other if the enforcing artifacts are actually run.
3. **`FEATURES.md` updated** — a new entry with its enforcing artifact, or an existing entry's artifact revised. A feature with no enforcing artifact is recorded as `NOTHING YET — exposed` and carried in `TODO.md`.
4. **`CLAUDE.md` / `ARCHITECTURE.md` sync.** Ask explicitly: could this file still recreate the project? Does the architecture still describe it? Any design decision made goes into the decisions log with its *why*. Skipping this check is how the recreation standard becomes a lie.
5. **`TODO.md` updated** — done items cleared, discovered work added.

State the five results briefly at the end of any task. If a request conflicts with the discipline ("just patch it quickly"), do the patch, then say which checks were skipped and what exposure that creates — never silently drop it, never block the user with ceremony.

*This is the project-layer application of `[R:definition-of-done-every-change]`, whose general form is in `rules/` and reaches every emitted layer as of 2026-08-25. It stays written out here rather than reduced to a pointer because the five checks above are **specialised** to this repository — check 2 names `FEATURES.md` and its enforcing artifacts, check 4 names `ARCHITECTURE.md`'s decisions log — and a charter that recreates the project has to state what a contributor must actually do. The original justification for writing it here (that an external contributor cannot load the author's `project-discipline` skill) has partly expired: the generic rule is now public in this repository too. What has not expired is the specialisation. If the two ever disagree, `rules/` is the source and this section is the derived text — see TODO.md, where the residual duplication is carried as George's call rather than resolved unilaterally.*

## Design discipline: Type-Driven, not test-first

Standing and deliberate (`rust-typedd`). A test samples points of the behaviour space; a type constrains the whole space and the compiler proves it everywhere, at compile time, forever. In a codebase written with AI assistance, the type system is the only deterministic, whole-space checker available — a retirement stage in silicon. Order of application:

1. **Types** — structure, states, boundaries, invariants (illegal states unrepresentable; parse, don't validate; parse wide then range-check).
2. **Property-based tests** — behavioural laws over generated input spaces (`proptest`).
3. **Unit tests** — regression pins for past bugs, documentary examples, edge cases neither of the above can express.
4. **Prose** — only what none of the above can hold; anything held only in prose is flagged as unenforced.

Unit testing is **not** removed — it is demoted to the layer where it is the right tool. Every bug fix ships its pinning test (check 1 above). Never delete a failing test to make a change pass.

## Standing rules referenced by this repo

Rules cited by tag elsewhere in these files **now live in `rules/`** and are compiled out to every emitted layer. This section used to restate six of them in prose, "scheduled for migration into `rules/` in Phase A, at which point this section points there instead." That migration completed 2026-08-25 and this is the pointer it promised: run `relearn list` for the corpus, or read `rules/<tag>.md` for any single rule with its full provenance.

Keeping the prose copies would have made this file a second home for six rules that have one — the P2 violation this tool exists to prevent, in the charter of the tool that prevents it. The last two to migrate were `[R:wired-artifact]` and `[R:revision-integrity]`; the corpus had been citing both for weeks without defining either.

## The generated project layer

The `claude-rules` target emits **`.claude/rules/<home-slug>.md`**, one file per project home — *not* the repository-root `CLAUDE.md`, which is this hand-authored charter. That separation is deliberate and was a defect until 2026-08-22; `ARCHITECTURE.md`'s decisions log carries the full account. This file pulls the generated layer in:

@.claude/rules/project-relearn.md

## This repository is a publication surface

A rule's `incident` field is a **verbatim quotation from a private working session**, and `TODO.md` narrates the same material. Provenance is mandatory here for good reasons, and a narrative is specific by nature — which means the field that makes a correction durable is also the one that carries private identifiers out of the repository they came from. That repository's own detector cannot help: a gate scans its own tree, stays green, and the quotation travels without it. `[R:names-travel-with-the-quote]`

Install the gate once per clone:

```bash
git config core.hooksPath .githooks     # pre-push -> scripts/no-banned-names.sh
```

It reports **location and count, never the term**, and exits **2 rather than 0 when it has no list** — a disarmed gate is not a pass. The term list is deliberately **not committed**: this repository is public and one protected name normalises to four characters, so publishing the salt would publish the name. It lives at `$BANNED_TERMS_FILE`, or `~/.claude/usage/banned-terms.sha256`. CI therefore cannot run this gate and is not wired to it; FEATURES.md records it as configuration-dependent rather than claiming a CI gate it does not have.

Before writing an incident, ask: **which repository's detector covers the file I am about to write into?** If the answer is the repository the quotation came *from*, nothing covers the destination.

## Where the rest is

- `copilot-pack/` — a committed **second emission** of the copilot target plus a hand-authored install README, so the folder can be downloaded and dropped into an unrelated repository whole. Generated, never transcribed: rebuild with `relearn build --targets copilot --out copilot-pack`, and CI runs the matching `verify` because the pack lies outside every relearn-owned path and bare `verify` cannot reach it.
- `claude-pack/` — the same idea for the `claude` (Skills) target: one installable skill folder per home layer, plus a README covering a Claude Code project (`.claude/skills/`), a whole machine (`~/.claude/skills/`), and claude.ai. Rebuild with `relearn build --targets claude --out claude-pack`; CI verifies it as a second output root. A skill is only installable if its `description` fits Claude's 1024-character cap, which is why that field is bounded by the `SkillDescription` type rather than assembled inline.
- `docs/session-start-poke.md` — the `SessionStart` hook snippet for the poke, **deliberately not installed**: anything under `~/.claude` edits the layer loaded into every session on the machine, and no gate in this repository could see it.
- `ARCHITECTURE.md` — modules, data flow, the decisions log.
- `FEATURES.md` — the regression ledger; every feature names the artifact that enforces it.
- `docs/worked-example-two-installs.md` — the same rule published, taken, revised, refreshed and forked between two installs, with real output. The corpus it describes is real: [`relearn-corpus`](https://github.com/george-andrikopoulos/relearn-corpus), public, five rules.
- `TODO.md` — open work, including every `NOTHING YET — exposed` gap from FEATURES.md.
- `README.md` — the public face.

## Domain rules referenced, not duplicated

This project is governed by the author's standing skills: `rust-typedd` (design discipline), `project-discipline` (these five files and the definition of done), `relearn` (the error loop itself). **Do not copy their content into this repo** — reference them. Duplicated domain rules fork and rot (P2).
