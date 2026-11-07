# Adopting relearn: the steps and the prompts

This guide covers two situations:
- **A new project**, set up with relearn from the first commit.
- **An existing project** that has code but none of the standing documents. The guide writes those documents from evidence, then shows how to move the project forward with them.

Each step gives you:

- **Do:** what the step is for.
- **Prompt:** the text to paste into your coding assistant (Claude Code, Cursor's agent, Copilot's agent, or anything that can run commands in the repository).
- **Expect:** what a good answer looks like, and what a bad one looks like.
- **Check:** a command you run yourself, so you never have to take the assistant's word for it.

The last section, [What to expect from the assistant](#what-to-expect-from-the-assistant), lists the answers you will get that sound right and aren't: flattery, invented history, enforcement claims nothing backs. It matters most on an existing project.

The [manual](manual.md) is the command reference. This guide gives the order to do things in. Every command output below is real, from relearn 0.1.0 on 2026-11-07, run in scratch repositories named `acme-api`, `legacy-app` and `shop-svc`.

**The prompts are complete on their own.** They work with any assistant and need no extra skill installed. This repository does not ship an error-loop skill.

**Contents**

- [Before you start](#before-you-start)
- [Path 1: a new project](#path-1-a-new-project)
- [Path 2: an existing project](#path-2-an-existing-project)
- [Moving the project forward](#moving-the-project-forward)
- [Day to day with relearn](#day-to-day-with-relearn)
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

Then continue with [Day to day with relearn](#day-to-day-with-relearn).

---

## Path 2: an existing project

This path is for a project that has code but none of the standing documents: no `CLAUDE.md`, no `ARCHITECTURE.md`, no `FEATURES.md`, no `TODO.md`, and perhaps no real `README.md`. It is about the project, not about relearn. Steps 1 to 7 write the five documents from evidence. [Moving the project forward](#moving-the-project-forward) is how you then make every change with them. relearn comes in only when an assistant's mistake is worth turning into a rule.

| File | Its one job | Written for |
|---|---|---|
| `README.md` | what the project is, and how to build, run and test it | outsiders |
| `ARCHITECTURE.md` | modules, data flow, the perimeter, and an append-only decisions log | the maintainer |
| `FEATURES.md` | the behaviour contract: every feature, each naming the test or type that enforces it | the maintainer and the assistant |
| `TODO.md` | open work, including every gap `FEATURES.md` admits | the maintainer and the assistant |
| `CLAUDE.md` | the recreation standard: from this file alone, the project could be rebuilt | the assistant |

Write them in that order. Each draws on the ones before it, and `CLAUDE.md` summarises the rest, so it comes last.

Two principles hold throughout:
- **Nothing is written from guesswork.** A command goes in only after it has run, and a reason only if someone knows it.
- **A gap is written as a gap.** "Reason not recorded" and "NOTHING YET — exposed" are correct entries. A confident line nobody checked is the failure this path exists to prevent.

### Step 0. A branch and the ground rules

```sh
git checkout -b docs/standing-files
```

**Prompt** (paste once, at the start of the session):

> For this session:
> - Don't assess the quality of this codebase, positively or negatively. Report findings only, each with a file and a line.
> - Before you say anything is done, passing or enforced, show the command you ran, its full output and its exit status. Never pipe a check into a filter.
> - Mark each claim as verified (you ran or read something) or unverified (you are inferring).
> - Don't write a reason, a piece of history or a feature you can't point to evidence for. Write "not recorded" and ask me instead.
> - Change only what a step asks for. List anything else you notice as a suggestion, and don't act on it.
> - Before anything that deletes files, spends money or takes more than a few minutes, tell me the options and what each costs, then wait.

Writing documents from scratch is where an assistant most readily fills a gap with plausible text. These rules are what stop it.

### Step 1. Survey the project (read-only)

**Prompt:**

> Survey this repository and write nothing. Report:
> 1. What it is, in two sentences, judged from the code, not from guesses.
> 2. The build, test and run commands. **Run each one** and show its exit status. A command you didn't run doesn't count.
> 3. The top-level modules or packages and what each one does, citing a file for each.
> 4. The tests that exist, the command that runs them, and whether CI runs them. Cite the CI file and line.
> 5. Anything you can't determine. List those as numbered questions for me.

**Expect:** a short report and a list of questions. Answer the ones you can now. Leave the rest for later; the documents will carry them.

**Check:** run the reported commands yourself. A command that fails now must not go into `README.md` or `CLAUDE.md` as a working one.

### Step 2. README.md, for outsiders

**Prompt:**

> Write `README.md`: what the project is, how to build or install it, and how to run it and its tests. Use only commands from step 1 that exited 0. Write it for someone who has never seen this repository. No badges, no roadmap, and no claims about quality or performance.

**Expect:** short. A README for a project with no documents usually fits on one screen.

**Check:** follow it literally in a fresh clone. Every command must work as written.

### Step 3. ARCHITECTURE.md: the structure, and an honest decisions log

**Prompt:**

> Write `ARCHITECTURE.md` with two parts.
>
> **Structure:** the modules, how data flows between them, and the perimeter, meaning where input enters and where it is parsed or checked. Cite a file for every claim.
>
> **Decisions log:** a table with the columns *Date, Decision, Why, Alternatives rejected*. For each decision visible in the code or the git history, give the commit that introduced it. **Don't invent reasons or rejected alternatives.** Where you don't know, write "not recorded" and add a numbered question for me. Mark every row *reconstructed 2026-11-07*.

**Expect:** rows like this one, with most *Why* cells empty until you answer:

| Date | Decision | Why | Alternatives rejected |
|---|---|---|---|
| reconstructed 2026-11-07 (commit `4e1c09a`) | Orders live in Postgres, not in the shared Redis | not recorded (question 3) | not recorded |

That is correct. A decision written down long afterwards always has a flattering rejected-alternatives column, so leave the cell empty rather than let it be filled.

**Check:** open three of the cited files and confirm each says what the document claims.

### Step 4. FEATURES.md: the behaviour contract

**Prompt:**

> Write `FEATURES.md`. Give it one level-3 heading per user-visible feature or guarantee the project provides **today**, not planned ones. Under each heading, add one sentence on the behaviour, and an **Enforced by:** line naming the test or type that enforces it and the command that runs it. Run that command and confirm it passes. If nothing enforces the feature, write `Enforced by: NOTHING YET — exposed`. Don't name a test that doesn't actually exercise the feature.

**Expect:** entries in this shape:

```markdown
### Orders can be cancelled until they ship

A customer can cancel an order that is pending or paid; cancelling a shipped order returns 409.

Enforced by: `tests/orders.rs::cancel_after_ship_is_refused`, run by `cargo test` (CI: `.github/workflows/ci.yml`).

### Prices are shown in the customer's currency

Enforced by: NOTHING YET — exposed (TODO.md).
```

In a project that never had this file, most entries will be `NOTHING YET — exposed`. That is the file doing its job. A ledger where every entry has a test is either a very well-tested project or an answer to check.

**Check:** for two or three named tests, break the feature on purpose, run the test, watch it fail, then revert your change.

### Step 5. TODO.md: open work

**Prompt:**

> Write `TODO.md`. Give every `NOTHING YET` entry in `FEATURES.md` one `[ ]` line, and every open question from steps 1 and 3 another. Use the markers `[ ]` open, `[~]` in progress, `[x]` done and `[!]` blocked. Don't add new feature ideas.

**Expect:** a list that is mostly gaps and questions. Wishes don't belong in it.

### Step 6. CLAUDE.md, last

**Prompt:**

> Write `CLAUDE.md` so that the project could be rebuilt from it alone. Include:
> - the purpose
> - the core design decisions with their reasons, from the decisions log in `ARCHITECTURE.md`, keeping "not recorded" where the log says so
> - the invariants that must never break, each saying whether `FEATURES.md` enforces it or marks it NOTHING YET
> - the verified commands
> - one line pointing at each of the other four files
> - this definition of done: the change and its enforcing test or type ship together; a regression pass against `FEATURES.md`; `FEATURES.md` updated; `CLAUDE.md` and `ARCHITECTURE.md` checked and updated; `TODO.md` updated
>
> Don't put coding rules in it. If a rule would apply to this project's code in general, list it for me separately.

**Check:** the recreation test. Open a fresh session, give it `CLAUDE.md` alone, and ask: *"What could you not rebuild from this file?"* Each thing it names goes into `CLAUDE.md`, or onto `TODO.md` as a question.

### Step 7. Commit, and guard the set

Commit the five files together. Then keep a sixth standing document from appearing at the root with relearn's `five-root-documents` gate, copied from `repo-pack/` in a relearn checkout:

```sh
mkdir -p gates
cp <relearn-checkout>/repo-pack/gates/five-root-documents.sh gates/
printf '*.sh text eol=lf\n' >> .gitattributes     # keeps the script runnable on Windows checkouts
bash gates/five-root-documents.sh .
```

With the five documents alone, it prints nothing and exits 0. Here is its output in a scratch repository with a sixth document at the root:

```
[R:five-files-no-more] five-root-documents: WARNING. A standing document at the repository root beyond the five (README, CLAUDE, ARCHITECTURE, FEATURES, TODO). Fold its content into one of the five, or into a domain rule if it generalises, unless it is a convention a host reads by name (a changelog, a contribution guide). AGENTS.md is exempt because relearn generates it.
1 finding(s) in .:
  DESIGN.md
```

It is a warning gate, so it still exits 0. Call it from the project's own pre-push hook or CI, the same way as in [Path 1, step 2](#step-2-make-the-gate-run-without-anyone-remembering).

The pack also contains a check that every `FEATURES.md` entry carries an `Enforced by` line. It is marked `draft` and is not ready to install, so use the monthly review below instead.

## Moving the project forward

From here on, the five documents are how every change is made. Each prompt below works without relearn.

### Every change: the definition of done

**Prompt** (put the change in the first line):

> [The change you want.]
>
> Before you start, read `FEATURES.md` and name the entries this change could touch. Then:
> 1. Make the change, and ship the test or type that enforces it in the same change.
> 2. Run the enforcing test of every entry you named, and show each result.
> 3. Update `FEATURES.md`: add an entry with its enforcing artefact, or revise an existing one.
> 4. Say whether `CLAUDE.md` could still recreate the project, and whether `ARCHITECTURE.md` still describes it. If you made a design decision, append it to the decisions log with its reason and the alternatives you rejected.
> 5. Update `TODO.md`: tick what is done, and add anything you discovered.
>
> End with one line per check saying what happened. If a check doesn't apply, say so; don't report it as passed.

**Expect:** the change, then five short lines. A documentation-only change says that checks 1 and 2 don't apply. Five greens on a change that had no test to write are five claims nobody checked.

**Check:** run the tests it names yourself, unfiltered.

### Fixing a bug

**Prompt:**

> [The bug, and how to reproduce it.]
>
> If a type could make this bug impossible to write, propose that first. Otherwise, write a test that reproduces the bug and show it failing. Then fix the bug and show the test passing. Keep the test: it is the regression pin for this bug. Then go through the definition of done.

**Expect:** the test is shown failing before the fix. A test written after the fix, and only ever seen passing, may not test the bug at all.

### Closing an exposed gap

**Prompt:**

> Take the `NOTHING YET` entry in `FEATURES.md` that matters most: [name it, or "you choose, and say why"]. Propose the strongest artefact that could enforce it: a type that makes the violation impossible to write, a property test, or a unit test. Don't implement it yet.

After you approve it:

> Implement it. Break the feature on purpose and show the artefact failing, then restore the feature. Replace `NOTHING YET — exposed` with `Enforced by:`, naming the artefact and its command, and tick the matching line in `TODO.md`.

**Check:** the entry now names an artefact, and the artefact runs in the command CI runs.

### Making a design decision

**Prompt:**

> Before choosing, list the options with what each one costs, and recommend one. After I choose, append a row to the decisions log in `ARCHITECTURE.md`: today's date, the decision, why, and the alternatives rejected, each with its reason. Don't edit earlier rows.

The log is append-only. When a decision later turns out wrong, a new row records the change and why. Rewriting the old row erases what was known at the time, and that is what the log exists to keep.

### Answering the open questions

**Prompt:**

> Here are answers to questions [numbers]: [answers]. In the decisions log, fill in the "not recorded" cells they answer, adding "(answered [date])". Don't change any other cell. Tick the questions in `TODO.md`.

### Once a month: keep the documents true (read-only)

**Prompt:**

> Check the five documents against the repository, and write nothing.
> - Run every command in `README.md` and `CLAUDE.md`, and report each exit status.
> - For every `Enforced by:` line in `FEATURES.md`, confirm the test or type exists, and run it.
> - List `TODO.md` items that are done but not ticked, or ticked but not done, with evidence.
> - List every claim in the structure section of `ARCHITECTURE.md` that the code no longer matches.

**Expect:** a short list. Documents drift one reasonable edit at a time, and this review catches the drift.

**Check:** rerun two of its commands yourself.

### When the assistant makes a mistake worth keeping

If the same kind of mistake would come back in the next session, writing it into `CLAUDE.md` is the wrong fix. A coding rule there is a copy with no source, and it competes with the recreation standard for space. Make it a rule instead. [Path 1, step 3](#step-3-turn-the-first-correction-into-a-rule) has the prompt, and it needs only a `rules/` directory and the build command. [Day to day with relearn](#day-to-day-with-relearn) covers what happens when the rule fails.

### If the project already has assistant instruction files

`relearn build` will not overwrite a hand-written `AGENTS.md`, `.github/copilot-instructions.md` or Cursor rule. It stops before writing anything, and names the file:

```
error: ".\\.github/copilot-instructions.md" exists but was not generated by relearn (no generated-by marker); refusing to overwrite unversioned content
```

Leave that file's target out of `--targets` until its rules have moved into `rules/`, then remove the file with `git rm` and build. A `CLAUDE.md` is never touched: relearn writes the generated layer to `.claude/rules/` beside it.

---

## Day to day with relearn

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

**Writing the documents**

| You will see | What it means | Say back |
|---|---|---|
| **Praise first.** *"This is a well-structured, thoughtfully designed codebase."* | No information. It often comes before a proposal to change a large part of the codebase it has just praised. | "Skip the assessment. Findings only, each with a file and line." |
| **Brochure prose.** *"A robust, scalable, production-ready service"* in `README.md` or `CLAUDE.md`. | Adjectives where facts should be. It fails the recreation test: nobody can rebuild anything from "robust". | "Replace every adjective with the fact behind it, or delete it." |
| **Invented reasons.** *"Postgres was chosen for its reliability and scalability"*, with no commit or person behind it. | A plausible reason nobody can confirm. Once it's in the decisions log, it reads as what was known at the time. | "Write 'not recorded' and add a question for me." |
| **Aspirations as features.** Planned work written up in `FEATURES.md` as if it shipped. | The contract now promises something that doesn't exist. | "Only what the code does today. Move the rest to `TODO.md`." |
| **A test that doesn't test the feature.** `Enforced by:` names a test with a similar name. | It looks enforced and isn't. This is worse than `NOTHING YET`, because nobody will look. | "Break the feature on purpose and show this test failing." |
| **A command it never ran.** `README.md` lists `make test` because projects like this one usually have it. | The first reader who follows the README hits an error. | "Show the exit status of every command you wrote down." |
| **A sixth document.** It offers a `CONTRIBUTING.md`, `DESIGN.md` or `docs/overview.md` "for completeness". | Another resident file that will drift, and that taxes the five that matter. | "Fold it into one of the five." |
| **Agreement with your framing.** You say *"our tests are solid"*, and it agrees. | Your sentence became its evidence. | Ask without the conclusion: "Which features do the tests enforce? Show one failing on a violation." |

**Making changes**

| You will see | What it means | Say back |
|---|---|---|
| **"Done" or "all tests pass" with no output**, or output that went through `tail` or `grep`. | The command ran. Whether it passed is unknown, because a pipeline reports the last command's exit status. | "Show the command, its full output and its exit status, unfiltered." |
| **Five greens on every change.** The definition of done comes back all passed, including a docs-only change. | Checks reported without being run. | "Which of these didn't apply? Say so, rather than marking it passed." |
| **A count from a subset.** *"All 42 tests pass"* after `cargo test some_filter`. | True of 42 tests, presented as true of all of them. | "What exact command produced that number, and what does it leave out?" |
| **A document repeated as fact.** *"The README says CI runs the tests, so it does."* | A claim inherited without being checked. Documents go stale. | "Show me the line in the CI config that runs it." |
| **Scope creep.** Asked to fix one bug, it also reformats files, renames modules, bumps dependencies and rewrites the README. | Requirements nobody gave it. Each one is harmless alone; together they bury the change you asked for. | "Revert everything outside the bug fix. List the rest as suggestions." |
| **A rewritten decision.** It edits an old row in the decisions log to match the new design. | The record of what was known at the time is gone. | "Restore the old row, and append a new one." |
| **Deleting to get past an error.** A check or a refusal blocks it, so it removes the file or the check. | The error was the safety net. | "Put it back. Show me why it fails, and what fixing the cause would take." |
| **Apology, then over-correction.** *"You're absolutely right, I apologise"*, followed by a promise like *"I'll always double-check everything"*. | Contrition in place of analysis, and a platitude in place of a fix. | "No apology needed. Fix it, then tell me what would catch this next time." |
| **Silent spend.** It fans out to many agents, or runs a 40-minute suite three times. | It decided for you what the task was worth. | "Before anything that costs money or more than a few minutes, give me the options and their cost, then wait." |
| **Hedging everything.** *"It might be worth considering whether perhaps…"* | Over-correction in the other direction: nothing is claimed, so nothing can be checked. | "Recommend one option. Say what you verified and what you didn't." |

**Turning corrections into rules** (relearn only)

| You will see | What it means | Say back |
|---|---|---|
| **Too many rules.** Thirty rules extracted from a 200-line `CLAUDE.md`. | Commands, paths and preferences reclassified as rules. A library full of non-rules stops being read. | "For each one, name the error class and two different future mistakes it prevents. Drop any where you can't." |
| **Invented provenance.** *"After a past incident where logs leaked data…"*, with `--origin mined`. | An incident nobody can find. Provenance is the one field that must be true. | "Use `codified` and say where the practice came from. Give me a commit hash, or no incident." |
| **A new rule beside an old one.** The mistake recurs and it writes `R:x-v2`. | The old rule failed, and that failure now goes unrecorded. | "Search `rules/` first. If a rule covers this, record a recurrence on it." |

**What a good answer looks like:**
- It names files and lines.
- It shows the commands it ran and their exit statuses.
- It separates what it checked from what it inferred.
- It writes "not recorded" where it doesn't know.
- It is often smaller than you expected.
- It asks before anything public, paid or irreversible.

When an answer has none of these, run the **Check** line for that step before you act on it.
