# Federation programme — the one place to look

### relearn, A1 through C2 · 13 September 2026

**Purpose.** One document George reviews against, one phase per Claude Code session, and a named
thing that can go wrong at each step so a deviation is visible rather than discovered.

**Design source:** [`docs/federated-relearn.md`](federated-relearn.md) (v2, §12 Decided). **This
document does not restate it** — it sequences it. Where the two disagree, the design document is
the source and this one is wrong. `[R:decisions-log-records-rejected-alternatives]` applies to
both.

**Detailed phase prompts live in their own files and are not duplicated here.** A1 was
`relearn_scope_applies_to_2026-09-13.md`. Later phases get their own when their turn comes.
Restating a phase's design in two places is the P2 violation this tool exists to prevent. *Those
prompt files live in the author's working notes, outside this repository: the reference above
does not resolve here, and is recorded as a pointer to something external rather than a dangling
path.*

> **Filing note, 13 September 2026.** Filed after **A1 had already shipped** (`3798521`), under
> its own prompt and before this document existed. The sequence below is therefore read with A1
> behind it and **Phase 0 next** — an inversion that is recorded rather than tidied away, because
> Phase 0 exists to repair a document A1 has now contradicted in the opposite direction from the
> one anticipated. Three claims were checked against the tree before filing
> (`[R:measure-the-claim-not-a-subset]`): the corpus is **52** rules, not 51, in every place this
> document counted it; **§0.3 was already closed** and its paragraph is struck below with the
> measurement that closed it; §0.1 and §0.2 are live, verified at
> `docs/federated-relearn.md:134` and §5 respectively.

---

## RULE ZERO — read before every session

**One phase per session. Do not build ahead.**

`docs/federated-relearn.md` describes the whole system. It is a design, not a backlog to drain.
A session that lands A1 *and* starts A2 has made the diff unreviewable, and an unreviewable diff
is an unreviewed diff — which is §2 of Paper 3 arriving in this repository's own history.

If a phase turns out to need something from a later phase, **stop and report it**. Do not import
the later phase. The dependency is information; acting on it unasked is an invented requirement.

---

## Standing gates — green at the end of every phase, no exceptions

| Gate | What it holds |
|---|---|
| `cargo test` | Everything below, plus the corpus |
| `relearn verify` | **Emission is byte-identical for every rule the phase did not touch.** The single most important check in this programme |
| `tests/solo_mode.rs` | No socket, no spawned process, no ambient state, no networking crate. Federation must never become a dependency of the compiler |
| `tests/dependencies.rs` | Any new crate is priced in the decisions log first |
| `cargo clippy -- -D warnings`, `cargo fmt --check` | |
| The five checks in `CLAUDE.md` | Stated briefly at the end of every session |

**A phase that needs a new dependency stops and asks.** Pricing it is a decisions-log entry with
rejected alternatives, and that is George's call, not a step in the phase.

---

## PHASE 0 — repair the design document. No code. **← next**

Two contradictions are live in `docs/federated-relearn.md` today, and the document is what every
later session reads. Fix the source before building from it.

**0.1 — §2 contradicts invariant 3.** Line 134 reads *"An install declares its own scopes in
config (`scopes = [...]`)"*. Invariant 3 reads *"no per-machine file that makes the same corpus
compile differently in two places."* A scopes config is exactly that file. `tests/solo_mode.rs`
holds invariant 3.

Resolve one way or the other and record it. The A1 prompt already assumes the flag
(`--scope`, composable with `--home`, no config file), so choosing the flag makes the document
agree with the phase that runs next; choosing config means loosening invariant 3 and amending
`solo_mode.rs`, which should be argued for explicitly rather than done quietly.

> **Sharper since A1 shipped.** This is no longer a choice between two futures. `--scope` is
> built, documented in README, and gated by `tests/scope_filter.rs`; the sentence at line 134
> describes a mechanism that does not exist and contradicts one that does. It is a lying artefact
> in the document every later session reads — `[R:repair-the-lying-artefact]` — and the remaining
> decision is narrower than it was: not *"flag or config"*, which the shipped code answers, but
> *"does an install config ever arrive, and under what argument"*.

**0.2 — the rotating install pseudonym is ambient state.** §5's `install = "7f3c9a1e"` must
survive between runs for `reports/<install-id>.toml` to be *replaced* rather than duplicated.
Anything that survives between runs on one machine is the per-machine file invariant 3 forbids.
Two resolutions that keep solo mode green: pass it on the command line so the human holds it, or
keep it inside the cloned aggregate repository, which is a path given on the command line.

~~**0.3 — while the document is open: `copilot-pack/README.md` is stale and will get staler.** It
claims 46 rules and 540 lines in its tree, 477 in its install prompt, and four project rules where
there are now seven across three projects. Its own verification step fails on a correct install.~~

> **Already closed, measured 2026-09-13 before filing.** `copilot-pack/README.md` reads *"52
> rules, 746 lines"*; `copilot-pack/.github/copilot-instructions.md` is **746** lines and
> `grep -c '^## '` over it is **52**; the install prompt's expected count is **746**, and its
> per-home figures (446 global, 200 domain-rust) are stated as alternatives rather than as the
> pack's own number. The layer table reads 27 / 18 / 7 and `relearn list --home` returns 27, 18
> and 7 — summing to 52. The README even says the counts are hand-written and nothing checks
> them, which remains true and is the standing exposure; it is not, today, wrong. The paragraph
> is struck rather than deleted because *"this is the pattern, not the exception"* below is still
> the correct warning, and a struck claim with its measurement is more useful than a silent
> removal.

**What George approves:** the two remaining resolutions, as decisions-log entries with the
rejected alternative recorded.

**What goes wrong here:** resolving 0.1 by loosening invariant 3 without noticing that
`solo_mode.rs` is the only thing standing between this tool and a config file. If the answer is
config, the test changes in the same commit and the reason is written down.

---

## PHASE A1 — `applies_to`: home is not scope — **SHIPPED 2026-09-13 (`3798521`)**

**Prompt:** `relearn_scope_applies_to_2026-09-13.md` (author's notes, not in this repository).

**In one line:** a new optional `applies_to: Vec<ScopeTag>` and a `--scope` flag, so a
low-latency rule lives in one home and compiles into both the Rust and Java layers.

**Why first:** it is the only phase worth building if federation never happens, and every later
phase needs the scope concept to exist.

**What George approves:** the `ScopeTag` newtype; the four-row emission table as four assertions;
the property test proving an unscoped rule emits identically under any `--scope`; the
decisions-log entry recording **why nesting was rejected**.

**What goes wrong:**
- **Scope leaks into the emitted path.** If `--scope rust` can produce a differently-named file,
  the change is wrong. Home alone decides the path.
- **`--scope` becomes subtractive below the unscoped baseline** — i.e. an unscoped rule stops
  emitting under some invocation. That is a silently dropped rule, the top invariant.
- **The 52 existing rules are "migrated"** by adding `applies_to` to some of them. The prompt
  asks for candidates and forbids adding any. Deciding a rule's audience is a judgement about who
  is harmed by not seeing it.

> **As built:** `ScopeTag` (private field, perimeter-parsed); `Rule::serves` holding both safety
> defaults where no caller can reimplement them; `--scope` on `build`, `verify` and `list` through
> one shared `restrict_to_scopes`; the four rows as four assertions in `tests/scope_filter.rs`;
> `narrowing_never_touches_an_unscoped_rule` and `scope_never_reaches_an_emitted_path_or_body` as
> whole-space properties; the nesting rejection in the decisions log. Emission unchanged for all
> 52 rules (`verify` green, 63 files). **No rule was given an `applies_to`** — candidates were
> reported and none applied. The three named failure modes are each covered by a test rather than
> by this paragraph.

---

## PHASE A2 — the org layer and mandated content

**Adds:** `Home::Org { name }`; `Origin::Mandated` beside `Mined | Codified`; an `approval = { by,
date, control }` block required when and only when origin is `Mandated`.

**Why now:** it is local, needs no network, and it is the layer a corporation fills. It also
establishes the federation exclusion *before* anything can federate, which is the right order.

**The load-bearing property:** an `Org`-homed rule can never be contributed and never appears in
a recurrence report — enforced by an **exhaustive match that will not compile** if a future home
variant is added without deciding its federation behaviour. Not a filter. Not a policy.

**What George approves:** the exhaustive match and its compile-fail pin; that `approval` is
mandatory for `Mandated` and unrepresentable otherwise; the FEATURES row for the exclusion.

**What goes wrong:**
- **`approval` ships optional**, and is therefore empty in exactly the rules that most need it —
  the identical argument that made provenance mandatory in decision 4.
- **`Mandated` rules are counted in recurrence statistics**, swamping the only number that says
  whether prose is holding.
- The exclusion is written as `if !matches!(home, Org)` somewhere in the report path rather than
  as a match the compiler forces. One is a guard someone can forget to add in a second code path;
  the other cannot be forgotten.

---

## PHASE B1 — `Authority`, and `adopt`

**Adds:** `Authority::Local | Cached { from, version, pulled }`; an `adopt` command converting a
cached rule to local with recorded provenance.

**The load-bearing property:** the edit path refuses `Cached`. Editing a cached rule is a silent
fork and P2 is gone with no error to read. Forking is allowed; forking by accident is not.

**What George approves:** that refusal being structural rather than a check in one function; the
version type having a **total order** (two caches of one rule must be comparable, always); the
`adopt` provenance line.

**What goes wrong:**
- **The refusal lives in one code path** and a second write path appears later without it. Same
  shape as `[R:generate-guards-unversioned]` — the answer is a marker the code recognises, not
  discipline at each call site.
- **Version comparison without a total order**, so "is this cache stale?" has no answer for some
  pairs.
- **Emitters start treating cached rules differently.** They must not. A cached rule compiles
  exactly like a local one; that is the whole point.

---

## PHASE B2 — `contribute`. The highest-risk phase in the programme.

**Adds:** a command that produces a publishable rule file with a **published incident** — a
rewritten account with no quotation, no names, no paths, no repository identifiers — while the
original never leaves the machine.

**Read §12.2 of the design first.** The contributor owns the scrub; review checks only what a
reviewer can actually see.

**What George approves:** that the raw `incident` is structurally incapable of reaching the output
(a different field, not a transformed one); the confirmation step showing exactly what will leave;
the fact that publication writes a file and does not transmit it.

**What goes wrong, and this is the one that ends the project's credibility if it happens once:**
- **A published incident still carries identifying content.** No gate can catch this — it is
  judgement about text, which is why §12.2 puts it on the contributor. What the code can do is
  make the raw field unreachable and force a human to read the output before it is written.
- **A "helpful" automatic scrubber appears.** It will leak what it did not recognise and destroy
  context it did not understand, and worse, it will make people stop reading the output because
  something is handling it.
- **`contribute` transmits.** It writes a file. A person runs `git`. `tests/solo_mode.rs` should
  fail loudly if this is got wrong, and that is the gate doing its job.

---

## PHASE B3 — `report`. Anonymous, always.

**Adds:** an anonymous recurrence report: upstream tags, counts as buckets, month-level dates,
status and control kind. Nothing else.

**Depends on Phase 0.2** being resolved — the install pseudonym cannot be ambient state.

**What George approves:** the field list, by exclusion as much as inclusion; the bucket
boundaries; that the k-floor and buckets are published constants a reader can check.

**What goes wrong:**
- **Ambient state.** `solo_mode.rs` will catch a home-directory file. It will not catch an
  environment variable read that someone adds "temporarily".
- **Bucket boundaries leak exact counts at small n.** A bucket of `1–1` is not a bucket.
- **Day-level dates survive somewhere** — in a `latest` field, in a filename, in a git commit
  date on the report file itself. That last one is real and easy to miss.
- **Local-only rules get reported.** Only upstream tags have shared identity; a local tag in a
  report is a private name published.

---

## PHASE C1 — the aggregate repository

**Adds:** a repository, not a service. Contributions and reports arrive as pull requests; a
scheduled job recomputes `aggregate.toml`.

**What George approves:** that `relearn` itself never reads it over a network; the recompute job;
the k-floor and the confound sentence printing beside the numbers.

**What goes wrong:**
- **Someone adds a fetch command "for convenience"** and the tool acquires a network client. This
  is the exact failure `tests/solo_mode.rs` exists to prevent and it will be proposed as an
  improvement.
- **The aggregate becomes authoritative** — a build that behaves differently depending on whether
  it has seen the aggregate. It must not.
- **The confound sentence gets dropped as clutter.** Cross-install recurrence measures frequency
  *and* diligence, inseparably, and a headline figure without that sentence is the overclaim this
  whole project exists to prevent.

---

## PHASE C2 — the poke, inside `lint`

**Adds:** the four triggers from §6, with the reactive one on by default and broadcast capped.

**What George approves:** the default table; the cap being a number in config rather than a
judgement in code; that it lives in `lint` and not in a new command.

**What goes wrong:**
- **Broadcast triggers default on**, because they are easier to demo, and within a month the poke
  is a notification people dismiss — including the one that mattered. Paper 3 §9 already
  documents this failure mode on exposed rows.
- **A `SessionStart` hook is installed.** The recurrence design already ruled this out of scope:
  anything under `~/.claude` edits the layer loaded into every session on the machine. Write the
  snippet into `docs/`; do not install it.
- **The poke fires during `build`.** `lint` reports, `build` emits. Mixing them means a build that
  talks.

---

## The review protocol

One phase, one session, one diff. Before George approves anything:

1. The five checks from `CLAUDE.md`, stated
2. `relearn verify` green — **and the phase says explicitly which rules' emission changed and why**
3. `tests/solo_mode.rs` and `tests/dependencies.rs` green
4. The `FEATURES.md` row, naming a real artefact and the invocation that runs it — or
   `NOTHING YET — exposed` and a `TODO.md` line
5. The `ARCHITECTURE.md` decisions-log entry, **with the rejected alternative**
6. The "bring back, do not decide" list, with nothing decided

**What George refuses:** a phase with no enforcer; a phase that also started the next one; a
decisions-log entry with no rejected alternative; a new dependency that was not priced first; any
diff where `relearn verify` changed emission for a rule the phase did not touch.

**What to send back here for review:** the diff summary, the FEATURES row, the decisions-log
entry, and the "bring back" list. Those four are enough to spot a deviation without reading the
code, and the code review is George's.

---

## Programme-level risks — named so they are visible

**Building ahead.** `docs/federated-relearn.md` is in the repository and every session reads it.
The temptation to implement two phases because the design describes both is the strongest failure
mode here. Rule Zero exists for it.

**Documentation updated after the fact.** Check 4 is the one that quietly slips, and the
recreation standard becomes a lie one phase at a time.

**Count assertions drifting.** Every phase adding a rule changes numbers stated in READMEs,
FEATURES rows and pack documentation. Any phase that changes a total must name every place
asserting it. *This document was itself an instance on the day it was written — it counted the
corpus at 51 in one place and described a pack README that had already been repaired — which is
the argument for the rule, not against it. The pack's layer counts remain hand-written with
nothing checking them, and that is the standing exposure.*

**Solo mode eroding by a thousand conveniences.** No single addition breaks it. A config file
here, an env var there, a cached fetch "only when asked". The test is the only thing holding the
line and the line is the product.

**The programme finishing and nobody using it.** Federation's value is entirely in the second
install. A1 and A2 are worth building regardless; B onward is worth building only if a second
person is going to run it. **That question should be answered before B1, not after C2.**
