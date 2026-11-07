# Adopting relearn: the steps and the prompts

This guide takes you through relearn twice: once in a **new project**, once in an **existing project** that already has hand-written assistant instructions. Each step gives you:

- **Do:** what the step is for.
- **Prompt:** the text to paste into your coding assistant (Claude Code, Cursor's agent, Copilot's agent, or anything that can run commands in the repository).
- **Expect:** what a good answer looks like, and what a bad one looks like.
- **Check:** a command you run yourself, so you never have to take the assistant's word for it.

The last section, [What to expect from the assistant](#what-to-expect-from-the-assistant), lists the answers you will get that sound right and aren't: flattery, invented history, enforcement claims nothing backs. It matters most on an existing project.

The [manual](manual.md) is the command reference. This guide gives the order to do things in. Every command output below is real, from relearn 0.1.0 on 2026-11-07, run in scratch repositories named `acme-api` (new) and `legacy-app` (existing).

**The prompts are complete on their own.** They spell out every step of the loop, so they work with any assistant and need no extra skill installed. This repository does not ship an error-loop skill.

**Contents**

- [Before you start](#before-you-start)
- [Path 1: a new project](#path-1-a-new-project)
- [Path 2: an existing project](#path-2-an-existing-project)
- [Day to day, on either path](#day-to-day-on-either-path)
- [What to expect from the assistant](#what-to-expect-from-the-assistant)

---

## Before you start

### Install

```sh
cargo install --git https://github.com/george-andrikopoulos/relearn --tag v0.1.0
relearn --version
```

You need a Rust toolchain that supports edition 2024, and `git`. Generated hooks also need `bash` and `jq`. On Windows, run them under Git Bash.

### Decide where the rules live

| Layout | Use it when | Rules live in |
|---|---|---|
| **A: in the project** | You have one project, or each project's rules are its own | `rules/` at the project root, committed with the code |
| **B: one corpus for several projects** | The same global and language rules serve many repositories | a repository of its own; each project receives only its own layer (see the [manual](manual.md#the-claude-rules-layer)) |

This guide uses layout A. Everything in it works in B, except that `--out` points at the project from the corpus directory.

### Build, and the gate

**Build**, after any change to a rule. Pass the targets and the layer name explicitly:

```sh
relearn build --targets cursor,copilot,agents,claude-rules --out . --rules-layer project=acme-api
```

This writes:
- `.claude/rules/project-acme-api.md`, which Claude Code loads in every session
- `.cursor/rules/<tag>.mdc`
- `.github/copilot-instructions.md`
- `AGENTS.md`

**The gate**, before every commit or push. It checks, verifies and lints. It never builds, because a gate that builds and then verifies only proves that building is deterministic, never that the committed files match the rules.

```sh
relearn check
relearn verify --targets cursor,copilot,agents,claude-rules --out . --rules-layer project=acme-api
relearn lint --deny error
```

**Why spell out the targets and the layer.** A bare `relearn build --out .` also writes Claude skills to `skills/` at the project root, and Claude Code does not load skills from there. A bare `verify` also names the project layer after the directory it runs in. In a clone whose directory has another name, the real layer is reported as a stranger:

```
.claude/rules/project-acme-api.md: orphan — a relearn-generated file this build no longer emits here: its rule or home was removed, or (for `.claude/rules/`) it belongs to another rules layer. `build` will not touch it. Delete it, or restore the rule
3 of 7 generated file(s) drifted
```

**Global rules as a Claude skill.** Once you have a `global` rule, build it into the folder Claude Code reads skills from, and verify it the same way:

```sh
relearn build  --targets claude --home global --out .claude    # writes .claude/skills/global/SKILL.md
relearn verify --targets claude --home global --out .claude
```

`--home global` is an error until at least one global rule exists, so add these two lines when the first one does.

---

## Path 1: a new project

### Step 1. Set up relearn and its gate

**Do:** create `rules/` and a gate script, and run the gate once on the empty corpus.

**Prompt:**

> Set up relearn in this repository, and change nothing else.
>
> 1. Create an empty `rules/` directory.
> 2. Create `scripts/relearn-gate.sh`. It runs these three commands in order, each as its own command and not piped into anything. It exits with the status of the first one that fails:
>    `relearn check`
>    `relearn verify --targets cursor,copilot,agents,claude-rules --out . --rules-layer project=<name>`
>    `relearn lint --deny error`
> 3. Run the script. Show me its full output and its exit status.
>
> Do not write any rules yet.

**Expect:** two new files and this output:

```
ok: 0 rule(s) validated; every detector's fixtures pass, and none fires on a rule file or an emitted file
ok: 0 generated file(s) up to date
ok: no lint findings
0 rule(s): 0 recurred, 0 inert (codified and never fired)
```

A bad answer pipes a command into `tail` or `grep`. The script then exits with the filter's status, and a failure reads as a pass. Another bad answer adds a `relearn build` line to "make it pass".

**Check:** `bash scripts/relearn-gate.sh; echo $?` prints `0`.

### Step 2. Make the gate run without anyone remembering

**Do:** call the gate from a git hook, from CI, or from both.

**Prompt:**

> Run `scripts/relearn-gate.sh` on every push. Add it to `.githooks/pre-push` and tell me the one `git config` line I need to run. If this repository has CI, also add it there as its own step. Show me the files you changed, and nothing else.

**Expect:** a `pre-push` hook and the line `git config core.hooksPath .githooks`. That setting lives in each clone, so nothing in the repository can check that a clone has run it. The assistant should say that. If it doesn't, you know it now.

**Check:** run the `git config` line, then `git push --dry-run`. The gate's output appears.

### Step 3. Turn the first correction into a rule

This is the core of relearn, and the step you will repeat. It starts when the assistant gets something wrong.

**Do:** fix the instance, name the general class, check whether a rule already covers it, and only then write a rule.

**Prompt** (fill in the brackets):

> You just [what it did]. That was wrong because [why].
>
> Fix that first. Then turn the correction into a relearn rule:
>
> 1. **Name the error class**: the general failure, not this instance. Give two different future mistakes the rule would prevent. If you cannot, tell me the correction is one-off and stop there.
> 2. **Search `rules/`** for a rule that already covers the class. If one exists, do not write a new rule. Tell me which one, and we will record a recurrence on it instead.
> 3. **Draft the body**: imperative, at most a few sentences, including one sentence on why.
> 4. **Choose the home**: `project=<name>` if it is about this repository, `global` if it applies to any project, or `domain=<language>` if it applies to one language.
> 5. **Show me the exact `relearn new` command** you would run, with `--origin mined` and this incident in one line: "[what happened, in your words]". Do not run it until I say so.

**Expect:** the fix, then a short proposal, then a command like this:

```sh
relearn new --tag R:migrations-are-reversible \
  --title "Every migration ships its down step" \
  --error-class "A migration without a down step cannot be rolled back during an incident" \
  --home project=acme-api --on 2026-11-07 \
  --incident "Release 41: the orders migration had no down step; rollback took a restore." \
  --body 'Write the down migration in the same change as the up migration, and run both against a scratch database before committing.'
```

A good proposal is short. Its class is broader than the incident and narrower than a platitude. *"Be careful with migrations"* is a platitude. *"The orders migration needs a down step"* is the incident again. Approve it, or edit it and approve the edit. Then:

> Run it, then run the build command and `scripts/relearn-gate.sh`. Show me each command's output and exit status.

```
wrote rules\migrations-are-reversible.md — run `relearn check` before you trust it
wrote 4 file(s) under .
ok: 1 rule(s) validated; every detector's fixtures pass, and none fires on a rule file or an emitted file
ok: 4 generated file(s) up to date
ok: no lint findings
1 rule(s): 0 recurred, 0 inert (codified and never fired)
```

**Check:** `relearn list` shows the rule, and `.claude/rules/project-<name>.md` contains it. Commit the rule and the generated files together.

**Rules from standing practice.** Some rules come from practice, not from an incident: *"we always do X"*. Use `--origin codified`, and make the incident say where the practice came from, for example `"Standing practice."`. `lint` counts codified rules that have never fired as **inert**, so a growing library cannot pass for evidence.

### Step 4. Optional: put the global rules where Claude loads them

**Do:** when your first `global` rule exists, add the two skill lines from [Before you start](#before-you-start) to your build and to `scripts/relearn-gate.sh`.

**Prompt:**

> We now have a global rule. Add `relearn verify --targets claude --home global --out .claude` to `scripts/relearn-gate.sh`, as its own command. Then run `relearn build --targets claude --home global --out .claude` and the gate, and show me both outputs.

**Check:** `.claude/skills/global/SKILL.md` exists, and the gate exits 0.

Then continue with [Day to day](#day-to-day-on-either-path).

---

## Path 2: an existing project

An existing project already has instructions: a `CLAUDE.md`, a `.cursorrules`, a Copilot file, an `AGENTS.md`. relearn will not overwrite any of them. When `build` meets a file it did not write, it stops before writing anything:

```
error: ".\\.github/copilot-instructions.md" exists but was not generated by relearn (no generated-by marker); refusing to overwrite unversioned content
```

So adoption is a migration: find the instructions, decide which ones are rules, move those into `rules/`, then remove them from the hand-written files. **Steps 1 and 2 write nothing.** Keep them that way. Read-only steps are where the assistant is most tempted to "improve" things, and where the answers in [What to expect](#what-to-expect-from-the-assistant) come thickest.

### Step 0. Set the ground rules for the session

**Prompt** (paste once, at the start):

> For this session:
> - Don't assess the quality of this codebase, positively or negatively. Report findings only, each with a file and a line.
> - Before you say anything is done, passing or enforced, show the command you ran, its full output and its exit status. Never pipe a check into a filter.
> - Mark each claim as verified (you ran or read something) or unverified (you are inferring).
> - Change only what a step asks for. List anything else you notice as a suggestion, and don't act on it.
> - Before anything that deletes files, spends money or takes more than a few minutes, tell me the options and what each costs, then wait.

### Step 1. Inventory the instructions (read-only)

**Prompt:**

> List every file in this repository that an AI assistant loads as instructions. Include:
> - `CLAUDE.md` at any depth
> - `.claude/rules/` and `.claude/skills/`
> - `AGENTS.md`
> - `.cursorrules` and `.cursor/rules/`
> - `.github/copilot-instructions.md` and `.github/instructions/`
>
> For each file, give its path, its length in lines, and whether a tool generated it (look for a generated-by marker). Change nothing.

**Expect:** a table of paths. It shouldn't come with opinions about the files.

**Check:** `relearn budget --project . --project-only` independently lists everything Claude Code loads at the start of a session, with sizes:

```
SHARE       CHARS  GENERATED                          PATH                                                 LOADS
project   83  -                                  project:CLAUDE.md                                    always
```

The `-` in `GENERATED` means the file is hand-written. The budget covers Claude Code only, so Cursor and Copilot files come from the assistant's list.

### Step 2. Classify each instruction (read-only)

**Prompt:**

> Go through every instruction in those files and classify each one as exactly one of:
> - **RULE**: a practice whose absence causes a failure you can name. Give the error class.
> - **REFERENCE**: a fact, a command, a path or a convention. It belongs in `CLAUDE.md` or the README, not in a rule.
> - **PREFERENCE**: a matter of taste with no failure behind it.
> - **STALE**: the code contradicts it. Cite the file and line that does.
> - **DUPLICATE**: it repeats another instruction. Say which.
>
> Quote each instruction in one line at most. Don't propose changes to the code. Write nothing.

**Expect:** most instructions in a typical `CLAUDE.md` are REFERENCE, not RULE: build commands, directory layout, naming conventions. That's fine, because they stay where they are. A classification with most items marked RULE is the [too many rules](#what-to-expect-from-the-assistant) answer. Every STALE item should cite a line. If one doesn't, the assistant is guessing.

**Check:** spot-check three STALE citations yourself by opening the cited line.

### Step 3. Write the approved rules

**Do:** go through the RULE list and strike what you disagree with. Then:

**Prompt:**

> Create a rule for each of these: [the items you approved].
>
> - Use `relearn new` with `--origin codified` and `--incident "Carried over from <file it came from>"`.
> - Use home `project=<name>` unless I marked an item global or as a language domain.
> - **Don't invent an incident, a date or a past event.** If git history shows a real incident behind a rule, show me the commit hash and one line about it, and I'll decide whether to use it.
>
> Then run `relearn check` and show its output.

**Expect:** one file per rule in `rules/`, and `ok: N rule(s) validated`. Read two or three rule files. The incident should say where the rule came from, and nothing more.

### Step 4. Remove the migrated text and build

**Prompt:**

> In the hand-written instruction files, remove only the instructions now carried by a rule in `rules/`, and keep everything else as it is.
>
> If a file becomes empty, or held nothing but rules, remove it with `git rm` so relearn can generate it. `.github/copilot-instructions.md` and `AGENTS.md` are the usual cases.
>
> Never delete or regenerate `CLAUDE.md`. relearn does not write it; Claude Code loads the generated layer from `.claude/rules/` next to it.
>
> Show me the diff of every hand-written file before you build. Then run the build command and `scripts/relearn-gate.sh` (create it as in Path 1, step 1, if it doesn't exist yet), and show me both outputs.

**Expect:**

```
wrote 4 file(s) under .
ok: 1 rule(s) validated; every detector's fixtures pass, and none fires on a rule file or an emitted file
ok: 4 generated file(s) up to date
ok: no lint findings
1 rule(s): 0 recurred, 1 inert (codified and never fired)
```

The rule is **inert**: it was carried over from practice and has never fired. That is accurate, not a problem.

**Check:** search the hand-written files for a key phrase from each migrated rule. relearn does not detect a rule that still sits in `CLAUDE.md` as well. In the `legacy-app` run, `budget` passed with the same instruction loaded twice, once from `CLAUDE.md` and once from the generated layer. Only removing the line fixes that.

If you are not ready to replace a hand-written file, leave it in place and leave its target out of `--targets` until you are:

```sh
relearn build --targets cursor,agents,claude-rules --out . --rules-layer project=legacy-app
```

### Step 5. Credit the controls that already exist

An existing project often enforces some of its rules already, through a test, a linter setting or a CI step. Record that, and record what those controls don't cover.

**Prompt:**

> For each rule in `rules/`, is there a test, lint setting, CI step or hook in this repository that already enforces part of it? For each one you find, give:
> - the file
> - the exact command that runs it, and where that command is invoked (CI, a hook, or nowhere)
> - which part of the rule's error class it does **not** cover
>
> Show it working: make a throwaway change that breaks the rule, run the control, show it failing, then revert the change.
>
> Propose `status = { kind = "partial", by = "...", uncovered = "...", date = "..." }` for each rule a control partly holds. Don't propose `graduated` unless the control covers the whole class, and say why it does.

**Expect:** few controls, each one demonstrated failing. "The test suite covers this" with no demonstration is the [enforcement overclaim](#what-to-expect-from-the-assistant). A control nothing invokes is not a control. Record it as found, not wired.

**Check:** to have relearn resolve each named control and confirm something runs it, see [`controls`](manual.md#controls-is-each-named-control-real-and-running) in the manual.

### Step 6. Wire the gate

As in [Path 1, step 2](#step-2-make-the-gate-run-without-anyone-remembering).

### Step 7. Optional: look for real incidents in the history (read-only)

**Prompt:**

> Look through the last 200 commits for fixes that repeat the same kind of mistake at least twice. For each pattern, give the commit hashes and one line each, quoting the commit subject. Don't summarise the history in general, and write nothing.

**Expect:** a short list, possibly empty. An empty list is a real answer. A long, confident list of "patterns" is not evidence until you've looked.

**Check:** `git show --stat <hash>` for each hash it cites. A hash that doesn't exist, or a commit that doesn't do what was claimed, means the pattern was invented. A real one becomes a `mined` rule through [Path 1, step 3](#step-3-turn-the-first-correction-into-a-rule), with the commit as its incident.

Then continue with [Day to day](#day-to-day-on-either-path).

---

## Day to day, on either path

### When a mistake happens that a rule already covers: record a recurrence

The rule failed. Record that on the rule. Don't write a second rule beside it, and don't edit the original incident.

**Prompt:**

> That's the error class `R:<tag>` covers, so the rule failed. Fix the instance. Then add a `[[recurrence]]` table to `rules/<file>.md`, with today's date and a one-line incident: "[what happened]". Don't change the rule's original `incident` or its body. Run the build command and the gate, and show both outputs.

**Expect:** the front matter gains a table, and `lint` now warns:

```
+++
tag = "R:no-customer-email-in-logs"
title = "Never log a customer email address"
error_class = "Personal data written to logs leaves the system through log shipping"
home = { kind = "project", path = "legacy-app" }
created = "2026-11-07"
origin = "codified"
status = { kind = "active" }
incident = "Carried over from the hand-written CLAUDE.md and Copilot instructions."

[[recurrence]]
date = "2026-11-20"
incident = "The payments handler logged the request body, email included."
+++

Never write a customer email address to a log. Log the customer id instead.
```

```
warning: unheld recurrence: R:no-customer-email-in-logs is `active`, so prose is the only thing holding it — and it has fired 1 more time(s) since it was written, most recently 2026-11-20. Promote it to a control that can hold it (a type, a property test, a gate check). If the control covers part of the class, record `status = { kind = "partial", by = "...", uncovered = "...", date = "..." }` and name in `uncovered` what it does not catch; record `graduated` only if it covers the whole class
1 rule(s): 1 recurred, 0 inert (codified and never fired)
1 finding(s), 1 fatal at --deny warning
```

Your gate runs `lint --deny error`, so this warning does not block a push. It is a prompt to do the next step.

### When a rule keeps recurring: push it towards a control

Firmer wording is not the fix. A rule that recurs while only prose holds it needs a check.

**Prompt:**

> `R:<tag>` has recurred while only prose holds it. Propose the strongest control that could hold its error class, in this order of preference:
> 1. a type that makes the mistake impossible to write
> 2. a test
> 3. a gate in CI or a git hook
> 4. a relearn `[[detector]]` with a `hit` and a `clean` fixture
>
> For your proposal, say exactly which part of the error class it would **not** catch. Don't implement it yet.

After you approve it:

> Implement it, then show it catching a deliberate violation. Set the rule's status to `partial`, with `by` naming the control and `uncovered` naming the part it misses. Run the build and the gate.

**Prefer `partial` to `graduated`,** as `lint`'s message says. `graduated` claims the control holds the *whole* class, and that claim is printed into every assistant's layer. The next recurrence outside the control then makes the record wrong. A `partial` status must name its gap, or it does not parse. The [manual](manual.md#detectors) covers detector syntax and turning detectors into hooks and gates.

### When a rule is no longer needed: retire it

**Prompt:**

> Retire `R:<tag>`: set `status = { kind = "attic", reason = "[why]", date = "[today]" }`. Don't delete the file. Run the build and the gate, and show me any `lint` findings about other rules that still refer to it.

An atticked rule stops being emitted but stays in `rules/`, so `lint` can flag anything that still cites it.

### Once a month: read the corpus (read-only)

**Prompt:**

> Run `relearn list --format tsv` and `relearn lint`, and show both outputs whole. Then list:
> - each rule that has recurred, with its count and status
> - each rule that is `active` and has recurred
> - the inert count
>
> Don't edit anything, and don't recommend deleting a rule because it has never fired. A rule that never fires may be working, or may be loaded nowhere, and this data can't tell those apart.

### To measure whether a rule changes what the model does

A probe replays the rule's error class in fresh sessions, with the rule loaded and without it. Probes cost money. `scripts/probe-run.sh --dry-run` prints the cost and starts nothing, and a real run spends nothing without `--budget-usd`. See [Probes](manual.md#probes-measuring-whether-a-rule-works).

**Prompt:**

> Run `bash scripts/probe-run.sh --dry-run --model <model> --samples 10` and show me the output. Don't start a real run.

---

## What to expect from the assistant

An assistant's answer about your repository arrives in one confident register. Whether it is right or wrong, the tone is the same. The answers below are the common ones that sound right and aren't. They come thickest on an existing project, where there is a lot to summarise and little that has been checked.

None of them is rare. In this repository's own history, an assistant's report recommended tagging a release that was already tagged. A reply to that report then stated that no with-and-without measurement existed, when one had been run a week earlier. Both read as confident, and a one-line check corrected each.

| You will see | What it means | Say back |
|---|---|---|
| **Praise first.** *"This is a well-structured, thoughtfully designed codebase."* | No information. It often comes before a proposal to change a large part of the codebase it has just praised. | "Skip the assessment. Findings only, each with a file and line." |
| **Agreement with your framing.** You say *"our tests are solid"*, and it agrees. | Your sentence became its evidence. | Ask without the conclusion: "Which rules does the test suite enforce? Show a test failing on a violation." |
| **Too many rules.** Thirty rules extracted from a 200-line `CLAUDE.md`. | Commands, paths and preferences reclassified as rules. A library full of non-rules stops being read. | "For each one, name the error class and two different future mistakes it prevents. Drop any where you can't." |
| **Invented history.** *"After a past incident where logs leaked data, the team…"*, with `--origin mined`. | A plausible incident nobody can find. Provenance is the one field that must be true. | "Use `codified` and 'Carried over from <file>'. Give me a commit hash, or no incident." |
| **An enforcement claim.** *"This is already enforced by the test suite"*, or a proposal to mark a rule `graduated`. | Usually a test exists near the subject. Whether it fails on a violation, and whether anything runs it, is unchecked. | "Break the rule on purpose, run the control and show it failing. Then name what it doesn't cover." |
| **"Done" or "all tests pass" with no output**, or output that went through `tail` or `grep`. | The command ran. Whether it passed is unknown, because a pipeline reports the last command's exit status. | "Show the command, its full output and its exit status, unfiltered." |
| **A count from a subset.** *"All 42 tests pass"* after `cargo test some_filter`. | True of 42 tests, presented as true of all of them. | "What exact command produced that number, and what does it leave out?" |
| **A document repeated as fact.** *"The README says CI runs verify, so it does."* | A claim inherited without being checked. Documents go stale. | "Show me the line in the CI config that runs it." |
| **Scope creep.** While migrating instructions, it also reformats files, renames modules, bumps dependencies and rewrites the README. | Requirements nobody gave it. Each one is harmless alone; together they bury the change you asked for. | "Revert everything outside `rules/`, the generated files and the lines I approved. List the rest as suggestions." |
| **Deleting to get past a refusal.** `build` refuses to overwrite a hand-written file, so it removes the file. | The refusal was the safety check. The content may not have been migrated. | "Show me where each removed line now lives before anything is deleted." |
| **A new rule beside an old one.** The mistake recurs and it writes `R:x-v2`. | The old rule failed, and that failure now goes unrecorded. | "Search `rules/` first. If a rule covers this, record a recurrence on it." |
| **Apology, then over-correction.** *"You're absolutely right, I apologise"*, followed by a rule like *"always double-check everything"*. | Contrition in place of analysis, and a platitude in place of a rule. | "No apology needed. Fix it, then name the class narrowly enough that a check could test it." |
| **Silent spend.** It starts a paid probe run, fans out to many agents, or runs a 40-minute suite three times. | It decided for you what the task was worth. | "Before anything that costs money or more than a few minutes, give me the options and their cost, then wait." |
| **Hedging everything.** *"It might be worth considering whether perhaps…"* | Over-correction in the other direction: nothing is claimed, so nothing can be checked. | "Recommend one option. Say what you verified and what you didn't." |

**What a good answer looks like:**
- It names files and lines.
- It shows the commands it ran and their exit statuses.
- It separates what it checked from what it inferred.
- It is often smaller than you expected.
- It asks before anything public, paid or irreversible.

When an answer has none of these, run the **Check** line for that step before you act on it.
