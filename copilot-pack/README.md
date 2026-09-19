# Copilot pack — installation

A self-contained drop-in that loads this repository's rule library into GitHub Copilot
as custom instructions. Download this folder, copy one file into your repository, done.

## What is in here

```
copilot-pack/
├── README.md                          this file
└── .github/
    └── copilot-instructions.md        83 rules, 2109 lines — the instruction file
```

The instruction file is **generated** by `relearn build --targets copilot` from the
neutral rule library in [`rules/`](../rules). It carries every rule in the library:

| Layer | Rules | What they cover |
|---|---:|---|
| `global` | 30 | The repository discipline and its controls, plus language-agnostic verification, provenance, disclosure and cost reasoning — see below |
| `domain-rust` | 18 | The complete Rust type-driven design discipline — see below |
| `domain-low-latency` | 15 | Mechanism and measurement discipline for latency-sensitive work: contended paths, publication windows, memory ordering, and the instruments that report confidently wrong numbers |
| `domain-java` | 13 | The Java language discipline — nullability, equality, sealed alternatives, safe publication, resource and inheritance rules |
| `project-*` | 7 | Rules belonging to the `relearn`, `stochos-lab` and `design-architecture-tool` repositories specifically |

These counts are hand-written and nothing checks them — which is exactly how they went
wrong: this table read 27 / 18 / 7 for long enough to sum to 52 while the line above it
said 57, and the `domain-low-latency` layer was missing from it altogether. `grep -c '^## '`
on the instruction file is the number that cannot go stale, and
`grep -c '^home = ' ../rules/*.md` grouped by home is the breakdown that cannot.
`[R:doc-currency]`

### The repository discipline

The four standing documents and the checks that keep them true — the subject of
*Aiming the Stochastic Machine*, and the discipline this repository is itself built on.

The documents: `five-files-no-more` (the set, and resisting a sixth),
`claude-md-recreates-the-project` (the charter's recreation standard),
`decisions-log-records-rejected-alternatives` (the architecture log — the why *and* what
was refused), `features-ledger-names-its-artefact` (the regression ledger with teeth),
`definition-of-done-every-change` (the five checks that run on every fix).

The controls that keep enforcement honest — these are the ones that fail quietly:
`wired-artifact` (a success check must consume evidence nothing else can forge),
`guarantee-needs-a-reader` (a prose safety claim names what enforces it, or goes),
`verdict-survives-the-channel` (the check was right and its answer was lost on the way
to the reader — a pipeline reports its last stage, a filter crops the verdict),
`reconcile-wiring-at-start` (a correctly-wired control can go dark and say nothing),
`no-stale-push-over-fresh` (a mirror script must know which side is authoritative),
`detector-excludes-own-definitions` (a self-matching check is always red, so it gets
muted), `revision-integrity` (restructuring silently breaks references the author cannot
see), `case-collision` (two names differing only by case are one file on NTFS).

Three about what a record costs, and who pays: `price-every-dependency` (a dependency is
a decision that arrives as one line in a manifest, and the decision to take *none* leaves
no artifact at all, so a crate already refused is added by the next session),
`names-travel-with-the-quote` (quoting an incident carries its names past the gate that
was holding them — a gate scans its own tree, and the quotation travels without it),
`report-the-hit-not-the-match` (searching for a thing whose whole problem is that it
exists means the output is another copy of it: report location, count and length, never
the matched text, and never a field that can contain it).

And one about whose call a cost is: `no-silent-spend` — a trade of the user's time
against their money against thoroughness is theirs to make, put as a one-line costed
fork at the moment it arises. It is symmetrical: quietly spending more to be thorough
is exactly as wrong as quietly spending less to be quick.

### The Rust set

All 18 Rust rules are included. Types and boundaries: `design-types-first`,
`newtype-liberally`, `parse-dont-validate`, `parse-wide-then-range-check`,
`private-fields-only`, `typestate-for-protocols`, `typestate-builder-for-required-fields`,
`seal-closed-trait-sets`, `module-visibility-is-deliberate`. Errors and outcomes:
`errors-name-what-failed`, `no-anyhow-in-libraries`, `no-unwrap-in-production`,
`must-use-on-consequential-returns`. Ownership and cost: `justify-every-clone`,
`borrow-in-signatures`, `async-all-the-way`, `verify-the-abstraction-compiled-away`.
Testing: `xplat-fixtures`.

Together with the global rules on illegal states and sentinel values, this is the
whole discipline: the type system carries the guarantee, property tests carry the
behavioural laws types cannot encode, and unit tests are regression pins.

### The Java set

All 13 Java rules are included, and they are the **language** discipline rather than
performance advice — the split is deliberate and is described below. Values and
contracts: `null-is-not-a-value` (absence stated in the type, never done by a
reference), `equality-is-one-contract` (equals and hashCode as one decision, and a
mutable key is lost inside its own map), `identity-is-not-equality-for-boxes` (`==`
right up to 127, then silently wrong). Structure: `seal-the-alternatives` (a closed set
is a sealed hierarchy matched exhaustively, never a `default` arm),
`design-for-inheritance-or-forbid-it`, `no-reference-to-internals-escapes` (`final`
freezes the reference, not the object). Concurrency and lifetime:
`publish-safely-or-not-at-all` (the JMM — a reader can see a non-null reference to a
half-built object), `close-what-you-open`, `a-view-is-not-a-copy`. And the three that
catch a language feature behaving as a second constructor or a hidden cost:
`serializable-is-a-second-constructor`, `exceptions-name-what-failed`,
`a-wrapper-type-is-not-free-here`, and
`a-speculated-path-deoptimises-when-the-input-changes` (the JIT compiles a bet about
your code and settles it at the moment the bet stops being true — so the worst latency
lands on the least ordinary input).

That last one exists because the corpus itself creates the hazard: `newtype-liberally`
argues a newtype is ordinarily zero-cost, which is true in Rust by construction and true
in Java only when escape analysis agrees. It keeps the design argument and refuses the
cost argument.

### The low-latency set

All 15 are included. This is a **discipline** domain rather than a language one, so the
rules are written for whoever has a deadline — most declare `applies_to = ["rust",
"java"]`. Mechanism: `no-retry-loop-on-a-contended-path`,
`transient-state-is-not-a-terminal-state` and `no-stall-inside-a-publication-window` (the
two ends of a two-store publication window), `a-structure-keeps-the-regime-it-was-proved-under`,
`a-queue-without-a-bound-has-no-overload-behaviour`, `verify-ordering-on-the-weakest-target`,
`no-false-sharing-on-a-hot-line`. Memory and the machine: `no-allocation-on-the-hot-path`,
`allocated-is-not-resident`, `no-syscall-on-a-bounded-path`.

And three about instruments that report a **confidently wrong** number rather than a
visibly bad one, which is the group worth reading first: `no-coordinated-omission` (a
load generator that stalls with the system deletes exactly the worst latencies),
`profiler-samples-where-it-can-stop` (the sampler reports where it was permitted to stop,
so the hottest loop can be absent from its own profile), and
`a-measurement-matches-the-regime-it-reports` (stated in both directions — measuring cold
when production is warm, and discarding warm-up on a path that is always cold).

The two reasoning rules that predate the set, `answer-the-requirement-at-its-layer` and
`attack-the-design-in-a-second-pass`, sit above all of it.

## Install

**1. Find the path your Copilot actually reads — do not assume it.** In most
repositories it is:

```
<your-repo>/.github/copilot-instructions.md
```

which Copilot Chat and code completion read in VS Code, Visual Studio, JetBrains,
github.com and the CLI. It is not the only surface, and in a managed corporate
environment it is often not the available one. `.github/` may be owned by a repository
template or a CODEOWNERS entry that rejects new files; the local convention may be
path-scoped `*.instructions.md` files carrying an `applyTo:` header; VS Code can be
pointed at other folders through `chat.instructionsFilesLocations`; and a root
`AGENTS.md` is read by the coding agent. The prompt in the next section makes Copilot
establish which of these applies, and report back, before it moves anything.

**2. Confirm instruction files are enabled.** In VS Code the setting is
`github.copilot.chat.codeGeneration.useInstructionFiles`. A managed or enterprise
profile can pin it off, in which case nothing you do to the file will have any effect —
check this before debugging anything else.

**3. Commit it.** Repository custom instructions apply to everyone working in the
repository, which is the point; a file left uncommitted helps only you.

## Letting Copilot do the install

Into a repository whose layout you already know, copying one file by hand is faster and
cannot go wrong. Use the prompt when you do **not** know the layout — an unfamiliar
repository, or an employer with its own folder schema — because it makes Copilot find
the reader before it creates the file, and stop rather than guess. "Put it where it
should go" is not something it can infer.

```
You are installing a generated instruction file into this repository. It is a build
artifact from another project: copy it, never rewrite it.

STEP 1 — Before moving anything, tell me where instructions actually load from here.
Report each of these, and write "unknown" rather than guessing:
  - which Copilot client this is (VS Code, Visual Studio, JetBrains, github.com, CLI)
    and its version;
  - which of these this workspace already uses: .github/copilot-instructions.md; any
    *.instructions.md file, under .github/instructions/ or anywhere else; a root or
    nested AGENTS.md; instruction paths set in workspace or user settings
    (github.copilot.chat.codeGeneration.instructions,
    github.copilot.chat.codeGeneration.useInstructionFiles,
    chat.instructionsFilesLocations);
  - whether .github/ in this repo is constrained — a CODEOWNERS entry, a repository
    template, or a CI check that fails on unexpected files there.

STEP 2 — Propose ONE target path and wait for my yes. Name the single file path you
would create and one sentence saying what will read it. If this repository already has
a repository-wide instructions file, do not touch it: report its path and line count
and stop, so I decide whether to merge or to scope this one to a subdirectory. If
nothing here establishes a convention, say so and propose your client's documented
default — do not invent a folder.

STEP 3 — Copy, do not author. Place copilot-instructions.md at the agreed path byte for
byte. Do not reformat, reflow, re-order, summarise, rewrite in your own words, split it
across files, or drop the HTML comment on the last line — that comment carries a hash of
the body, and any edit makes the next regeneration a conflict and is silently reverted.
Add front-matter (applyTo:) only if the agreed path is a path-scoped *.instructions.md
file, and only that one key.

STEP 4 — Report the final path, the line count (expected: 746), and confirm that no
other file was created or modified.
```

The expected line count belongs in the prompt because it is the one check the person
pasting it can make without reading the file: 746 for this pack. A single-home build is
shorter — 446 for `global` alone, 200 for `domain-rust` alone — so correct the number if
you narrowed the set (see *Seven rules are about other repositories*, below).

## Confirming it loaded

Do not treat file placement as proof, and do not ask the agent that just placed it — the
file is in its context either way. In a **fresh chat**, ask something only these rules
answer:

> What does R:parse-wide-then-range-check require, and how does it differ from
> R:parse-dont-validate?

A correct answer distinguishes parsing into a type wide enough to *represent* the
out-of-range value from minting the narrow witness at the boundary. In VS Code the reply
also shows a **References** chip naming `copilot-instructions.md`. A generic answer with
no reference chip means the file is not loaded, and the `useInstructionFiles` setting in
step 2 above is the first place to look.

## Three things to know

**Do not hand-edit the instruction file.** It ends with a
`<!-- relearn:generated … sha256=… -->` trailer. The generator refuses to overwrite a
file whose hash does not match, so an edit here turns the next regeneration into a
conflict, and the edit itself is invisible to the rule library — it will be silently
reverted the next time anyone rebuilds. Change the rule in `rules/<tag>.md`, rebuild,
re-copy.

**Some rules describe an enforcement that does not exist outside this project.** Eight
rules are marked *graduated* as of 2026-09-12, and each says its guarantee has moved to
a stronger control — a write-time hook in the rule author's own agent configuration, or
a test or gate inside another of their repositories. In any other environment that
control does not exist, and the note reads as a reason to relax an instruction which is
in fact the only thing enforcing the rule. Treat every graduated rule as fully active
unless you have installed equivalent tooling. Find them with

```bash
grep -n 'Also enforced by' .github/copilot-instructions.md
```

rather than trusting this paragraph to have kept count.

**Seven rules are about other repositories.** `R:generate-guards-unversioned` and
`R:order-by-explicit-rank` are homed in `project-relearn`; `R:no-secrets-in-config-repo`
and `R:verify-tracked-after-move` in `project-stochos-lab`; `R:role-is-an-edge-property`,
`R:seeded-data-needs-a-migration` and `R:verify-the-glyph-exists` in
`project-design-architecture-tool`. They are sound rules and harmless to carry, but they
were written about those codebases. To emit a set without them, build one home at a time
from the repository root:

```bash
cargo run -- build --targets copilot --home global      --out <dir>
cargo run -- build --targets copilot --home domain-rust --out <dir>
```

Concatenate the two results and drop the second `# Copilot instructions` heading.

## Regenerating this pack

From the repository root:

```bash
cargo run -- build  --targets copilot --out copilot-pack   # rewrite it
cargo run -- verify --targets copilot --out copilot-pack   # prove it is in sync
```

CI runs the `verify` line, so a rule change that is not rebuilt into this pack fails the
build rather than shipping a stale instruction file. This README is hand-authored and
sits outside that check — see the note under the table at the top.
