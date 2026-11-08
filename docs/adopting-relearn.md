# Adopting relearn

**relearn is a way of writing code with the rules already loaded.** Each rule names one bad habit of software development: inventing a reason nobody gave, claiming a test passes without running it, copying a rule into a second place, building more than was asked for. With the rules loaded, those habits are less likely before the work starts. When one happens anyway, it is recorded against the rule that should have prevented it, and a rule that keeps failing is pushed towards a check that can hold it.

**You don't pick the rules, and you don't copy them into a project. The model chooses them.** relearn's library is available to the model on your machine or account. When you describe a project, or point the model at an existing one, the model chooses the rules that apply, names each by its tag, and works under them. The prompts stay short, because the rules already say *how* the work is done. A prompt only says *what* to do.

**Choosing a rule is not loading it.** The model's choice is written into `CLAUDE.md`, which loads in every session, so the choice outlives the session that made it. See [Choosing is not loading](#choosing-is-not-loading).

That gives two ways in:

- **A new project** is the easy case. Nothing has been decided yet, so every decision is made fresh, under the rules the model chose. Four prompts take you from a description to a list of milestones.
- **An existing project** already holds decisions in its code, but no rules and no record of why. Four prompts rediscover it, one standing document each: `ARCHITECTURE.md`, `FEATURES.md`, `TODO.md` and `CLAUDE.md`.

Both end in the same place: the rules chosen, the standing documents written, and a `TODO.md` of milestones. From then on, [Working through the milestones](#working-through-the-milestones) is the whole job.

**You describe, approve and check. The model chooses the rules and writes everything, including all the code.** Each prompt comes with **Expect** (what a good answer looks like, and a bad one) and **Check** (something you run or read yourself, so you never have to take the model's word for it).

The [manual](manual.md) is the command reference. Every command output below is real, from relearn 0.1.0 on 2026-11-07, run in scratch repositories named `orders-api`, `acme-api`, `legacy-app` and `shop-svc`.

**Contents**

- [Once, before any project](#once-before-any-project)
- [A new project: four prompts](#a-new-project-four-prompts)
- [An existing project: four prompts](#an-existing-project-four-prompts)
- [Working through the milestones](#working-through-the-milestones)
- [Day to day with relearn](#day-to-day-with-relearn)
- [What to expect from the assistant](#what-to-expect-from-the-assistant)

---

## Once, before any project

You do this once per machine or account, never per project.

- **The binary:** `cargo install --git https://github.com/george-andrikopoulos/relearn --locked`. It needs a Rust toolchain that supports edition 2024, and `git`.
- **The library, where the model can reach it:** the skills in [`claude-pack/`](../claude-pack/README.md) at user level (`~/.claude/skills/`, or uploaded to claude.ai), and the language layers in [`rules-pack/`](../rules-pack/) in `~/.claude/rules/`. `claude-pack/README.md` has the steps for each.

From then on, every project you open has the whole library within reach, and the model chooses from it. Nothing about rules is copied into a project. A project carries only its **own** rules, the ones the library doesn't have, which the model writes with `relearn new` into the project's `rules/`.

### Choosing is not loading

The library reaches a session in two ways, and only one of them is automatic:

- **A language layer** in `~/.claude/rules/` (from `rules-pack/`) loads by itself whenever the session reads a matching file. `domain-rust` loads for `**/*.rs`.
- **A skill** (from `claude-pack/`) loads only when the model decides to open it. That includes `global` and `domain-project-files`, the two layers every prompt below relies on. In relearn's probe pilot, sessions opened the skill in 26 of 70 runs.

So the model's choice has to be written somewhere that loads every time. That place is `CLAUDE.md`. When the model writes it (prompt 3 of a new project, prompt 4 of an existing one), it records the layers it chose and tells every later session to open the skills among them before starting work. That is a pointer, not a copy: no rule text goes into `CLAUDE.md`, and the rules still load from their own layer. The choice is still the model's. `CLAUDE.md` only carries it into sessions that did not make it.

For a rule that must hold whatever the model decides to open, don't rely on a skill: make it a project rule, which loads in every session, or a check.

### A project's own rules: the build, and the gate

When the model writes a project rule, it compiles it into every assistant's layer. The targets and the layer name are passed explicitly:

```sh
relearn build --targets cursor,copilot,agents,claude-rules --out . --rules-layer project=<name>
```

This writes `.claude/rules/project-<name>.md`, which Claude Code loads in every session, plus the same rules for Cursor (`.cursor/rules/`), Copilot (`.github/copilot-instructions.md`) and `AGENTS.md`.

The gate runs before every push. It never builds: a gate that builds and then verifies proves only that building is deterministic, never that the committed files match the rules.

```sh
relearn check
relearn verify --targets cursor,copilot,agents,claude-rules --out . --rules-layer project=<name>
relearn lint --deny error
```

Pass `--rules-layer` every time. A bare `verify` names the project layer after the directory it runs in, so a clone with another directory name reports the real layer as an `orphan`.

---

## A new project: four prompts

Nothing is captured yet, so nothing has to be recovered. You describe; the model chooses the rules and decides nothing you haven't.

### 1. Describe it

**Prompt** (fill in the brackets):

> I'm starting a new project. Don't create any files yet. Here is what it is:
> - **What it does:** [for example: an HTTP API server for orders: create, list and cancel, JSON over HTTPS]
> - **Stack:** [for example: Rust, axum, PostgreSQL]
> - **Architecture:** [for example: one service; handlers call a domain layer, which calls storage; no database shared with other services]
> - **Guidelines:** [for example: type-driven design; every API error is an application/problem+json body; every endpoint has an integration test]
> - **Out of scope for now:** [for example: authentication, multi-tenancy, a UI]
>
> Reply with what you understood, in five lines at most, then every decision I haven't made that you would have to make, as numbered questions.

**Expect:** a short summary and a list of questions, such as which PostgreSQL client, how IDs are generated, and what "cancel" means once an order has shipped. Answer them in your next message. A bad answer is a finished design with Docker, a cache, authentication and a message queue. None of those were in your description.

**Check:** read the five-line summary against what you meant. Anything wrong here will be wrong in every document that follows.

### 2. The model chooses the rules

**Prompt:**

> Choose the relearn rules that apply to this project, from the library available to you. Name each layer and the rule tags in it that matter here, with one line on why. Then take each of my guidelines in turn: either a chosen rule covers it (name the tag), or it is a project rule the library doesn't have (write it with `relearn new`, `--origin codified` and `--incident "Stated by the project owner in the project description."`), or it is an architecture fact for ARCHITECTURE.md. Show me the choice before writing anything; after I approve, write the project rules, then run the build and the gate and show every output.

**Expect:** layers chosen with reasons, such as `global` for every project, `domain-project-files` for the standing documents, and `domain-rust` for *"type-driven design"* (`R:design-types-first`, `R:newtype-liberally`, `R:parse-dont-validate`). The error format becomes a project rule, because the library has no web-API layer. *"One service, no shared database"* is an architecture fact. After you approve, the model writes the project rule and builds:

```sh
relearn new --tag R:api-errors-are-problem-json \
  --title "Every API error is an application/problem+json body" \
  --error-class "An error body in an ad-hoc shape forces every client to special-case each endpoint" \
  --home project=orders-api --on 2026-11-07 --origin codified \
  --incident "Stated by the project owner in the project description." \
  --body 'Return every 4xx and 5xx response as application/problem+json (RFC 9457), built by one function. No handler builds an error body itself.'
relearn build --targets cursor,copilot,agents,claude-rules --out . --rules-layer project=orders-api
```

```
wrote rules\api-errors-are-problem-json.md — run `relearn check` before you trust it
wrote 4 file(s) under .
ok: 1 rule(s) validated; every detector's fixtures pass, and none fires on a rule file or an emitted file
ok: 4 generated file(s) up to date
ok: no lint findings
1 rule(s): 0 recurred, 1 inert (codified and never fired)
```

A bad answer chooses every layer "to be safe", or picks a layer by its name without reading it. A layer that names another project (`project-*`) is never right for yours.

**Check:** ask *"Which of the rules you chose decides how a decisions-log row is written?"* The answer should be a tag, `R:decisions-log-records-rejected-alternatives`. A paraphrase with no tag means the rule wasn't read. `relearn budget --project . --user ~/.claude` then lists what a session in the project loads:

```
SHARE       CHARS  GENERATED                          PATH                                                 LOADS
user      23843  domain-rust                        user:rules/domain-rust.md                            paths: **/*.rs
project   244  project-orders-api                 project:.claude/rules/project-orders-api.md          always
```

### 3. Write the five documents

**Prompt:**

> Write the five standing documents from my description and my answers, under the rules you chose. Don't write any code. Mark every command in README.md "planned" until milestone M0 makes it work. FEATURES.md has no entries yet. TODO.md is numbered milestones, each one something I could see working, with M0 setting up the workspace, the tests, CI and the gate. CLAUDE.md names the rules you chose, by layer and tag, copies no rule text, and tells every session to open the skills among them before starting work.

**Expect:** short documents. A new project's `FEATURES.md` is nearly empty, and its decisions log is your description turned into rows, with "not stated" wherever you gave no reason.

**Check:** search `CLAUDE.md` for the title of one chosen rule. It should not be there, because the rule loads from its own layer. Then start a fresh session in the project and ask *"Which rule decides how a decisions-log row is written?"* before anything else. The answer should be the tag, and the session should have opened `domain-project-files` to find it. Finally, give a fresh session `CLAUDE.md` alone, and ask: *"What could you not rebuild from this file?"*

### 4. Approve the milestones

**Prompt:**

> Show me the milestones in TODO.md as a numbered list, with one line each on what I will be able to see working at the end of it.

Reorder them, cut some, or merge them, then tell the model what you changed. Commit the documents, the project rules and the generated files together, and go to [Working through the milestones](#working-through-the-milestones), starting at M0.

---

## An existing project: four prompts

The code already holds decisions, but nobody wrote down the rules it follows or why it is the way it is. Work on a branch (`git checkout -b docs/standing-files`) and give these four prompts in order. The first one has the model choose the rules from the code, and every prompt after works under them. Each prompt rediscovers one standing document from the code and its git history. `CLAUDE.md` comes last, because it summarises the other three.

### 1. Choose the rules, and rediscover the architecture: `ARCHITECTURE.md`

**Prompt:**

> Read this project's code and git history. From the relearn library available to you, choose the rules that apply to it, and name each layer and tag with one line on why. Then, under those rules, rediscover the architecture and write `ARCHITECTURE.md`. Run the build and test commands first and show each exit status. Describe the modules and the data flow, citing a file for every claim. Then write the decisions log: every decision the code embodies that would be expensive to change, each with its evidence, its commit (from `git log -S` or `git blame`) and the word *reconstructed*. Fill in **Why** and **Alternatives rejected** only where a commit, a comment or a document states them, and write "not recorded" everywhere else, with a numbered question for me.

**Expect:** a short list of chosen rules with tags, then a document with many rows that say "not recorded". That is the document doing its job. A reason written down long after the decision is always the flattering one, so a gap is better than an invention.

**Check:** run `git show --stat <hash>` for two of the commits it cites. A hash that doesn't exist, or a commit that does something else, means the history was invented.

### 2. Rediscover what it does: `FEATURES.md`

**Prompt:**

> Write `FEATURES.md`: one entry per user-visible feature the project provides **today**, each with one sentence on its behaviour and an `Enforced by:` line naming the test and the command that runs it. Prove each named test: break the feature, show the test failing, restore it, and show it passing. Where nothing enforces a feature, write `Enforced by: NOTHING YET — exposed`.

**Expect:** most entries say `NOTHING YET — exposed` in a project that never had the file. A ledger where every entry has a test is either a very well-tested project or an answer to check.

**Check:** confirm that every test named in `FEATURES.md` has a failing run in the transcript.

### 3. Rediscover what is open: `TODO.md`

**Prompt:**

> Write `TODO.md` as numbered milestones. M1 closes the most important `NOTHING YET — exposed` gaps from FEATURES.md. The later milestones come from the open questions in ARCHITECTURE.md and from what I tell you I want next. Give each milestone three to six `[ ]` items. Add no feature ideas of your own.

**Expect:** a short list built from the two documents you already have. A milestone you didn't ask for, and that no gap or question explains, is the model's idea. Cut it.

**Check:** every M1 item should name a `FEATURES.md` entry.

### 4. Rediscover the charter: `CLAUDE.md`

**Prompt:**

> Write `CLAUDE.md` to the recreation standard, from the three documents and the code: the purpose, the decisions with their reasons (keeping "not recorded"), the invariants, each saying whether FEATURES.md enforces it, the verified commands, one line pointing at each of the other documents, the rules you chose by layer and tag with no rule text copied, an instruction to every session to open the skills among them before starting work, and the definition of done. If the project has no README.md, write one using only commands that exited 0. If it has one, report every command in it that failed, and change nothing else in it.

**Expect:** a charter that is mostly pointers and reasons. Rules are named, never restated.

**Check:** start a fresh session and ask for the tag of the rule that decides how a decisions-log row is written, as in [prompt 3 of the new-project path](#3-write-the-five-documents). Then give a fresh session `CLAUDE.md` alone, and ask what it could not rebuild. Then approve the milestones exactly as in [prompt 4 of the new-project path](#4-approve-the-milestones), commit everything together, and go to [Working through the milestones](#working-through-the-milestones).

### Then: the project's own conventions, as project rules

The library covers what is general. What is particular to this project, such as how its errors are shaped, the model writes as a project rule. This can wait until the four documents are committed.

**Prompt:**

> List the conventions this code follows consistently that none of the rules you chose covers, each with three examples (file and line) and every counterexample you find. Show me the list; after I approve, write each one as a project rule with `relearn new`, `--origin codified` and `--incident "Observed convention: <three files>."`, then run the build and the gate.

A convention with many counterexamples isn't a convention yet. It's a decision to make, so it goes to `TODO.md` as a question.

### If the project already has assistant instruction files

A hand-written `AGENTS.md` or `.github/copilot-instructions.md` is not overwritten. `relearn build` stops before writing anything, and names the file:

```
error: ".\\.github/copilot-instructions.md" exists but was not generated by relearn (no generated-by marker); refusing to overwrite unversioned content
```

Leave that target out of `--targets` until the file's rules have moved: each is either covered by a rule in the library, or becomes a project rule. Then remove the file with `git rm` and build. A hand-written `CLAUDE.md` full of rules is treated the same way, and the charter keeps only what prompt 4 asks for.

### Guarding the set

To keep a sixth standing document from appearing at the root, ask the model to add relearn's `five-root-documents` gate from `repo-pack/gates/`, and run it:

```sh
bash gates/five-root-documents.sh .
```

With the five documents alone, it prints nothing and exits 0. Here is its output in a scratch repository with a sixth document at the root:

```
[R:five-files-no-more] five-root-documents: WARNING. A standing document at the repository root beyond the five (README, CLAUDE, ARCHITECTURE, FEATURES, TODO). Fold its content into one of the five, or into a domain rule if it generalises, unless it is a convention a host reads by name (a changelog, a contribution guide). AGENTS.md is exempt because relearn generates it.
1 finding(s) in .:
  DESIGN.md
```

It is a warning gate, so it still exits 0.

---

## Working through the milestones

From here on, the assistant writes all the code. Your part is to approve each milestone's plan, read what the assistant reports, and run the **Check** lines. Every prompt below works under the rules the model chose, which `CLAUDE.md` carries into each session, and starts from `TODO.md`.

### Start a milestone

**Prompt:**

> Start milestone [Mn]. In TODO.md, expand it into sub-tasks, each small enough to finish and verify in one go. For each sub-task, name the FEATURES.md entry it will add or change, and the test or type that will enforce it. Show me the plan, and don't write code yet.

**Expect:** a short plan. It should name, for each sub-task, the test or type that will prove it. A plan that says "add tests at the end" has moved the proof to a later change, and a later change rarely comes.

### Do each sub-task

**Prompt** (once per sub-task):

> Do the next open sub-task of [Mn]. Before you start, read FEATURES.md and name the entries this could affect. Then:
> 1. Write the code, and ship the test or type that enforces it in the same change. Show the new test failing before the code exists, then passing.
> 2. Run the enforcing test of every entry you named, and show each result.
> 3. Update FEATURES.md: add an entry with its enforcing artefact, or revise an existing one.
> 4. Say whether CLAUDE.md could still recreate the project and whether ARCHITECTURE.md still describes it. If you made a design decision, append it to the decisions log with its reason and the alternatives you rejected.
> 5. Tick the sub-task in TODO.md, and add anything you discovered as new items.
>
> End with one line per check saying what happened. If a check doesn't apply, say so; don't report it as passed.

**Expect:** the change, then five short lines.

**Check:** run the project's test command yourself, unfiltered, and confirm the exit status.

### Close the milestone

**Prompt:**

> Close [Mn]. Run the full test suite and the gate, unfiltered, and show their output and exit status. Confirm that every sub-task is ticked, and that every FEATURES.md entry this milestone added names a test that ran. Check whether CLAUDE.md could still recreate the project, and update it if not. Then name the next milestone, and wait.

**Expect:** a milestone is closed only when all its sub-tasks are ticked and its tests ran. "Mostly done" is a milestone still open.

**Check:** read the `FEATURES.md` entries the milestone added. Each should name a test you saw run.

### When something is wrong

**Prompt:**

> [What you saw, and how to reproduce it.] If a type could make this bug impossible to write, propose that first. Otherwise, write a test that reproduces the bug, show it failing, fix the bug, and show the test passing. Keep the test. Then go through the five checks.

### When a decision comes up

**Prompt:**

> Before choosing, list the options with what each one costs, and recommend one. After I choose, append a row to the decisions log in ARCHITECTURE.md: today's date, the decision, why, and the alternatives rejected, each with its reason. Don't edit earlier rows.

The log is append-only. When a decision later turns out wrong, a new row records the change and the reason. Rewriting the old row erases what was known at the time.

### Once a month: keep the documents true (read-only)

**Prompt:**

> Check the five documents against the repository, and write nothing.
> - Run every command in README.md and CLAUDE.md, and report each exit status.
> - For every `Enforced by:` line in FEATURES.md, confirm the test or type exists, and run it.
> - List TODO.md items that are done but not ticked, or ticked but not done, with evidence.
> - List every claim in the structure section of ARCHITECTURE.md that the code no longer matches.

**Check:** rerun two of its commands yourself.

### When the assistant makes a mistake worth keeping

Writing it into `CLAUDE.md` is the wrong fix: that makes a copy of a rule with no source. Make it a rule instead, as described in the next section.

---

## Day to day with relearn

### When the assistant makes a mistake: write a rule

**Prompt** (fill in the brackets):

> You just [what it did]. That was wrong because [why].
>
> Fix that first. Then turn the correction into a relearn rule:
>
> 1. **Name the error class**: the general failure, not this instance. Give two different future mistakes the rule would prevent. If you cannot, tell me the correction is one-off and stop there.
> 2. **Search `rules/` and the installed layers** for a rule that already covers the class. If one exists, don't write a new rule. Tell me which one, and we will record a recurrence on it instead.
> 3. **Draft the body**: imperative, at most a few sentences, including one sentence on why.
> 4. **Show me the exact `relearn new` command** you would run, with `--home project=<name>`, `--origin mined` and this incident in one line: "[what happened, in your words]". Don't run it until I say so.

**Expect:** the fix, then a short proposal. Its class is broader than the incident and narrower than a platitude. *"Be careful with migrations"* is a platitude. *"The orders migration needs a down step"* is the incident again. After you approve, the assistant runs the command, the build and the gate:

```
wrote rules\migrations-are-reversible.md — run `relearn check` before you trust it
wrote 4 file(s) under .
ok: 1 rule(s) validated; every detector's fixtures pass, and none fires on a rule file or an emitted file
ok: 4 generated file(s) up to date
ok: no lint findings
1 rule(s): 0 recurred, 0 inert (codified and never fired)
```

**Check:** `relearn list` shows the rule, and `.claude/rules/project-<name>.md` contains it.

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

An assistant's answer about your project arrives in one confident register. Whether it is right or wrong, the tone is the same. The answers below are the common ones that sound right and aren't.

None of them is rare. In this repository's own history, an assistant's report recommended tagging a release that was already tagged. A reply to that report then stated that no with-and-without measurement existed, when one had been run a week earlier. Both read as confident, and a one-line check corrected each.

**Describing and planning**

| You will see | What it means | Say back |
|---|---|---|
| **Praise first.** *"Great idea!"*, or *"This is a well-structured, thoughtfully designed codebase."* | No information. It often comes before a proposal to change much of what it has just praised. | "Skip the assessment. Findings only, each with a file and line." |
| **A bigger project than you described.** You asked for an orders API, and the plan has Docker, Redis, authentication and a message queue. | Requirements nobody gave it. Each one has to be built, tested and kept, and none was asked for. | "Only what I described. List the rest as questions, and I'll decide." |
| **Choices made for you.** It picks the database driver, the ID format and the deployment target without asking. | Decisions that belong in your decisions log, made in the assistant's head. | "List every decision you made that I didn't, as questions." |
| **Agreement with your framing.** You say *"our tests are solid"*, and it agrees. | Your sentence became its evidence. | Ask without the conclusion: "Which features do the tests enforce? Show one failing on a violation." |

**Writing the documents**

| You will see | What it means | Say back |
|---|---|---|
| **Brochure prose.** *"A robust, scalable, production-ready service"* in `README.md` or `CLAUDE.md`. | Adjectives where facts should be. Nobody can rebuild anything from "robust". | "Replace every adjective with the fact behind it, or delete it." |
| **Invented reasons.** *"PostgreSQL was chosen for its reliability"*, with no commit, comment or person behind it. | A plausible reason nobody can confirm. In the decisions log, it reads as what was known at the time. | "Write 'not recorded', and add a question for me." |
| **Rule text copied into `CLAUDE.md`.** The type-driven rules restated in the charter. | A second copy with no source. It drifts from the installed layer, and loads twice. | "Name the layer, and delete the copied text." |
| **Rules named but never opened.** `CLAUDE.md` lists `global` and `domain-project-files`, and a later session answers a question they decide without citing a tag. | A skill loads only when the model opens it. Naming it is not opening it. | "Open the skills `CLAUDE.md` names, then answer again with the tag." |
| **Aspirations as features.** Planned work written up in `FEATURES.md` as if it shipped. | The contract now promises something that doesn't exist. | "Only what the code does today. Move the rest to `TODO.md`." |
| **A test that doesn't test the feature.** `Enforced by:` names a test with a similar name. | It looks enforced and isn't. That is worse than `NOTHING YET`, because nobody will look. | "Break the feature, show this test failing, then restore it." |
| **A command it never ran.** `README.md` lists `make test` because projects like this one usually have it. | The first reader who follows the README hits an error. | "Show the exit status of every command you wrote down." |
| **A sixth document.** It offers a `CONTRIBUTING.md`, `DESIGN.md` or `docs/overview.md` "for completeness". | Another resident file that will drift, and that taxes the five that matter. | "Fold it into one of the five." |

**Working through the milestones**

| You will see | What it means | Say back |
|---|---|---|
| **"Done" or "all tests pass" with no output**, or output that went through `tail` or `grep`. | The command ran. Whether it passed is unknown, because a pipeline reports the last command's exit status. | "Show the command, its full output and its exit status, unfiltered." |
| **Five greens on every change.** The checks come back all passed, even on a change with no test to write. | Checks reported without being run. | "Which of these didn't apply? Say so, rather than marking it passed." |
| **A milestone closed with work left.** *"M2 is essentially complete"*, with two sub-tasks unticked. | "Essentially" means not. | "Close it only when every sub-task is ticked and its tests ran." |
| **A count from a subset.** *"All 42 tests pass"* after `cargo test some_filter`. | True of 42 tests, presented as true of all of them. | "What exact command produced that number, and what does it leave out?" |
| **A document repeated as fact.** *"The README says CI runs the tests, so it does."* | A claim inherited without being checked. Documents go stale. | "Show me the line in the CI config that runs it." |
| **Scope creep.** Asked for one sub-task, it also reformats files, renames modules and bumps dependencies. | Each change is harmless alone; together they bury the change you asked for. | "Revert everything outside this sub-task. List the rest as suggestions." |
| **A rewritten decision.** It edits an old row in the decisions log to match the new design. | The record of what was known at the time is gone. | "Restore the old row, and append a new one." |
| **Deleting to get past an error.** A check or a refusal blocks it, so it removes the file or the check. | The error was the safety net. | "Put it back. Show me why it fails, and what fixing the cause would take." |
| **Apology, then over-correction.** *"You're absolutely right, I apologise"*, followed by a promise like *"I'll always double-check everything"*. | Contrition in place of analysis, and a platitude in place of a fix. | "No apology needed. Fix it, then tell me what would catch this next time." |
| **Silent spend.** It fans out to many agents, or runs a 40-minute suite three times. | It decided for you what the task was worth. | "Before anything that costs money or more than a few minutes, give me the options and their cost, then wait." |
| **Hedging everything.** *"It might be worth considering whether perhaps…"* | Over-correction in the other direction: nothing is claimed, so nothing can be checked. | "Recommend one option. Say what you verified and what you didn't." |

**Rules**

| You will see | What it means | Say back |
|---|---|---|
| **Too many rules.** Thirty project rules from a one-page description. | Facts, commands and preferences reclassified as rules. A library full of non-rules stops being read. | "For each one, name the error class and two different future mistakes it prevents. Drop any where you can't." |
| **Invented provenance.** *"After a past incident where logs leaked data…"*, with `--origin mined`. | An incident nobody can find. Provenance is the one field that must be true. | "Use `codified` and say where the practice came from. Give me a commit hash, or no incident." |
| **A new rule beside an old one.** The mistake recurs, and it writes `R:x-v2`. | The old rule failed, and that failure now goes unrecorded. | "Search first. If a rule covers this, record a recurrence on it." |

**What a good answer looks like:**
- It names files and lines.
- It shows the commands it ran and their exit statuses.
- It separates what it checked from what it inferred.
- It writes "not recorded" where it doesn't know.
- It asks questions about decisions you haven't made.
- It is often smaller than you expected.
- It asks before anything public, paid or irreversible.

When an answer has none of these, run the **Check** line for that step before you act on it.
