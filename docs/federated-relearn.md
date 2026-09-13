# Federated relearn — design

**Status: a design, now partly built.** One phase has shipped and the rest has not, and this
line says which — a document that still claimed "nothing here is implemented" would be lying
about the first thing a reader checks.

| | |
|---|---|
| **Shipped 2026-09-13 (A1, `3798521`)** | §2's `applies_to` and the `ScopeTag` type, with `--scope` on `build`, `verify` and `list`. Enforced by `tests/scope_filter.rs` and four properties in `tests/properties.rs`; FEATURES carries both rows |
| **Shipped 2026-09-13 (A2)** | §2's `Home::Org`, §7's `Origin::Mandated` and its `approval` block, and the federation exclusion as an exhaustive match — enforced by `tests/federation_exclusion.rs` and a compile-fail pin |
| **Shipped 2026-09-13 (B1)** | §3's `Authority` and `adopt`, with the cached-rule refusal as a witness type at the write boundary and a compile-fail pin |
| **Shipped 2026-09-13 (B2)** | §4's `contribute`, as a projection that never holds the raw incident, plus the banned-terms matcher over one string |
| **Shipped 2026-09-13 (B3)** | §5's `report` — five fields, bucketed counts, a day-free `Month` type, and the pseudonym living in the clone (§0.2 resolved in code). The aggregate itself (§9) is not built |
| **Shipped 2026-09-13 (C1)** | §9's recompute, as `relearn aggregate`: the k-floor applied from the same constant the producer reads, both §8 confounds written unconditionally, and a source-level check that no build depends on an aggregate. The aggregate **repository** does not exist yet — this is the command a job would run |
| **Not built** | the aggregate repository and its scheduled job (§9); the poke (§6); `ScopeNearDuplicate` (§12.5) |

The rest is filed so the design is written down rather than re-invented, on the same footing as
[`recurrence-session-hook.md`](recurrence-session-hook.md). The six questions §12 carried open
were **answered on 13 September 2026** and §12 now records the decisions with their whys; two of
them moved the design rather than merely filling a blank, and §5 and §8 carry the edits. §13
records what the definition of done demands of the phases still to come.
[`federation-programme.md`](federation-programme.md) sequences them, one per session.

### One home, many caches, and a signal that travels · v2, 13 September 2026

*Supersedes v1 of this date — filed as `docs/federated-recurrence.md` in commit 522679f and
renamed with the design, so read v1 in that commit rather than in this path's history, which the
rewrite is too large for git to follow across. v1 federated only recurrence counts and kept every
rule local, because sharing a rule means sharing its `incident`. That constraint has not gone
away; it has moved to contribution time, where a human is present and knows what is sensitive.
Rules now travel. The privacy design is stricter as a result, not looser.*

**Where it lives:** the public `relearn` repository, Apache-2.0. Nothing here is designed around
any employer's constraints and nothing passes anyone's door.

**State when written:** 52 rules. `Rule { tag, title, error_class, home, created, origin, status,
incident, body, recurrences }`. `Recurrence { date, incident }`. `Origin::Mined | Codified`.
`Status::Active | Graduated { to, date } | Attic { .. }`. `Home::Global | Domain { name } |
Project { path }`. `Finding::OverlappingScope | HomeSlugCollision | DanglingReference |
RetiredReference | UnheldRecurrence`. Verify before building.

> The draft read "51 rules", as v1's did. Corrected at filing by enumerating `rules/*.md` — 52
> files, every one a rule, with no README or fixture among them; `[R:measure-the-claim-not-a-subset]`
> asks for an enumeration that can be read rather than a count that must be trusted. The type
> inventory above was re-checked against `src/lint.rs`, `src/rule/origin.rs` and `src/rule/home.rs`
> and stands.

---

## 1. The thesis

**One home. Many caches. The cache is an emission.**

A rule has exactly one canonical home, as it does today. Installs that find a rule suitable hold
a **cached copy**, and a cache is never authoritative and is always regenerable — which is the
rule this repository already lives by:

> *No emitted artifact is a source. Anything under an emitter's output path may be regenerated at
> any time and must never be hand-edited.*

That principle, applied one level up, is the whole architecture. P2 is not weakened by
federation; it is the thing that makes federation safe.

Three things then travel, and **each has a different privacy model**, which is the part that is
easy to get backwards:

| Flow | Direction | Attribution |
|---|---|---|
| **Contribution** — publishing a rule you authored | up | **Attributed.** It is authorship |
| **Cache** — pulling a rule someone else authored | down | Carries its author |
| **Recurrence report** — this class fired for me | up | **Anonymous**, always |

Getting those the wrong way round kills both halves. Credit is what makes anyone contribute.
Anonymity is what makes anyone admit a rule of theirs failed — and recurrence is the only number
that says whether a rule works.

### The single-install guarantee — solo mode is the product

Everything above is an option. **What `relearn` is, and must still be after all of it, is a
compiler one engineer points at their own rules to emit their own instruction layers** — Claude
skills, Cursor `.mdc`, Copilot, `AGENTS.md`, project `CLAUDE.md` — on one machine, offline, with
no account, no config file and nobody else in the picture. Federation is a feature of the corpus,
never a dependency of the compiler.

Four invariants, and each names what holds it rather than promising it:

1. **No configuration is required, and its absence is never a filter.** A rule with `applies_to`
   emits everywhere when the invocation asked for no audience, and an unscoped rule emits under
   every audience. Narrowing happens only when somebody asks for it, and an audience no rule
   declares is a loud error rather than a tree quietly missing every scoped rule — a rule dropped
   in silence is a lost correction, which is the failure the whole project exists to prevent.
   *Held by:* `Rule::serves`, which carries both defaults where no caller can reimplement them,
   plus `narrowing_never_touches_an_unscoped_rule` and
   `no_audience_emits_exactly_what_an_unnarrowed_build_emits` over generated libraries, and
   `CliError::UnknownScope` for the loud half.

   > **Repaired 2026-09-13 (Phase 0).** This read *"Held by: `Audience::Everything` as the
   > constructed default"* — a type that was never built, named as an enforcing artefact before
   > the phase ran. A1 held the invariant differently and better, in a method on `Rule` rather
   > than a wrapper type around the audience. An *"Enforced by"* pointing at a plan is the failure
   > `FEATURES.md` exists to prevent, and it had reached the design document instead
   > (`[R:guarantee-needs-a-reader]`).
2. **The tool never speaks to a network.** Contribution and reporting are a person running `git`,
   not a client calling a server, which is exactly why §9's aggregate is a repository rather than
   a service. *Held by:* `tests/solo_mode.rs` — no socket, no spawned process, no networking or
   TLS or async-runtime crate anywhere in the dependency tree.
3. **The tool consults no ambient state.** Its input is the path it was given; its output is the
   path it was given. No home directory, no environment, no per-machine file that makes the same
   corpus compile differently in two places. *Held by:* `tests/solo_mode.rs`.
4. **Every federated field is optional, and absence is today's behaviour exactly.** `applies_to`,
   `Authority`, `approval`: a solo corpus that never adopts anything never meets any of them, and
   no existing rule file needs editing. *Held by:* the round-trip assertion in §13 — which for
   `applies_to` is no longer a promise: `tests/corpus.rs` round-trips all 52 rules byte-identically
   and `relearn verify` was green on the committed tree the day A1 landed.

The reason to write this down as invariants rather than intent: every federated feature is a
standing reason to acquire an account, a client, a config file, a cache directory — each
individually reasonable, and collectively the end of the thing a single engineer downloads. That
erosion does not arrive as a decision anyone would have approved. It arrives one convenience at a
time, which is what a gate is for.

---

## 2. Layers: home is not scope

### The problem with a tree

`rust/low-latency` and `java/low-latency` both need NUMA residence, cache-line behaviour, what a
socket crossing costs, why a busy-spin core must be exclusive. Under a nested hierarchy that
knowledge is written twice — two homes, drifting, in the tool built to prevent two homes. What
varies by language is the implementation; what varies by discipline is the principle. A tree
expresses one axis and forks the other.

### The split

**`Home` answers *who owns and maintains this rule*.** One, enforced as a sum type, unchanged
except for a new variant:

```rust
pub enum Home {
    Global,
    Org { name: OrgName },        // NEW
    Domain { name: DomainName },
    Project { path: ProjectPath },
}
```

**`applies_to` answers *which audiences load it*.** A new optional set, absent by default:

```toml
home       = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
```

One file, one tag, one incident, one owner — P2 untouched — compiled into both language layers.
Low-latency knowledge accumulates once and both languages inherit it.

An invocation names the audience it wants — `relearn build --scope rust --scope low-latency`,
repeatable, composing with `--home` — and emission filters on the intersection. **Absent
`applies_to` means today's behaviour exactly**, so no rule file needs editing; the same migration
shape as `recurrences`, and the same round-trip assertion over the existing 52 proves it.

> **Repaired 2026-09-13 (Phase 0.1).** This paragraph read *"An install declares its own scopes in
> config (`scopes = [...]`)"* — a per-machine file that makes the same corpus compile differently
> in two places, which is exactly what §1's invariant 3 forbids and `tests/solo_mode.rs` fails on.
> The contradiction was written into the document by the later addition of §1, and by the time
> A1 shipped the flag the sentence was describing a mechanism that does not exist while
> contradicting one that does. **A config file is not deferred, it is refused**: the decisions log
> (2026-09-13) carries the argument and the condition that would reopen it.

### The org layer

`Home::Org { name }` carries an organisation's own engineering principles. Its defining property
is structural:

**An `Org`-homed rule can never be contributed and never appears in a recurrence report.** Not by
policy — by the type system, with an exhaustive match that will not compile if a future home
variant is added without deciding its federation behaviour. The public tool supports the layer;
every organisation fills it privately; nothing of theirs can cross the boundary even by accident.

---

## 3. Authority: local or cached

A local install now holds two species of rule, and the distinction must be unrepresentable to
confuse:

```rust
pub enum Authority {
    /// This install is the rule's home. Source of truth. Editable.
    Local,
    /// Home is elsewhere. A cache. Regenerable. Never hand-edited.
    Cached { from: SourceId, version: Version, pulled: Date },
}
```

Emitters treat both identically — a cached rule compiles into the instruction layer exactly like
a local one, which is the point. **The edit path refuses `Cached`**, the same way
`[R:generate-guards-unversioned]` refuses to overwrite a target it did not generate. Editing a
cached rule is a silent fork, and a silent fork is P2 gone with no error to read.

To change a cached rule you either open a contribution upstream, or **adopt** it: a deliberate
command that converts it to `Local` with a recorded provenance line saying what it was forked
from and when. Forking is allowed. Forking by accident is not.

---

## 4. Contribution — attributed, and scrubbed where the human is

Publishing a rule is a deliberate authored act. It carries the author's name, because credit is
the incentive that makes the corpus exist.

**The `incident` field is the whole problem and it is solved at contribution time, not at sync
time.** A rule's incident is a verbatim quotation from a private working session. It cannot
travel as written.

So `relearn contribute` requires a **published incident**: a rewritten account giving the error
class, what went wrong and what it cost, with no quotation, no names, no paths, no repository
identifiers. The original stays local and is never transmitted.

This is deliberately a human step. A scrubber guessing at sync time will be wrong in both
directions — it will leak what it did not recognise and destroy context it did not understand.
The author, at the moment of contributing, still remembers what was sensitive. `relearn
contribute` shows exactly what will leave and requires confirmation.

`[R:names-travel-with-the-quote]` is the rule this implements, and the contribution path is its
first mechanical enforcement rather than a note to be careful.

---

## 5. Recurrence reporting — anonymous, always

Separate flow, separate file, separate privacy model.

```toml
schema    = 1
install   = "7f3c9a1e"     # rotating pseudonym; regenerable; not a person
                           # lives in the cloned aggregate repo, never on the machine — see below
generated = "2026-09"      # month, never a day

[[observation]]
rule        = "R:verify-through-production-path"   # an upstream tag; never a local-only one
recurrences = "2-4"         # a bucket: 1 | 2-4 | 5-9 | 10+ — never an exact count
latest      = "2026-08"
status      = "graduated"
control     = "type"        # type | property-test | unit-test | gate | hook — the KIND, never the code
```

Absent: title, incident, body, path, name, repository, language, day-level dates.

**Where the pseudonym lives, and why it is not a dotfile (Phase 0.2, 2026-09-13).** The id has to
survive between runs or a second report *adds* an install rather than replacing one, and the
aggregate counts one person twice. Anything surviving between runs on the machine is the
per-machine state §1's invariant 3 forbids, so it lives **inside the cloned aggregate repository**
— `reports/<install-id>.toml` is its own name, and the clone is a path the human passes on the
command line. `report` reads the id from the report file already in the clone and writes a new one
only when there is none; there is nothing to read on the machine, so `tests/solo_mode.rs` stays
green with no exemption. Delete the clone and the pseudonym is gone, which is the mental model a
person already has for a clone. The cost is stated rather than hidden: the pseudonym is exactly as
private as the clone, so a shared or backed-up checkout shares it.

**Dates coarsen to the month as a privacy decision, not a formatting one.** In a team of eight,
*"someone hit this on 28 August"* identifies a person to anyone who was in the room.

**k-anonymity floor, and buckets.** The aggregate publishes no rule's count until at least *k*
distinct installs have reported it; k = 5, a published constant a reader can check rather than a
policy they must trust. But k protects the wrong quantity on its own — which rule you hold is
barely sensitive, whereas an **exact** count plus a month, reported month after month, stitches a
rotating pseudonym back into one install. So counts publish as buckets. The decay curve in §11
needs orders, not integers, so this costs almost nothing and closes the longitudinal path. §12.3
carries the argument, including the case a constant cannot fix.

**Only upstream tags may be reported.** A locally-mined rule has no shared identity, so it cannot
be counted. To federate its signal you either adopt the upstream rule that covers the class, or
contribute yours — both deliberate human acts. v1's error-class catalogue existed for exactly
this case and is dropped (§12.1): the upstream library **is** the shared vocabulary, one identity
space rather than two.

**Which makes `adopt` load-bearing rather than a convenience.** The bar for contributing is
deliberately high, and a high bar suppresses signal hardest where scrubbing is hardest — so the
common case, where upstream already holds a rule for the class that just bit you, has to cost
nothing: `relearn adopt <tag>` caches it, and from that moment recurrences are reportable with no
authoring at all. Contribution is then reserved for genuinely new classes, where a high bar is
right.

---

## 6. The poke

Four possible triggers. Only one should be the default.

| Trigger | Kind | Default |
|---|---|---|
| A rule fired here, and upstream already has one for that class | **reactive** | **on** |
| A new rule matching your scopes was contributed | broadcast | off |
| A rule you cache has a newer version upstream | broadcast | on, silent until `lint` |
| A high-recurrence rule exists that you do not cache | broadcast | off |

**The reactive trigger is the one that makes this a collective mind rather than a mailing list.**
It arrives at the moment a developer has just demonstrated they needed it — *you recorded a
recurrence; forty-seven installs hit this class and X wrote a rule for it in March.* Everything
else is broadcast, and broadcast is how a notification channel teaches people to ignore it.

**Budget it, because this failure mode is already documented in the author's own §9:** an exposed
row *"is useful only while it embarrasses someone"* — twenty of them and readers learn to skip
the column. A poke that fires weekly trains people to dismiss the one that mattered. Cap the
broadcast pokes per run and make the cap a number in the config, not a judgement in the code.

*Amended in implementation, 2026-09-13 (C2).* **"A number in the config" is a flag**, because
Phase 0 refused a configuration file outright and invariant 3 forbids one: the cap is
`--poke-cap`, its default is the published constant `BroadcastCap::DEFAULT`, and zero is a real
setting that turns broadcast off while leaving the reactive trigger alone. The sentence's point —
that the cap is a number the operator sets rather than a judgement buried in the code — is kept;
only the place it is set has moved, because the place this document named does not exist and must
not. The rest of the table shipped as written.

**Surface it in `relearn lint`.** It already exists, and it already runs on every change in CI —
so it is an invocation actually on the path, which is the author's own sharpest criticism of the
enforced-by column: it records enforcement, never invocation. A separate `relearn news` command
is a command nobody runs.

---

## 7. Compliance content is a different species

An organisation's control-framework requirements are not corrections. They have no incident. They
were never mined. Blending them into the corpus would swamp every metric that makes relearn
worth anything.

**Give them their own origin and their own provenance.** For a mined rule, provenance is the
incident. For a mandate, provenance is the approval:

```toml
origin   = "mandated"                      # NEW, beside mined | codified
approval = { by = "...", date = "2026-07-11", control = "..." }
```

Then:

- **Recurrence statistics exclude `mandated` rules.** A rule nobody mined cannot tell you whether
  prose is holding.
- **The counter-metric to the whole federation is the mined fraction.** A corpus dominated by
  `codified` and `mandated` rules is sharing doctrine, not experience — and the author's own null
  result says doctrine is the part that changes nothing.

**And the tool must not claim regulatory alignment.** *"Aligned with compliance, security and
several regulations"* is a guarantee, and `[R:guarantee-needs-a-reader]` asks what enforces it.
Nothing does, unless a named function signs it. A tool asserting global regulatory alignment with
no signer is the unenforced-guarantee failure class from Paper 3 — in a regulated environment,
a liability rather than a feature. Record the approver; let any claim follow from the record.

---

## 8. Counter-metrics — P5

Recurrences are recorded only by people willing to write down that their rule failed. Published,
the incentive to under-report is stronger than it is locally. Three defences, and the first two
are structural rather than promises:

1. **No install can be ranked, because the aggregate cannot identify one.** Rotating pseudonym
   plus k-floor. A design where the aggregator *could* rank and undertakes not to is a design
   that will eventually rank.
2. **Nothing is ever auto-published.** `report` and `contribute` write files; a human publishes
   them. The first person who discovers the tool syncing silently is the last person who uses it.
3. **The aggregate prints its own confounds — both of them.** Cross-install recurrence measures
   frequency *and* diligence, inseparably: a class that appears rarely may be rare, or may be one
   nobody admits to. And since v1's catalogue is gone (§12.1), the aggregate can only ever count
   **classes somebody has published a rule for** — never the universe of error classes. That
   second sentence is not a sparseness caveat, it is a bias: the authoring-and-scrubbing bar is
   highest exactly where the incidents are most sensitive, which is often where they are most
   expensive. Both sentences ship beside the numbers, or the headline figure is the overclaim
   this project exists to prevent.

---

## 9. Where the aggregate lives — v1 needs no infrastructure

**A git repository.** Contributions arrive as pull requests adding `rules/<tag>.md`. Recurrence
reports arrive as pull requests replacing `reports/<install-id>.toml`. A scheduled job recomputes
`aggregate.toml`. That is the entire service.

- Publication is deliberate by construction: it is a pull request
- Every byte that ever crossed is in public history, auditable forever
- Human review of contributions is free, because it is code review — and contributions *need*
  review, since §4's scrub is the thing most likely to be done badly
- No hosting, no accounts, no database, no privacy policy to write
- If it fails it fails as an empty repository rather than as an outage

A service can come later if volume demands it. It should not come first.

---

## 10. What must not be built

- **No automatic sync**, in any version, in either direction.
- **No identity in recurrence reports**, ever. Not opt-in, not hashed-but-reversible, not "for
  support".
- **No transmission of a raw `incident`.** The published incident is a separate authored field;
  the original never leaves the machine.
- **No `Org`-homed rule leaving the machine**, enforced by an exhaustive match rather than a
  filter someone can forget.
- **No retrieval.** Scopes bound what an install caches, so the local corpus stays small enough
  to compile. The moment this needs a vector index it has become a different product and must be
  argued for as one.
- **No silent adoption.** A cached rule arrives because someone pulled it, not because it matched.
- **Nothing federated may become required** — no account, no config file, no network client, no
  ambient state, and no scope filter that narrows a build nobody asked to narrow. The affirmative
  form, with what holds each part, is §1's single-install guarantee; `tests/solo_mode.rs` is the
  gate, and it is in the suite today rather than waiting for the first federated line of code.

---

## 11. Why this matters beyond the tool

The author's central research problem is **n = 1**: nine repositories, one author, one assistant,
and a reviewer objection no venue change dissolves.

Federation makes every install a potential data point, and cross-install recurrence against
classes retired is the first measurement of a curve Paper 3 explicitly declines to claim:

> *"The shape is an image and not a measurement: nobody has shown the decay is geometric and this
> paper does not claim it."* — Figure 3

Nobody else holds this data, because nobody else records corrections with mandatory provenance in
a portable format.

**But publication is not research consent.** Using the aggregate as study data requires separate,
explicit, informed opt-in, stated at the point of publication and recorded. Building the pipeline
and then deciding it is a dataset is precisely the failure this series argues against.

---

## 12. Decided — 13 September 2026

These six were filed open, as *bring back, do not decide*. They were brought back and decided the
same day. Each records the question as it stood, the decision, and the why — and where a decision
moved the design rather than filling a blank, it says which section changed.

### 12.1 The error-class catalogue stays dropped, and `adopt` pays the cost back

*Open as: it made federating a signal cheap; without it, contributing a whole scrubbed rule is the
only route. Simpler, higher bar, fewer signals — is that the right trade?*

**Dropped, and the strongest argument is not simplification.** A catalogue is a second identity
space for one concept, and mapping between two identity spaces is entity resolution performed
per-install, per-rule, forever — with no error-correcting feedback, because a mis-mapped class is
invisible while a contributed rule is read by someone. Note that v1's hardest open question,
*who curates the catalogue*, existed **only** because of the catalogue: dropping it deletes a
governance problem rather than deferring one.

But v2 understated the cost, and it is not "fewer signals" — it is a **biased** under-count.
Raising the bar to author-scrub-PR-review suppresses signal hardest where scrubbing is hardest,
which is the sensitive private repositories, which are disproportionately where the expensive
incidents happen. Sparse is fine; skewed toward the cheap incidents is a measurement problem.

The repair is not a taxonomy, it is making the common case free — see §5, where `adopt` is now
load-bearing rather than a convenience — and stating the residual bias in the aggregate itself,
which is §8's second confound. Both sections changed.

### 12.2 The contributor owns the scrub; review checks what a reviewer can actually see

*Open as: the scrub in §4 is the step most likely to be done badly, and a reviewer who does not
know the contributor's context cannot check it properly.*

**Then stop designing review as the scrub control.** A reviewer cannot know that *"the deploy
script"* names a customer. A control whose reader cannot perform the check is a guarantee with no
reader — `[R:guarantee-needs-a-reader]`, in the design of the tool that enforces it.

The scrub moves to the only place it can work, the contributor's machine, mechanically:
`relearn contribute` runs the banned-terms matcher over the **published** incident before it
writes anything. That machinery exists here already — `scripts/no-banned-names.sh`, salted
digests, normalise then join runs of up to three tokens then slide every stored length inside
each token. The matcher is reusable; its `File::Find` half is not, since the target is one string
rather than a tree. It must keep the gate's two hard properties: a finding reports location and
length and **never** the match (`[R:report-the-hit-not-the-match]`), and a missing term list
exits 2, not 0 — a disarmed scrub refuses to contribute rather than passing green.

That leaves the reviewer the job a contributor genuinely cannot do: does this duplicate an
existing rule's error class (P2 — only someone holding the whole corpus can see it), is it
`mined` or doctrine wearing a rule's clothes, does it parse, and a belt-and-braces read for an
obvious identifier. Governance starts as one maintainer plus a published merge checklist;
per-domain reviewers arrive when a domain has volume, because deciding a scaling structure at
zero volume is inventing a requirement.

One thing said plainly rather than implied: **a bad scrub is irreversible.** A contribution can
be withdrawn from the corpus; public git history is forever. Pre-publication is the only real
control, which is the whole reason the check moved to the contributor's machine.

### 12.3 k = 5, and counts publish as buckets because k alone protects the wrong quantity

*Open as: proposed 5, needs an argument, not a preference.*

**k = 5**, because it is the standard statistical-disclosure cell-suppression floor — national
statistics offices commonly use 3 or 5, the k-anonymity literature 5 to 10 — and its virtue is
that a reader can look the benchmark up instead of trusting the author's taste.

The sharper finding is that k guards the wrong thing. Which rule an install holds is barely
sensitive. The fingerprint is the **exact** recurrence count plus a month, reported month after
month: rotation of the pseudonym does not help when successive counts stitch the identities back
together. So counts publish as buckets — `1 | 2-4 | 5-9 | 10+` — which §5's report format now
carries. The decay curve in §11 needs orders of magnitude, not integers, so the analytical cost
is near zero and the longitudinal path closes.

And the caveat a constant cannot fix, stated rather than papered over: if the reporting population
is small and homogeneous — every report from one organisation's installs — k = 5 protects nobody,
and the aggregator cannot detect that it is happening.

### 12.4 Publish `control` kind — conditional on it being a sealed enum with no escape hatch

*Open as: most useful field in the aggregate, most likely to leak something about a codebase.*

The adversarial pass: the value set is five closed values. *"This install uses gates"* says they
have CI. *"type"* implies a typed language, which the rule's tag usually implies anyway. Against
the rule identity already being published, the control kind adds essentially no linkage — and it
is the field that makes the aggregate worth reading, since *four installs held this with a type
and one with a gate* conveys the shape of the answer without anyone shipping code.

The leak arrives the day someone can write `gate:internal-payments-lint`. So the condition is
structural rather than procedural: `Control` is a **sealed enum with no free-text variant and no
`Other(String)`**, parsed at the perimeter, a sixth kind requiring a public schema bump. The
control's *name* is never published, and not as a hash either — a digest over a small dictionary
is a lookup, not a protection.

Published on that condition, which the type system holds rather than a reviewer remembering.

### 12.5 The corpus is the scope vocabulary; a linter catches the near-misses

*Open as: free strings grow four spellings of low-latency; a controlled list needs an owner.*

Neither. A `Scope` newtype parses at the perimeter and enforces lexical shape — lowercase,
`[a-z0-9-]`, no leading or trailing hyphen, bounded length — which kills `Low-Latency` and
`low latency` outright. The set of scopes in use is then **whatever appears in the public
corpus**: one identity space, not two, which is §12.1's answer applied one layer down.

Drift is a near-miss problem and near-misses are detectable without a curator: a
`ScopeNearDuplicate` finding for a scope used by exactly one rule within edit distance 2 of a
scope used by many. That is the shape of `Finding::OverlappingScope`, which exists — extend it
rather than inventing a parallel mechanism.

One convention that cannot be typed and therefore has to be written down: **scopes are audiences,
not topics.** `rust`, `java`, `embedded` — things an install can declare it *is*. Not
`performance`, `security`, `testing`, which turn the field into tags and make it a second home
for what `error_class` already carries.

### 12.6 A local attic suppresses; an upstream attic only warns

*Open as: the cache needs its own staleness story, and it is the same shape as `Status` on rules.
Do not invent it twice.*

Reuse `Status`; `Authority::Cached { version, pulled }` already supplies the comparison point.
The decision that matters is **who** retired it. A local `Attic` suppresses emission. An upstream
`Attic` only warns — deleting an instruction a team relies on because a stranger retired it is a
correction lost with no reader, which is P1 and the exact failure this tool exists to prevent;
the local install may hold evidence the upstream author does not. That local-versus-remote
distinction is precisely what `Authority` models, so expressing it needs nothing new.

`CachedRuleRetiredUpstream { tag, upstream_status, since }`, Warning. Three human resolutions:
pull, adopt, drop. And **`adopt` on an upstream retirement is the most valuable signal the
federation can produce** — a retirement that installs refuse is evidence the retirement was
wrong, which is the population telling an author something no single install can know.

*Amended in implementation, 2026-09-13.* **It ships as the fifth poke, not as a lint Warning.**
The sentence above predates the poke, and a `Warning` reachable only with `--upstream` is
federation failing a run at the default `--deny warning` — which §1's invariant 3 forbids and
which the poke was shaped to make impossible. Same input, same reach, same inability to touch a
verdict; it sits beside `cache-behind`, on by default, and cannot fire at all against an install
that caches nothing. Everything the paragraph is actually *about* survives unchanged: an upstream
attic only warns, the three resolutions are named with `adopt` first, and the suppression is
unreachable rather than declined, because only a local `Status` reaches `Status::emittability`.
The trigger reads only an attic — a graduated rule is still emitted, annotated — so the "no new
`Status` variant" reading below is what held.

On supersession: **no new `Status` variant.** `Graduated { to }` means promoted to a control, not
replaced by another rule, and rule-replaces-rule is just a new version of the same tag — the
cache sees `version` move. Only tag-level death needs `Attic`. That is the reading of *do not
invent it twice* that actually holds.

---

## 13. Definition of done, for the phases still to come

All five per `CLAUDE.md`. Check 2 especially: `Authority` touches parse, serialize, lint and every
emitter — and **the emitted tree must not change for any of the 52 existing rules**. That
round-trip assertion is what proves the corpus was not disturbed, exactly as it did for
`recurrences`, exactly as it did again for `Home::Org` and `Origin::Mandated` in A2 (where the
compiler enumerated the six emitter sites that had to decide about the new home, and the lint
summary stayed byte-identical because the corpus holds no mandate), and exactly as it did for
`applies_to`, which was
the fourth field to arrive under it and the first of these phases to ship.

`contribute`, `report` and `adopt` write no instruction layer and must not be reachable from
`build`.

**§12 turned four of the answers into types rather than policies, and those are the enforcing
artifacts to name when the FEATURES rows are written.** `Control` is a sealed enum with no
free-text variant (§12.4) — the leak is unrepresentable, not forbidden. A recurrence count
serializes as a bucket, so there is no path that emits an exact integer (§12.3). `Scope` parses
at the perimeter — **shipped as `ScopeTag` in A1**, while `ScopeNearDuplicate` has not been built
and waits for a second scope to exist, since a drift detector over an empty vocabulary arrives
already green (§12.5). `CachedRuleRetiredUpstream`
is a Warning and cannot suppress emission, because only a local `Status` reaches
`Status::emittability` (§12.6). The one answer that is *not* a type is §12.2's governance split,
and it is written down precisely because nothing can check it — the half that can be checked, the
banned-terms matcher over the published incident, exits 2 when disarmed.
