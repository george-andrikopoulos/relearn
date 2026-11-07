<div align="center">

# relearn

**Corrections become rules, rules climb towards checks, and every recurrence is counted.**

[![Release](https://img.shields.io/github/v/tag/george-andrikopoulos/relearn?label=release&color=3366cc)](https://github.com/george-andrikopoulos/relearn/tags)
[![Licence](https://img.shields.io/badge/licence-Apache--2.0-b8860b)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-edition%202024-c0392b?logo=rust)](Cargo.toml)
[![arXiv](https://img.shields.io/badge/arXiv-2608.19125-555)](https://arxiv.org/abs/2608.19125)

[Quick start](#quick-start) · [**Manual**](docs/manual.md) · [relearn and rulesync](#relearn-and-rulesync) · [How it works](#how-it-works) · [What was measured](#what-was-measured) · [Papers](#papers)

</div>

---

When you correct an AI assistant, the correction usually dies with the session, and the same class of mistake comes back next week. Writing it into an instruction file helps until the day the model reads the file and makes the mistake anyway. Nothing tells you that happened, and nothing tells you which of your rules are doing any work.

**relearn treats a correction as the start of a rule's life, not a line of text.**

- **Provenance.** Each correction becomes one rule file recording the incident, the date and the error class it is meant to retire.
- **What holds it.** The rule records what holds it: prose alone, or a hook, test, gate or type. If a check covers only part of it, the rule must name the part still exposed.
- **Recurrences.** When the error class comes back, the recurrence is recorded on that rule. `relearn lint` flags any rule that keeps failing while only prose holds it.
- **Retirement.** A rule that is no longer needed is retired with its reason, not deleted.

relearn also compiles each rule into every assistant's own format, so one correction reaches Claude, Cursor, Copilot and anything that reads `AGENTS.md`:

<table>
<tr>
<td width="50%" valign="top">

**You write** `rules/no-dbg-in-src.md`

```
+++
tag         = "R:no-dbg-in-src"
title       = "No dbg! in committed source"
error_class = "Debug output reaches users' stderr"
home        = { kind = "domain", name = "rust" }
created     = "2026-11-02"
origin      = "codified"
status      = { kind = "active" }
incident    = "dbg! prints in release builds too."
+++

Remove `dbg!` before committing; use a test
or a log line.
```

</td>
<td width="50%" valign="top">

**relearn writes**

| File | Read by |
|---|---|
| `skills/domain-rust/SKILL.md` | Claude |
| `.cursor/rules/no-dbg-in-src.mdc` | Cursor |
| `.github/copilot-instructions.md` | GitHub Copilot |
| `AGENTS.md` | any assistant that reads `AGENTS.md` |
| `.claude/rules/<home>.md` | Claude Code's rules layer (for project and user homes) |

Each output file is stamped with a hash and is never edited by hand. `relearn verify` fails if one drifts from its rule.

</td>
</tr>
</table>

## relearn and rulesync

[rulesync](https://github.com/dyoshikawa/rulesync) also compiles one set of rules into many assistants' formats, and it covers much more of each tool's configuration than relearn does. Besides rules, it handles commands, MCP servers, ignore files, subagents and skills, and it can import the configuration you already have. If you need one source for all of that, use rulesync.

relearn compiles to fewer targets, because the compiler is not the point. It is built around what happens to a rule after it is written:

- where the rule came from
- what enforces it
- whether it is still failing
- when it was retired, and why

A rule file without provenance does not parse. Detectors compile into hooks and git gates and are tested against their own fixtures. Probes measure whether a rule changes what a model does. If you have only a few rules and the model follows them, you probably do not need relearn.

## Highlights

- 🔁 **Recurrence is the metric.** When a rule's error class happens again, the rule failed, and the event is recorded on that rule. `lint` reports any rule that keeps failing while only prose holds it.
- 🪜 **Rules climb towards mechanical controls.** A rule's status records whether prose alone holds it, or a hook, test, gate or type holds part or all of it. A rule that is only partly held must name what is not held.
- 🪝 **Checks become hooks and gates.** A rule can carry a detector with fixtures. relearn compiles it into a Claude Code hook or a git gate, and proves it against its fixtures on every build.
- 📏 **Rules can be measured.** A probe replays a rule's error class in fresh sessions, with the rule loaded and without it, and reports the difference with its confidence interval ([what the first run found](#what-was-measured)).
- 🧾 **Provenance or it doesn't parse.** Every rule needs a date, an incident and the error class it retires. A rule that fails to parse stops the build, because a silently skipped rule is a lost correction.
- 🏠 **One home per rule.** A rule has exactly one owner: `global`, a domain, an org or a project. A second copy is how a rule forks and goes stale, so relearn cannot represent one.
- 🔒 **Instruction files are checked like code.** `build` refuses to overwrite a file it did not write, and `verify` fails on any hand edit or drift.
- 👤 **Solo first.** No account, no config file, no network, no ambient state. Sharing between installs exists, and it is optional.

## Quick start

```sh
cargo install --git https://github.com/george-andrikopoulos/relearn

relearn new --tag R:no-dbg-in-src --title "No dbg! in committed source" \
  --error-class "Debug output reaches users' stderr" --home domain=rust \
  --on 2026-11-02 --origin codified \
  --incident "dbg! prints in release builds too." \
  --body 'Remove `dbg!` before committing; use a test or a log line.'

relearn check            # validate every rule and run every detector's fixtures
relearn build --out .    # compile to every assistant's layer
relearn verify --out .   # in CI: fail if a generated file was edited or is stale
```

**The [manual](docs/manual.md) covers everything else:**
- the full rule format
- every target and command
- detectors, hooks and gates
- probes
- sharing rules between installs
- CI setup, exit codes and troubleshooting

**To start a new project with relearn, or to bring an existing project under the five standing documents and move it forward**, follow [Adopting relearn](docs/adopting-relearn.md). It gives every step with the prompt to paste into your assistant, the answer to expect, and the answers that sound right but aren't.

## How it works

In an agentic workflow the model is the unpredictable part. Everything around it is the **harness**: the instructions the model loads, the tools it may call, and the hooks and gates that check its work. relearn owns one piece of the harness, the instruction layer, and the path that carries a human correction back into it.

```mermaid
flowchart TD
    rules[("rules/*.md<br/>one file per rule, provenance required")]
    layers["HARNESS · instruction layers<br/>Claude skills · Cursor rules · Copilot · AGENTS.md · Claude rules layer"]
    model(["Model, any assistant<br/>stochastic"])
    tools["Tools"]
    controls["HARNESS · deterministic controls<br/>types · property tests · gates · hooks"]
    review["Expert review"]
    correction["Correction"]

    rules -->|"relearn build<br/>relearn verify: emitted == source"| layers
    layers -->|"loaded at session start"| model
    model <-->|"tool calls and results"| tools
    model -->|"change or answer"| controls
    controls -->|"what the controls let through"| review
    review -->|"an error gets through"| correction
    correction -->|"relearn new, or a recurrence"| rules

    classDef harness fill:#fff4d6,stroke:#b8860b,color:#222
    classDef source fill:#e6f0ff,stroke:#3366cc,color:#222
    classDef stochastic fill:#fde8e8,stroke:#c0392b,color:#222
    class layers,controls harness
    class rules source
    class model stochastic
```

### The error loop

```mermaid
flowchart TD
    A["Incident<br/>an error reaches review"] --> B["Generalise<br/>from the instance to the error class"]
    B --> C{"Search first<br/>does a rule already cover this class?"}
    C -->|"no"| D["Write the rule<br/>tag · error class · incident · home · date"]
    C -->|"yes, so that rule failed"| R["Record a recurrence<br/>on the existing rule, dated"]
    D --> E["Push it down the hierarchy of controls<br/>type › property test › gate or hook › prose"]
    E --> F["Compile and verify<br/>relearn build · relearn verify"]
    F --> G["Every later session loads it"]
    G -->|"the class returns"| A
    R --> L["relearn lint<br/>unheld recurrence"]
    L -->|"strengthen the control,<br/>do not reword the prose"| E
```

Prose is the weakest control, because a model can read it and still do the wrong thing. A rule's `status` records how far it has been pushed:

| Status | What holds the error class | In the emitted layer |
|---|---|---|
| `active` | prose alone | emitted |
| `partial` | a named control holds part of it, and `uncovered` names the rest | emitted, with "Partly enforced by …" |
| `graduated` | a named control holds all of it | emitted, with "Also enforced by …" |
| `attic` | retired | not emitted; kept so `lint` can flag references to it |

Behind most corrections is a decision the assistant made for itself, on no evidence in the repository. Paper 3 calls these **invented requirements**. Their visible residue is **skedasis** (from σκεδάννυμι, to scatter): the fraction of a change that nobody requested. Skedasis is a ratio, not a size. Two hundred lines is not skedasis if the task needed two hundred. Writing the correction down retires an invented requirement once, instead of having it invented again every session.

## What's in the box

| Command | What it does |
|---|---|
| `new` · `check` · `list` | author a rule, validate the library, read the corpus as text or TSV |
| `build` · `verify` | compile to each target, and prove the committed output matches the rules |
| `lint` | advisory findings: overlaps, dangling references, unheld recurrences, graduation candidates |
| `controls` | resolve every control a rule names: does it exist, and does anything run it? |
| `budget` · `loads` | what a Claude Code session loads at start, and what it actually loaded over time |
| `probe` | judge sessions that replay a rule's error class, with the rule loaded and without it |
| `contribute` · `pull` · `adopt` · `report` · `aggregate` | optional sharing between installs through a common directory |

The repository also commits ready-to-install packs, each with its own README:
- [`claude-pack/`](claude-pack/): skills, for Claude Code and claude.ai.
- [`copilot-pack/`](copilot-pack/): Copilot instructions.
- [`rules-pack/`](rules-pack/): user-level Claude rules.
- [`hooks-pack/`](hooks-pack/): Claude Code hooks.
- [`repo-pack/`](repo-pack/): git gates.
- [`load-log-pack/`](load-log-pack/): the load-logging hook.

A real rule set lives in [`rules/`](rules/). It spans a global layer, language and discipline domains (`rust`, `java`, `low-latency`, `disclosure`, `agent-tooling`, `project-files`) and several project homes. Run `relearn check` for its current size. A count written here would go stale ([`R:doc-currency`](rules/doc-currency.md)).

## What was measured

Two kinds of evidence exist so far. Neither is an efficacy claim for the discipline as a whole.

**Probes: the same task with the rule loaded and without it.** On 2026-10-31, seven rules were replayed on `claude-opus-5-5`. Each rule had ten sessions per arm, on a synthetic task built to provoke its error class: 140 sessions in all, costing $13.52.

One rule separated the arms. Without [`R:prefer-by-construction`](rules/prefer-by-construction.md), 9 of 9 sessions made the error. With it, 2 of 9 did, a reduction of 0.78 (95% CI 0.34 to 0.94). All six sessions that actually opened the rule held.

The other six rules gave no result:
- **Three:** the model did not make the error, with the rule or without it.
- **Two:** their probes were defective.
- **One:** no session opened the skill that carries the rule. That is a finding about how the rule is delivered, not about what it says.

The full table and the reading of each transcript are in [`docs/control-triage.md`](docs/control-triage.md#results-2026-10-31-the-first-pilot). This was one model, with small samples, on one author's rules. It shows that the instrument can separate the two arms, and that one rule changed what one model did on one task. It does not show that rules written this way work in general.

**Paper 3: repository history.** Paper 3 pre-registered its metrics and its falsification conditions before reading any commit history, and **its headline hypothesis was refuted.** Across nine of the author's repositories, the discipline leaves no mark on the size of a change: median lines added per code-changing commit fall in a common band whether a repository is governed or not. The paper puts this down to the instrument, since skedasis is a ratio and commit size measures only its numerator. Even so, the prediction failed, and the paper reports it as a failure. Capture of deferred work is the one metric that separated the two groups. The parking lot fires in a fifth to two-fifths of code-changing commits in governed repositories, and in under five per cent in ungoverned ones.

relearn is in the governed arm of that comparison. It is **excluded** from the paper's hand-coded study of invented requirements, because its sessions are mixed into one transcript store with hundreds of unrelated ones. **Nothing here should be read as an efficacy claim.** The discipline is implemented. The evidence from history is one refuted prediction, one correlational separation, and a single-author corpus that cannot establish causation either way.

To see one real incident become one rule and compile out to every layer, read [`docs/worked-example.md`](docs/worked-example.md). For the same rule passed between two installs, read [`docs/worked-example-two-installs.md`](docs/worked-example-two-installs.md).

## Contributing to this repository

This repository is **public**, and a rule's `incident` is a verbatim quotation from a private session. Arm the pre-push gate once per clone:

```sh
git config core.hooksPath .githooks
```

The gate runs `scripts/check.sh` and the banned-names scan. The scan reports a location and a count, never the term itself, and exits 2 rather than 0 when it has no term list. The list is deliberately not committed: it is read from `$BANNED_TERMS_FILE` or `~/.claude/usage/banned-terms.sha256`. Nothing in the repository can check that your clone did this, so run the line before your first push.

To learn how the code is built, read [`ARCHITECTURE.md`](ARCHITECTURE.md) for the pipeline and the decisions log, [`FEATURES.md`](FEATURES.md) for every guarantee and the artefact that enforces it, and [`CLAUDE.md`](CLAUDE.md) for the full recreation standard.

## Papers

relearn is the reference implementation of the error loop in a series of three papers by George Andrikopoulos (2026):

- **Tuning the Stochastic Machine** ([arXiv:2608.19125](https://arxiv.org/abs/2608.19125)): the operating discipline. relearn implements the Stochos framework's first two principles, *persist or perish* and *one home per rule*.
- **Grouping the Stochastic Machine** ([arXiv:2608.19140](https://arxiv.org/abs/2608.19140)): the precision-measurement companion. It uses this repository as its reference system.
- **Aiming the Stochastic Machine: A Repository Discipline for First-Time-Right, and What Survived Measuring It** ([doi:10.5281/zenodo.22083202](https://doi.org/10.5281/zenodo.22083202); study materials at [doi:10.5281/zenodo.22082967](https://doi.org/10.5281/zenodo.22082967)). It specifies the four-file repository discipline this repository is built on, and uses this repository as its worked case (§8), including a defect found here. Paper 3 is a Zenodo preprint: arXiv moderation declined it, and will consider an appeal only after journal publication.

## Licence

Apache-2.0, see [`LICENSE`](LICENSE). The explicit patent grant matters for anything an enterprise might adopt.

<sub>George Andrikopoulos · United Kingdom</sub>
