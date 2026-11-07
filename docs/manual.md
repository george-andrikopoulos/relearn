# The relearn manual

This manual covers relearn from installing it to running it across a team. The [README](../README.md) says what relearn is and why it exists. This page says how to use it. For step-by-step prompts to give your assistant, see [Adopting relearn](adopting-relearn.md). It covers starting a new project with relearn, and bringing an existing project under the five standing documents.

**Contents**

- [Install](#install)
- [Concepts](#concepts)
- [Your first rule](#your-first-rule)
- [The rule file](#the-rule-file)
- [Compiling to assistants](#compiling-to-assistants)
- [Keeping the library honest](#keeping-the-library-honest)
- [Mechanical controls: detectors, hooks and gates](#mechanical-controls-detectors-hooks-and-gates)
- [Probes: measuring whether a rule works](#probes-measuring-whether-a-rule-works)
- [Sharing rules between installs](#sharing-rules-between-installs)
- [Running relearn in CI](#running-relearn-in-ci)
- [Exit codes](#exit-codes)
- [Troubleshooting](#troubleshooting)

---

## Install

relearn is a single Rust binary with no runtime dependencies, no configuration file, and no network access.

```sh
git clone https://github.com/george-andrikopoulos/relearn
cd relearn
cargo install --path .        # puts `relearn` on your PATH
relearn --version
```

You need a Rust toolchain that supports edition 2024. The generated hooks and gates are bash scripts. The hooks also need `jq`, and the gates need `git` and `awk`. On Windows, run them under Git Bash.

Every command reads the paths you give it and writes only where you tell it. Nothing is read from your home directory, the environment or a clock, so a corpus compiles identically on any machine.

---

## Concepts

| Term | Meaning |
|---|---|
| **Rule** | One correction, written down once: a file in `rules/`, holding front matter and a markdown body. A rule is the source. Every assistant file is built from it. |
| **Tag** | A rule's identity, written `R:kebab-case-name`. It is unique across the library and appears in every emitted file, so you can trace any instruction back to its source. |
| **Error class** | The general failure the rule retires. It is not the single incident that triggered the rule. If you cannot state a class, what you have is reference material, not a rule. |
| **Incident** | Why the rule exists: the dated event that produced it, or a note saying where the practice came from. Every rule must have one. |
| **Home** | Who owns and maintains the rule. There is exactly one: `global`, `domain=<name>`, `org=<name>` or `project=<path>`. |
| **Audience** (`applies_to`) | Who should load the rule, for example `rust` or `java`. A rule may name several audiences. An audience is not a second home. |
| **Status** | How far the rule has moved from prose towards a mechanical control: `active`, `partial`, `graduated` or `attic`. |
| **Control** | Something stronger than prose that enforces the rule: a `type`, a `test`, a `gate` or a `hook`. |
| **Recurrence** | A dated later occurrence of an error class that a rule already covers. Recurrence is the measure of whether the rule is working. |
| **Target** | An output format: Claude skills, Cursor rules, Copilot instructions, `AGENTS.md`, a Claude rules layer, hooks or gates. |

The loop relearn supports:

1. An error gets through review.
2. You name its class and search the library.
3. If no rule covers the class, you **write a rule**. If one does, that rule failed, and you **record a recurrence** on it.
4. You push the rule towards the strongest control that can hold it.
5. `relearn build` compiles it into every assistant's layer, and `relearn verify` proves the layer matches.
6. `relearn lint` reports any rule that keeps recurring while only prose holds it.

---

## Your first rule

The walk-through below runs in an empty directory. The output shown is real.

**1. Scaffold the rule.** `relearn new` writes the front matter, which is the mechanical part. You supply the body and the incident, which are the rule itself.

```sh
mkdir rules
relearn new --tag R:no-dbg-in-src \
  --title "No dbg! in committed source" \
  --error-class "Debug output left in shipped code reaches users' stderr" \
  --home domain=rust --on 2026-11-02 --origin codified \
  --incident "Standing practice: dbg! prints in release builds too." \
  --body 'Remove `dbg!` before committing; use a test or a log line.'
```

```
wrote rules/no-dbg-in-src.md — run `relearn check` before you trust it
```

A paragraph is easier to write in an editor than in a shell, so for real rules use `--incident-file` and `--body-file`. Add `--dry-run` to print the document without writing it. `new` never overwrites anything, including an existing rule with the same tag.

**2. Check it.**

```sh
relearn check
```

```
ok: 1 rule(s) validated; every detector's fixtures pass, and none fires on a rule file or an emitted file
```

**3. Build every target.**

```sh
relearn build --out .
```

```
wrote 4 file(s) under .
```

```
.cursor/rules/no-dbg-in-src.mdc        Cursor, attached to **/*.rs
.github/copilot-instructions.md        GitHub Copilot
AGENTS.md                              any assistant that reads AGENTS.md
skills/domain-rust/SKILL.md            a Claude skill for the rust domain
```

Every emitted file ends with a stamp recording which rules produced it, and a hash of the content:

```
<!-- relearn:generated v0.1.0 sha256=7ba6…68fd rules=R:no-dbg-in-src -- DO NOT EDIT; regenerate with `relearn build` -->
```

**4. Verify.** Verification reads only and writes nothing. This is the command CI runs.

```sh
relearn verify --out .
```

```
ok: 4 generated file(s) up to date
```

If someone edits a generated file by hand, `verify` reports it:

```
AGENTS.md: stale — does not match the current rules; run `relearn build` to regenerate
1 of 4 generated file(s) drifted
```

`build` refuses to overwrite a file it did not write, and it aborts before touching disk:

```
error: ".github/copilot-instructions.md" exists but was not generated by relearn (no generated-by marker); refusing to overwrite unversioned content
```

**5. Inspect.**

```sh
relearn list
relearn lint
```

```
R:no-dbg-in-src  [domain-rust]  No dbg! in committed source
ok: no lint findings
1 rule(s): 0 recurred, 1 inert (codified and never fired)
```

*Inert* means the rule was written down from practice and has never fired. Such a rule is not evidence of anything yet, and `lint` prints the inert count so that a library cannot grow its credibility simply by growing its size.

---

## The rule file

A rule is a markdown file with TOML front matter between `+++` lines. The parser reads it strictly. Anything that fails to parse stops the build, because a rule that is silently skipped is a correction lost.

```
+++
tag         = "R:parse-wide-then-range-check"
title       = "Parse wide, then range-check"
error_class = "Range-check collapses out-of-range into not-a-number"
home        = { kind = "domain", name = "rust" }
created     = "2026-07-23"
origin      = "mined"
status      = { kind = "active" }
incident    = "Grouping task 01: 5/5 samples parsed into u16, so 70000 read as NotANumber."
+++

Parse into a type wide enough to represent the out-of-range value, then range-check.
```

### Required keys

| Key | Form | Notes |
|---|---|---|
| `tag` | `"R:kebab-case"` | Unique across the library. |
| `title` | string | The practice, in the imperative. It heads the rule in every emitted format. |
| `error_class` | string | What goes wrong without the rule, and the harm you can observe. |
| `home` | table | See [Homes](#homes). |
| `created` | `"YYYY-MM-DD"` | Must be a real date. `2026-02-30` is refused. |
| `origin` | `"mined"`, `"codified"` or `"mandated"` | See [Origin](#origin). |
| `status` | table | See [Status](#status). |
| `incident` | string | The provenance. For a mined rule it is a quotation from a private session, and it never leaves the machine (see [Sharing](#sharing-rules-between-installs)). |

### Optional keys

| Key | Form | Notes |
|---|---|---|
| `applies_to` | `["rust", "java"]` | The audiences that should load the rule. Omit it and every audience loads the rule. |
| `source` | string | Only allowed on a `codified` rule: the page, paper or clause it was written down from. |
| `approval` | `{ by, date, control }` | Required on a `mandated` rule and refused on any other origin. |
| `published_incident` | string | The incident text written for publication. It is required before `contribute` will publish the rule. |
| `[[recurrence]]` | `date`, `incident` | One table for each later occurrence of the class. |
| `[[detector]]` | table | A mechanical check. See [Detectors](#detectors). |
| `[probe]` | table | A replayable measurement. See [Probes](#probes-measuring-whether-a-rule-works). |

`pull` and `adopt` also write authority metadata into a cached or forked rule. Leave that alone. It is how relearn knows a rule is a cache.

### Homes

The front-matter form, which `relearn new --home` also takes, differs from the *slug* that `list --home` and `build --home` take:

| Home | Front matter | `new --home` | Slug |
|---|---|---|---|
| Global | `{ kind = "global" }` | `global` | `global` |
| Domain | `{ kind = "domain", name = "rust" }` | `domain=rust` | `domain-rust` |
| Org | `{ kind = "org", name = "acme" }` | `org=acme` | `org-acme` |
| Project | `{ kind = "project", path = "..." }` | `project=<path>` | `project-<slug>` |

A slug cannot be turned back into a project path, which is why `new` refuses one.

An **org** rule never leaves the machine. A **project** rule is withheld from sharing too, because its home records a filesystem path.

**Home or audience?** Home answers *who maintains this rule*. `applies_to` answers *who should load it*. A low-latency principle needed by both Rust and Java engineers is a single file:

```
home       = { kind = "domain", name = "low-latency" }
applies_to = ["rust", "java"]
```

An audience names a group of readers (`rust`, `java`, `embedded`), never a topic (`performance`, `security`). The topic belongs in `error_class`.

### Origin

| Origin | Meaning | Counts as evidence? |
|---|---|---|
| `mined` | A real failure produced it. This is the default for `new`. | Yes |
| `codified` | Standing practice, written down. May carry a `source`. | Only once it recurs |
| `mandated` | A control framework requires it. Carries an `approval`. | No, and it is excluded from the statistics |

```
origin   = "mandated"
approval = { by = "J. Doe", date = "2026-10-01", control = "SEC-12" }
```

### Status

| Status | Front matter | What holds the class | Emitted? |
|---|---|---|---|
| Active | `{ kind = "active" }` | prose alone | yes |
| Partial | `{ kind = "partial", by = "hook:x", uncovered = "...", date = "..." }` | a control holds part of it, and `uncovered` names the rest | yes, with a "Partly enforced by …" note |
| Graduated | `{ kind = "graduated", to = "hook:x", date = "..." }` | a control holds all of it | yes, with an "Also enforced by …" note |
| Attic | `{ kind = "attic", reason = "...", date = "..." }` | retired | no, but the rule is kept so `lint` can flag references to it |

A control is written `kind:name`, where the kind is `hook`, `test`, `gate` or `type`. Several controls are joined with ` + `, and each may carry a note in parentheses:

```
by = "hook:gate-verdict-intact (shell pipeline half) + hook:multiline-pattern-eol"
```

**Prefer `partial` to `graduated`.** A graduated rule claims that its control covers the *whole* class. That claim is printed into every assistant's layer, so the next time the class recurs outside the control, the record is wrong. A partial rule that does not name its gap will not parse.

`relearn new` always writes `active`. A rule that is graduated on the day it is written records nothing that anyone learned.

### Recurrences

When an error class that a rule covers happens again, the rule failed. Record that on the rule. Do not write a second rule beside it, and do not edit the original incident:

```
[[recurrence]]
date = "2026-09-13"
incident = "The gate built for this class missed the README's hand-written count."
```

`lint` reports an **unheld recurrence** whenever prose alone holds a rule that has recurred. The remedy it names is a stronger control, not firmer wording. Rules that have recurred are listed first in the emitted skill description.

---

## Compiling to assistants

```sh
relearn build [--rules rules] [--out .] [--targets ...] [--home SLUG] [--scope AUDIENCE] [--rules-layer ...]
```

### Targets

| Target | Writes | Read by | Default? |
|---|---|---|---|
| `claude` | `skills/<home>/SKILL.md` | Claude Code and claude.ai skills | yes |
| `cursor` | `.cursor/rules/<tag>.mdc` | Cursor | yes |
| `copilot` | `.github/copilot-instructions.md` | GitHub Copilot | yes |
| `agents` | `AGENTS.md` | assistants that read `AGENTS.md` | yes |
| `claude-rules` | `.claude/rules/<home>.md` | Claude Code's rules layer | yes |
| `copilot-paths` | `.github/instructions/<home>.instructions.md` | Copilot, path-scoped by `applyTo` | explicit only, and an alternative to `copilot` |
| `hooks` | `hooks/<name>.sh`, a runner, a settings snippet | Claude Code hooks | explicit only |
| `repo` | `gates/<name>.sh`, `gates/relearn-pre-commit.sh`, a runner | a repository's own gate | explicit only |
| `load-log` | `hooks/relearn-load-log.sh`, a runner, a settings snippet | Claude Code hooks | explicit only |

Pass `--targets` as a comma-separated list. An unknown target is rejected before anything is written.

The three targets that produce executable scripts are never built by default. Building them writes scripts; it does not install them. A person installs them.

### Narrowing

- **`--home <slug>`** emits only one home. A slug that no rule uses is an error, not an empty build.
- **`--scope <audience>`** narrows by audience and can be repeated. A rule with no `applies_to` is emitted whatever audience you ask for, so narrowing can never withhold it. An audience that no rule declares is an error.

### The Claude rules layer

Claude Code loads `.claude/rules/` from where the files are written, so `claude-rules` writes exactly one layer:

```sh
# A project's own tree: that project's home and nothing else.
# By default the project is named after the --out directory.
relearn build --targets claude-rules --out . --rules-layer project=myrepo

# The user layer: every domain home, for ~/.claude/rules/
relearn build --targets claude-rules --rules-layer user --out ~
```

Do not write another project's home into this tree. Claude Code would load it in every session here. Do not write domain homes into a project either: they already load from the user layer, so they would load twice. `relearn budget` reports both mistakes.

### Generated files are safe to share a directory with

- Every emitted file carries a generated-by marker and a content hash.
- `build` refuses to overwrite a file that lacks the marker, and aborts the whole run before writing anything.
- Re-running `build` on unchanged rules produces byte-identical output.
- Never edit an emitted file by hand. Edit the rule and rebuild.

### `verify`

```sh
relearn verify --out . [same --targets / --home / --scope / --rules-layer as build]
```

`verify` recomputes what `build` would write and reports each file that is missing, hand-edited or stale, or that is shadowed by an unversioned file. It writes nothing. Pass it the same narrowing flags that the matching `build` used.

---

## Keeping the library honest

### `check`: validate everything

```sh
relearn check
```

`check` parses every rule, runs every detector's fixtures, then runs every detector over every rule file and every emitted file. A detector that fires on its own rule's text would block the very edit that maintains it, so that counts as a failure.

### `list`: the corpus as data

```sh
relearn list                         # tag  [home]  title
relearn list --home domain-rust
relearn list --scope java
relearn list --format tsv            # tag, home, created, origin, status, controls, recurrences, last_recurrence
```

The TSV output starts with a `#` header that names each column. Select fields by name, not position. `incident` is deliberately left out of the TSV.

### `lint`: advisory findings

```sh
relearn lint                         # fails on any warning
relearn lint --deny error            # fails only on findings that make the emitted tree wrong
```

`lint` reports overlapping scopes, home-slug collisions, dangling references (including references to atticked rules), unheld recurrences, graduation candidates (a rule whose detector passes but which its status does not name) and hooks that no detector or control map places. It writes nothing.

### `controls`: is each named control real and running?

A `partial` or `graduated` rule names its controls. `controls.toml`, kept beside `rules/`, says where each control lives:

```toml
here = "myrepo"                      # the root this file sits in

[[control]]
control = "test:tests/ledger.rs"
root = "myrepo"
path = "tests/ledger.rs"             # wired when the file holds a running #[test]

[[control]]
control = "gate:scripts/no-banned-names.sh"
root = "myrepo"
path = "scripts/no-banned-names.sh"
invoked_by = ".githooks/pre-push"    # wired when this file runs the script

[[control]]
control = "hook:session-wiring-check"
root = "dotclaude"
path = "hooks/session-wiring-check.sh"   # wired when --settings registers it

[[proposed]]                         # a candidate that the rule does not name yet
rule = "R:verify-tracked-after-move"
control = "gate:scripts/verify-deploy.sh"
root = "dotclaude"
path = "scripts/verify-deploy.sh"
invoked_by = ".githooks/pre-push"
```

For a `test` control, adding `function = "<name>"` makes the resolver search for that test function. For a `gate` control, `function` names a shell function inside the script. A `type` control has no resolver and always reads UNCHECKABLE.

```sh
relearn controls --root dotclaude="$HOME/.claude" --settings "$HOME/.claude/settings.json"
```

Write `$HOME`, not `~`: the shell does not expand a tilde that follows `name=`.

Each control gets one verdict: `RESOLVED_AND_WIRED`, `RESOLVED_NOT_WIRED`, `MISSING`, `UNMAPPED` or `UNCHECKABLE`. The map is checked in both directions: a control that no rule names is a `STALE_MAP_ENTRY`. If you did not supply a root, or the settings file, the run exits **2**, never 0. A check that could not run is not a pass.

### `budget`: what a session actually loads

```sh
relearn budget --project . --user ~/.claude
relearn budget --project . --project-only --project-budget 40000   # the hermetic CI form
```

`budget` lists every instruction file that a Claude Code session in the project loads at start. For each one it gives the size as the loader counts it, whether the file is always resident or path-scoped, and whether relearn generated it. It reports:
- `OVER_LIMIT` and `OVER_BUDGET`
- `DUPLICATE_LAYER`: the same home loaded twice
- `FOREIGN_PROJECT_LAYER`: another project's rules loaded here

Without `--user`, the user share is reported as *not measured*, never as zero.

### `loads`: what was actually loaded

Install the load-log hook (`relearn build --targets load-log --out load-log-pack`, then follow `load-log-pack/README.md`). After that you can read the log against what `build` would emit now:

```sh
relearn loads --log ~/.local/state/relearn/load-log.jsonl --since 2026-10-01 --until 2026-10-31
```

| Verdict | Meaning |
|---|---|
| `COLD` | The home never loaded in the window. |
| `STALE` | The home loaded with content that `build` would no longer write, and the verdict says from where. |
| `SHADOWED` | The home loaded from two locations. |
| `LOADED` | Counts of homes that loaded correctly. |

---

## Mechanical controls: detectors, hooks and gates

Prose is the weakest control. If a rule is precise enough to check mechanically, give it a **detector**: the check itself, together with fixtures that prove it works.

### Detectors

```
[[detector]]
name    = "no-dbg-in-src"
engine  = "line-regex"
pattern = 'dbg!\('
comment = "//"                 # lines that are comments are ignored
allow   = "allow:dbg"          # a line carrying this token is allowed
files   = ["src/**/*.rs"]
exclude = ["src/bin/scratch.rs"]
events  = ["claude-code-pre-tool-use"]
action  = "block"              # or "warn"
message = "Remove the dbg! before writing; use a test or a log line."

[[detector.fixture]]
path   = "src/lib.rs"
input  = 'let x = dbg!(y);'
expect = "hit"

[[detector.fixture]]
path   = "src/lib.rs"
input  = '// dbg!(y) is banned here'
expect = "clean"
```

- A detector needs at least one `hit` fixture and one `clean` fixture. Its fixtures run *while the rule parses*, so a fixture that disagrees with the pattern stops the build.
- The same pattern runs in Rust at `check` time and under `grep -E` inside the hook. Constructs the two engines read differently are refused: `\d`, `(?i)`, lazy quantifiers and anything non-ASCII. Use POSIX classes such as `[[:space:]]` instead.
- A detector never changes its rule's status. `lint` calls the rule a graduation candidate. Whether the detector covers the whole class is your decision.

| Engine | Checks | Events |
|---|---|---|
| `line-regex` | each line of a written file | Claude Code hooks, pre-commit |
| `sections` | every heading at level `heading` carries `require` | Claude Code hooks, pre-commit |
| `sibling-case` | a new file whose name differs from a sibling's only in case | Claude Code hooks |
| `listed-path` | a staged file appears on a machine-local list | pre-commit |
| `eol-attribute` | executable text whose line ending is not pinned in `.gitattributes` | repository gate |
| `root-documents` | documents at the repository root beyond the allowed set | repository gate |
| `duplicate-paragraph` | a paragraph copied verbatim into a second file | repository gate |
| `lint-enabled` | a required compiler lint that is not enabled | repository gate |

| Event | Becomes |
|---|---|
| `claude-code-pre-tool-use` | a hook that runs before `Write`, `Edit` and `MultiEdit` |
| `claude-code-post-tool-use` | a hook that rereads the file after an edit (warn only) |
| `git-pre-commit` | a branch in the commit dispatcher `repo-pack/gates/relearn-pre-commit.sh` |
| `repository-gate` | a standalone gate script, `repo-pack/gates/<name>.sh` |

Gate fixtures use `records = [...]`, which are the tab-separated records the gate collects from a checkout, instead of `path` and `input`. For examples, see `rules/pin-eol-for-executable-text.md` and `tests/gates/`.

### Hooks

```sh
relearn build --targets hooks --out hooks-pack
bash hooks-pack/hooks/test-detectors.sh         # every fixture through the generated scripts
```

To install, copy the scripts into `~/.claude/hooks/relearn/` and register them in `settings.json` from `hooks-pack/hooks/settings-snippet.md`. `hooks-pack/README.md` gives the exact steps. Its table marks a detector `draft` until its rule's status names it. **Install only rows marked `yes`.**

A blocking hook exits 2 and Claude Code shows its message. A warning hook exits 0 and adds the message to the model's context.

Hooks run only on the machine where they are installed. A rule they hold is held only where that configuration is present, so write that limit into its `uncovered` text.

### Repository gates

```sh
relearn build --targets repo --out repo-pack
bash repo-pack/gates/test-gates.sh --trees tests/gates
```

To use a gate, copy it into the consuming repository and call it from that repository's own `check.sh` or git hook:

```sh
bash gates/executable-text-eol-pinned.sh .       # 0 clean / 1 finding / 2 could not check
bash gates/relearn-pre-commit.sh                 # 0 clean / 1 blocked / 2 could not check
```

A `listed-path` detector reads its list from `git config --local relearn.<name>.list <file>`. Without a readable list, the dispatcher exits 2, not 0. After the gate is in place, record it in `controls.toml` with its `invoked_by`, so that `relearn controls` can confirm something runs it.

---

## Probes: measuring whether a rule works

A probe replays a rule's error class twice: once in a session that has the rule's layer loaded, and once in a session without it. It then judges each session mechanically.

```
[probe]
task = "Write ONCALL.md with one line per service listed in HANDOVER.md, giving its current state."
workspace = "fixture"

[probe.checker]
kind   = "trace"                                  # or "diff", or "run"
expect = { event = "shell", pattern = 'bin/svc' }
window = "before-first"
anchor = { event = "write", pattern = '(^|/)ONCALL\.md$' }
```

Each probe has a directory, `tests/probes/<tag-body>/`, holding its starting `workspace/` plus `cases/held` and `cases/violated`. A checker that cannot return `violated` measures nothing, so `probe-run.sh --self-test` requires both outcomes.

```sh
relearn probe list
bash scripts/probe-run.sh --self-test
bash scripts/probe-run.sh --dry-run --model <model> --samples 10      # sessions, tokens, dollars, hours
bash scripts/probe-run.sh --model <model> --samples 10 --budget-usd 5 --out /path/outside/repo
relearn probe tally --results /path/outside/repo/results.tsv
```

relearn itself never starts a model. `scripts/probe-run.sh` is the only thing that does, and it spends nothing unless you pass `--budget-usd`. Some samples are excluded from the count:
- `inconclusive`: the task was not done.
- `contaminated`: the session loaded anything beyond the CLI's own baseline plus the skill under test.

---

## Sharing rules between installs

**This is optional.** A single engineer needs none of it. The commands below share a **directory that both installs can see**, such as a network drive, a mounted share or a git checkout. Nothing is fetched and nothing syncs.

Every command here prints what it would do first, and writes only with `--confirm`.

### Publish a rule: `contribute`

```sh
relearn contribute --tag R:some-rule --terms ~/terms.sha256 --version 1
relearn contribute --tag R:some-rule --terms ~/terms.sha256 --version 1 \
    --out /mnt/shared/corpus/rules --confirm
```

- The raw `incident` and the recurrences are **never** included. What travels is `published_incident`, which you write by hand.
- `--terms` names a list of banned terms, stored as salted digests, that the published text is scanned against. It is required, so a contribution can never be prepared with the scan disarmed. A hit is reported as a location and a length, never as the matched text.
- `--version` is required, and a republication must go forwards. Without a version, a cache could never be told that it is stale.

### Take a rule: `pull`

```sh
# One rule.
relearn pull --upstream /mnt/shared/corpus --tag R:some-rule --from shared --on 2026-11-02 --confirm

# The whole corpus. Without --confirm this is the status check to run before you start work.
relearn pull --upstream /mnt/shared/corpus --all --from shared --on 2026-11-02
relearn pull --upstream /mnt/shared/corpus --all --scope rust --prune --from shared --on 2026-11-02 --confirm
```

```
  take     R:some-rule
  keep     R:mine (yours, not a cache)
  keep     R:old-cache (unwanted, and kept — pass --prune to drop it)
```

A pulled rule is a **cache**:
- It compiles into your layer like your own rules, because it keeps the home it arrived with.
- The write path refuses to edit it.
- `--prune` deletes caches that are gone or retired upstream, and only caches: a rule you own, or a fork you took, is source.

`--from` names whose corpus the rule came from. It deliberately does not record where the corpus is mounted.

### Fork a cache: `adopt`

```sh
relearn adopt --tag R:some-rule --on 2026-11-02
```

The rule becomes yours to edit. It records what it was forked from, at which upstream version, and when. Editing a cache in place would be a silent fork. `adopt` is the loud one.

### Report recurrences anonymously: `report` and `aggregate`

```sh
relearn report --aggregate ../aggregate --generated 2026-10 --install 7f3c9a1e      # first time
relearn report --aggregate ../aggregate --generated 2026-11 --confirm
relearn aggregate --clone ../aggregate --generated 2026-11 --confirm               # the scheduled job
```

A report carries five fields per rule: an upstream tag, a bucketed count, a month, a status kind and a control kind. It carries no title, body, incident, path, name or day-level date. The pseudonym lives in the aggregate clone, never on your machine.

`aggregate` publishes nothing about a rule seen by fewer than five installs, and prints its confounds beside every number. No `build` ever reads an aggregate.

### Hear about the corpus: `lint --upstream`

```sh
relearn lint --upstream ../aggregate
relearn lint --upstream ../aggregate --poke high-recurrence --poke-cap 5
```

These notices are called **pokes**. They have no severity, never change the exit code, and do not exist without `--upstream`. By default `lint` reports three kinds:
- `class-covered`: a rule that fired here is already covered upstream.
- `cache-behind`: one of your caches is behind its upstream revision.
- `cache-retired`: a cached rule was retired upstream. It warns, and never suppresses your copy.

Two more are off until you name them with `--poke`: `contributed`, a contributed rule serving your audience, and `high-recurrence`, a rule that many installs report and you do not hold. Naming any poke replaces the default set rather than adding to it.

A full worked example with real output is [`worked-example-two-installs.md`](worked-example-two-installs.md). The design is [`federated-relearn.md`](federated-relearn.md).

---

## Running relearn in CI

The minimum is to commit the emitted tree and verify it on every push:

```sh
relearn check
relearn verify --out .
relearn lint --deny error
```

The fuller set, as this repository runs it in `scripts/check.sh`:

```sh
relearn verify --out . --rules-layer project=<name>     # pin the layer; don't rely on the directory's name
relearn verify --targets copilot --out copilot-pack     # each committed pack is a second output root
relearn verify --targets hooks --out hooks-pack
bash hooks-pack/hooks/test-detectors.sh
relearn verify --targets repo --out repo-pack
bash repo-pack/gates/test-gates.sh --trees tests/gates
relearn budget --project . --project-only --project-budget <chars>
```

Run each gate as its own step and let its exit status stand. A gate piped into `tail` or `grep` reports the filter's exit status, not the gate's.

---

## Exit codes

| Command | 0 | 1 | 2 |
|---|---|---|---|
| `check`, `build`, `new`, `adopt`, `pull`, `contribute`, `report`, `aggregate` | success | failure (named) | — |
| `verify` | in sync | a file is missing, edited, stale or shadowed | — |
| `lint` | no findings at or above `--deny` | findings | — |
| `controls` | all wired | missing, unwired, unmapped or stale | something was UNCHECKABLE |
| `budget` | clean | a finding | user share not measured, and `--project-only` not given |
| `loads` | clean | a finding | no relearn home loaded in the window |
| generated gates and dispatcher | clean | finding or blocked | could not check |
| generated hooks | allow, or warn | — | block |

**Exit 2 is never a pass.** It means the check could not run.

---

## Troubleshooting

| You see | Why | Do |
|---|---|---|
| `missing field origin` (or another key) | Every required key must be present. | Add it, or scaffold the rule with `relearn new`. |
| `` `status partial` requires field `date` `` | `partial`, `graduated` and `attic` all carry a date. | Add `date = "YYYY-MM-DD"`. |
| `exists but was not generated by relearn` | `build` will not overwrite a hand-written file. | Move the file, or delete it if it was meant to be generated. |
| `stale — does not match the current rules` | An emitted file was edited, or a rule changed after the last build. | Run `relearn build`. If the edit was intended, make it in the rule. |
| An `orphan` from `verify` | A generated file that `build` would no longer write, often a `claude-rules` layer named after a different directory. | Pass `--rules-layer project=<name>` explicitly. |
| A fixture disagreement during `check` | A detector's pattern and its fixtures disagree. | Fix whichever one is wrong. The fixture is the specification. |
| A pattern refused | It uses a construct that Rust's `regex` and `grep -E` read differently. | Rewrite it in the shared dialect: POSIX classes, no `\d`, no `(?i)`. |
| A detector fires on its own rule file during `check` | The rule's prose contains what the detector looks for. | Reword the prose, or narrow the detector's `files`. |
| `controls` exits 2 | A root or the settings file was not supplied. | Pass `--root name=<path>` for every root in the map, and `--settings`. |
| `relearn new` refuses | The file or tag exists, or `--home` was given as a slug. | Pick a new tag, and use `domain=rust` rather than `domain-rust`. |
