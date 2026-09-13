# Federated relearn — design

**Status: design note only. Deliberately NOT built.** Nothing in this file is implemented:
there is no `applies_to` field, no `Authority`, no `Home::Org`, no `Origin::Mandated`, and
no `contribute`, `report` or `adopt` command. It is filed so the design is written down
rather than re-invented, on the same footing as
[`recurrence-session-hook.md`](recurrence-session-hook.md). The six questions in §12 are
open and are the author's to decide; §13 records what the definition of done would demand
if it is ever built.

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

An install declares its own scopes in config (`scopes = ["rust", "low-latency"]`) and emission
filters on the intersection. **Absent `applies_to` means today's behaviour exactly**, so no rule
file needs editing — the same migration shape as `recurrences` and the same round-trip assertion
over the existing 52 proves it.

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
generated = "2026-09"      # month, never a day

[[observation]]
rule        = "R:verify-through-production-path"   # an upstream tag; never a local-only one
recurrences = 2
latest      = "2026-08"
status      = "graduated"
control     = "type"        # type | property-test | unit-test | gate | hook — the KIND, never the code
```

Absent: title, incident, body, path, name, repository, language, day-level dates.

**Dates coarsen to the month as a privacy decision, not a formatting one.** In a team of eight,
*"someone hit this on 28 August"* identifies a person to anyone who was in the room.

**k-anonymity floor.** The aggregate publishes no rule's count until at least *k* distinct
installs have reported it. Below the floor, a count of one is a finger pointing at someone.
Propose k = 5, as a published constant a reader can check rather than a policy they must trust.

**Only upstream tags may be reported.** A locally-mined rule has no shared identity, so it cannot
be counted. To federate its signal you either adopt the upstream rule that covers the class, or
contribute yours — both deliberate human acts. *(v1 proposed a separate lightweight error-class
catalogue for exactly this case. It is dropped: the upstream library **is** the shared
vocabulary, which is a real simplification. The cost is that federating a signal now requires
contributing a whole rule rather than picking a taxonomy term, which raises the bar. Probably
correct — it keeps quality up — but it is a decision with a price and §12 keeps it open.)*

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
3. **The aggregate prints its own confound.** Cross-install recurrence measures frequency *and*
   diligence, inseparably — a class that appears rarely may be rare, or may be one nobody admits
   to. That sentence ships beside the numbers, or the headline figure is the overclaim this
   project exists to prevent.

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

## 12. Open — bring back, do not decide

1. **Dropping the error-class catalogue (§5).** It made federating a signal cheap; without it,
   contributing a whole scrubbed rule is the only route. Simpler, higher quality bar, fewer
   signals. Is that the right trade?
2. **Who reviews contributions?** The scrub in §4 is the step most likely to be done badly, and
   a reviewer who does not know the contributor's context cannot check it properly. This is the
   governance question and it decides whether the corpus stays trustworthy.
3. **What is k?** Proposed 5. Needs an argument, not a preference.
4. **Is `control` kind safe to publish?** Most useful field in the aggregate, most likely to leak
   something about a codebase. Probably fine; worth one adversarial pass.
5. **Scope vocabulary.** `applies_to` is a free-string set today. Uncontrolled, it grows four
   spellings of *low-latency*; controlled, somebody owns the list. Same problem as the catalogue,
   one layer down.
6. **What happens when an upstream rule is retired or superseded while installs cache it?** The
   cache needs its own staleness story, and it is the same shape as `Status` on rules. Do not
   invent it twice.

---

## 13. Definition of done, when this is built

All five per `CLAUDE.md`. Check 2 especially: `applies_to`, `Authority`, `Home::Org` and
`Origin::Mandated` all touch parse, serialize, lint and every emitter — and **the emitted tree
must not change for any of the 52 existing rules**. That round-trip assertion is what proves the
corpus was not disturbed, exactly as it did for `recurrences`.

`contribute`, `report` and `adopt` write no instruction layer and must not be reachable from
`build`.
