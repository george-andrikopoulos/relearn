# Copilot pack — installation

A self-contained drop-in that loads this repository's rule library into GitHub Copilot
as repository custom instructions. Download this folder, copy one file into your
repository, done.

## What is in here

```
copilot-pack/
├── README.md                          this file
└── .github/
    └── copilot-instructions.md        45 rules, 505 lines — the instruction file
```

The instruction file is **generated** by `relearn build --targets copilot` from the
neutral rule library in [`rules/`](../rules). It carries every rule in the library:

| Layer | Rules | What they cover |
|---|---:|---|
| `global` | 23 | The repository discipline and its controls, plus language-agnostic verification, provenance and cost reasoning — see below |
| `domain-rust` | 18 | The complete Rust type-driven design discipline — see below |
| `project-*` | 4 | Rules belonging to the `relearn` and `stochos-lab` repositories specifically |

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
`reconcile-wiring-at-start` (a correctly-wired control can go dark and say nothing),
`no-stale-push-over-fresh` (a mirror script must know which side is authoritative),
`detector-excludes-own-definitions` (a self-matching check is always red, so it gets
muted), `revision-integrity` (restructuring silently breaks references the author cannot
see), `case-collision` (two names differing only by case are one file on NTFS).

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

## Install

**1. Copy the file into the target repository, keeping the path exactly:**

```
<your-repo>/.github/copilot-instructions.md
```

Create `.github/` if it does not exist. The filename and directory are what Copilot
looks for; neither is configurable.

**2. Confirm instruction files are enabled.** In VS Code, the setting is
`github.copilot.chat.codeGeneration.useInstructionFiles`. A managed or enterprise
profile can pin it off, in which case nothing you do to the file will have any effect —
check this before debugging anything else.

**3. Commit it.** Repository custom instructions apply to everyone working in the
repository, which is the point; a file left uncommitted helps only you.

## Letting Copilot do the install

The install is one file into one known path, so doing it by hand is faster and cannot go
wrong. If you would rather have Copilot place it, be explicit — "put it where it should
go" is not something it can infer. Paste this:

> Move the file `copilot-instructions.md` from this workspace to
> `.github/copilot-instructions.md`, creating the `.github` directory if it does not
> exist. Do not modify, reformat, reflow, summarise, or re-order its contents — it is a
> generated artifact and must be copied byte for byte, including the HTML comment on the
> final line. Do not create any other instruction files, and do not add an `AGENTS.md`.
> When you are done, report the file's path and its line count, which should be 477.

## Confirming it loaded

Do not treat file placement as proof. Ask Copilot Chat something only these rules answer:

> What does R:parse-wide-then-range-check require, and how does it differ from
> R:parse-dont-validate?

A correct answer distinguishes parsing into a type wide enough to *represent* the
out-of-range value from minting the narrow witness at the boundary. In VS Code the reply
also shows a **References** chip naming `copilot-instructions.md`. A generic answer with
no reference chip means the file is not loaded, and step 2 is the first place to look.

## Three things to know

**Do not hand-edit the instruction file.** It ends with a
`<!-- relearn:generated … sha256=… -->` trailer. The generator refuses to overwrite a
file whose hash does not match, so an edit here turns the next regeneration into a
conflict, and the edit itself is invisible to the rule library — it will be silently
reverted the next time anyone rebuilds. Change the rule in `rules/<tag>.md`, rebuild,
re-copy.

**Some rules describe an enforcement that does not exist outside this project.**
`R:no-unwrap-in-production`, `R:no-anyhow-in-libraries` and `R:case-collision` are marked
*graduated*, and each says its guarantee has moved to a write-time hook, so the
instruction layer no longer has to hold it. Those hooks live in the rule author's own
agent configuration. In any other environment there is no hook, and the note reads as a
reason to relax an instruction that is in fact the only thing enforcing the rule. Treat
every graduated rule as fully active unless you have installed equivalent tooling — grep
the file for `Also enforced by` to find them, rather than trusting this paragraph to have
kept count.

**Four rules are about other repositories.** `R:generate-guards-unversioned`,
`R:order-by-explicit-rank`, `R:no-secrets-in-config-repo` and
`R:verify-tracked-after-move` are homed in `project-relearn` and `project-stochos-lab`.
They are sound rules and harmless to carry, but they were written about those codebases.
To emit a set without them, build one home at a time from the repository root:

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
build rather than shipping a stale instruction file.
