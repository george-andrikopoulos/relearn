# Adopting relearn: the steps and the prompts

This guide covers two ways to start:
- **A new project.** You describe what it is and the guidelines it follows, for example *"an HTTP API server in Rust, built with type-driven design"*. The rules that fit are installed, and the project's standing documents are written with those rules in place.
- **An existing project.** Prompts read the code and recover the decisions already made in it. The standing documents are written from what they find.

Both paths end at the same place: five standing documents, the rules that fit the stack, and a `TODO.md` of milestones. From then on, [Working through the milestones](#working-through-the-milestones) is the whole job.

**You describe, approve and check. The assistant writes everything, including all the code.** Each step gives you:

- **Do:** what the step is for.
- **Prompt:** the text to paste into your coding assistant (Claude Code, Cursor's agent, Copilot's agent, or anything that can run commands in the repository).
- **Expect:** what a good answer looks like, and what a bad one looks like.
- **Check:** something you run or read yourself, so you never have to take the assistant's word for it. None of the checks asks you to edit code.

The last section, [What to expect from the assistant](#what-to-expect-from-the-assistant), lists the answers you will get that sound right and aren't: flattery, invented history, a bigger project than you asked for, enforcement claims nothing backs.

The [manual](manual.md) is the command reference. This guide gives the order to do things in. Every command output below is real, from relearn 0.1.0 on 2026-11-07, run in scratch repositories named `orders-api`, `acme-api`, `legacy-app` and `shop-svc`. The prompts work with any assistant and need no extra skill installed.

**Contents**

- [Before you start](#before-you-start)
- [Path 1: a new project](#path-1-a-new-project)
- [Path 2: an existing project](#path-2-an-existing-project)
- [Working through the milestones](#working-through-the-milestones)
- [Day to day with relearn](#day-to-day-with-relearn)
- [What to expect from the assistant](#what-to-expect-from-the-assistant)

---

## Before you start

### Install relearn, and keep a checkout

The rule library ships as committed packs inside the repository, so clone it as well as installing the binary:

```sh
git clone https://github.com/george-andrikopoulos/relearn
cd relearn
cargo install --path . --locked
relearn --version
```

You need a Rust toolchain that supports edition 2024, and `git`. Below, `<relearn-checkout>` means the directory you just cloned.

### The standing documents

Every project gets the same five, and no sixth:

| File | Its one job | Written for |
|---|---|---|
| `README.md` | what the project is, and how to build, run and test it | outsiders |
| `ARCHITECTURE.md` | modules, data flow, the perimeter, and an append-only decisions log | the maintainer |
| `FEATURES.md` | the behaviour contract: every feature, each naming the test or type that enforces it | the maintainer and the assistant |
| `TODO.md` | milestones and open work, including every gap `FEATURES.md` admits | the maintainer and the assistant |
| `CLAUDE.md` | the recreation standard: from this file alone, the project could be rebuilt | the assistant |

Two principles hold for all five:
- **Nothing is written from guesswork.** A command goes in only after it has run, and a reason only if someone knows it.
- **A gap is written as a gap.** "Not recorded" and "NOTHING YET — exposed" are correct entries. A confident line nobody checked is the failure the whole discipline exists to prevent.

### The rule library, and where each layer goes

Rules are never copied into `CLAUDE.md`. Each one has one home, and is installed into the place its assistant loads it from:

| Your project | Install | From `<relearn-checkout>` | Loads |
|---|---|---|---|
| any project | the `global` skill | `claude-pack/skills/global/` into the project's `.claude/skills/` | when a task matches the skill's description |
| Rust | the `domain-rust` rules layer | `rules-pack/.claude/rules/domain-rust.md` into `~/.claude/rules/` | whenever a `.rs` file is read |
| Java | the `domain-java` rules layer | `rules-pack/.claude/rules/domain-java.md` into `~/.claude/rules/` | whenever a `.java` file is read |
| low-latency, public disclosure, agent tooling | the matching skill | `claude-pack/skills/domain-<name>/` into `.claude/skills/` | when a task matches |
| anything the library doesn't cover, such as your API's error format | project rules | written with `relearn new` into the project's `rules/` | in every session in the project |

Three things to know before installing:
- **This library is one author's rules.** Read a layer before you install it. Each `SKILL.md` lists what it covers at the top.
- **A skill loads only when the model chooses to open it.** In relearn's probe pilot, sessions opened the skill in 26 of 70 runs. For a rule that must hold, prefer a rules layer or a project rule.
- **Install a domain layer once, at user level.** A copy inside a project as well makes it load twice. `relearn budget` reports that as `DUPLICATE_LAYER`.

### Project rules: the build, and the gate

**The build** runs after any change to a project rule. Pass the targets and the layer name explicitly:

```sh
relearn build --targets cursor,copilot,agents,claude-rules --out . --rules-layer project=<name>
```

This writes `.claude/rules/project-<name>.md`, which Claude Code loads in every session, plus the same rules for Cursor (`.cursor/rules/`), Copilot (`.github/copilot-instructions.md`) and `AGENTS.md`. A bare `relearn build --out .` would also write skills to a root-level `skills/` folder, which Claude Code never loads.

**The gate** runs before every push. It never builds: a gate that builds and then verifies proves only that building is deterministic, never that the committed files match the rules.

```sh
relearn check
relearn verify --targets cursor,copilot,agents,claude-rules --out . --rules-layer project=<name>
relearn lint --deny error
```

A bare `verify` names the project layer after the directory it runs in, so a clone with another directory name reports the real layer as an `orphan`. Pass `--rules-layer` every time.

---

## Path 1: a new project

### Step 1. Describe the what

**Do:** tell the assistant what the project is, in your words, before anything is created. This description is the input to everything else.

**Prompt** (fill in the brackets):

> I'm starting a new project. Don't create any files yet. Here is what it is:
> - **What it does:** [for example: an HTTP API server for orders: create, list and cancel, JSON over HTTPS]
> - **Stack:** [for example: Rust, axum, PostgreSQL]
> - **Architecture:** [for example: one service; handlers call a domain layer, which calls storage; no database shared with other services]
> - **Guidelines:** [for example: type-driven design, so newtypes for every ID and amount, parse at the boundary, and no unwrap; every API error is an application/problem+json body; every endpoint has an integration test]
> - **Out of scope for now:** [for example: authentication, multi-tenancy, a UI]
>
> Reply with three things: what you understood, in five lines at most; every decision I haven't made that you would have to make, as numbered questions; and anything in my description that contradicts something else in it.

**Expect:** a short summary and a list of questions, such as which PostgreSQL client, how IDs are generated, and what "cancel" means once an order has shipped. Answer them in your next message.

A bad answer is a finished design: a Docker setup, a cache, authentication, a message queue. None of those were in your description. See [a bigger project than you described](#what-to-expect-from-the-assistant).

**Check:** read the five-line summary against what you meant. Anything wrong here will be wrong in every document that follows.

### Step 2. Install the rules that fit

**Do:** match each guideline to a rule that already exists, and write project rules only for what's left.

**Prompt:**

> Match my guidelines against the rule library in `<relearn-checkout>`. Read the files; don't go by their names. Put each guideline into exactly one group:
> - **Covered by a layer:** name the layer (`claude-pack/skills/<name>` or `rules-pack/.claude/rules/<name>.md`) and the rule tags in it that cover the guideline.
> - **A project rule to write:** draft its title, its error class (what goes wrong without it) and a body of at most three sentences.
> - **Not a rule:** an architecture fact, which belongs in `ARCHITECTURE.md` or `CLAUDE.md`.
>
> Then list the exact install commands, and the `relearn new` command for each project rule, using `--origin codified` and `--incident "Stated by the project owner in the project description."`. Check whether anything already exists at each install destination. Run nothing yet.

**Expect:** for *"type-driven design"*, the `domain-rust` layer, citing rules such as `R:design-types-first`, `R:newtype-liberally` and `R:parse-dont-validate`. For the error format, a project rule, because the library has no web-API layer. For *"one service, no shared database"*, an architecture fact. If you approve, say so, and add: *"Run them, then the build and the gate, and show every output."*

```sh
mkdir -p rules .claude/skills
cp -r <relearn-checkout>/claude-pack/skills/global .claude/skills/
cp <relearn-checkout>/rules-pack/.claude/rules/domain-rust.md ~/.claude/rules/
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

The `cp` into `~/.claude/rules/` changes your machine, not just the project, so the assistant should say that before running it.

**Check:** `relearn budget --project . --user ~/.claude` lists what a Claude Code session in the project loads. These two rows show both layers in place (the rest of the output is omitted here):

```
SHARE       CHARS  GENERATED                          PATH                                                 LOADS
user      23843  domain-rust                        user:rules/domain-rust.md                            paths: **/*.rs
project   244  project-orders-api                 project:.claude/rules/project-orders-api.md          always
```

A `DUPLICATE_LAYER` line means the same layer is installed in two places. Remove one copy.

### Step 3. Write the standing documents

**Prompt:**

> Write the five standing documents from my description and my answers to your questions. Don't write any code.
> - **README.md:** what it is, and how it will be built and run. Mark every command "planned" until milestone M0 makes it work.
> - **ARCHITECTURE.md:** the planned modules and data flow, and a decisions log with columns *Date, Decision, Why, Alternatives rejected*. Give every decision in my description one row: today's date, the reason I gave (or "not stated", with a question for me), and the alternatives I named (or "not stated").
> - **FEATURES.md:** no entries yet, only a line saying that an entry is added when a milestone ships a feature, and that every entry names its enforcing test, or says `NOTHING YET — exposed`.
> - **TODO.md:** numbered milestones. Each one delivers something I could see working. M0 sets up the workspace, the test suite, CI, and the gate from this guide. Give each milestone three to six `[ ]` sub-items.
> - **CLAUDE.md:** the purpose, the decisions with their reasons, the invariants, the commands, one line pointing at each of the other four files, and the installed rule layers by name (`global`, `domain-rust`, `project-orders-api`), **without copying any rule text**. End it with this definition of done: the change and its enforcing test or type ship together; a regression pass against FEATURES.md; FEATURES.md updated; CLAUDE.md and ARCHITECTURE.md checked and updated; TODO.md updated.

**Expect:** short documents. A new project's `FEATURES.md` is nearly empty, and its decisions log is your description turned into rows.

**Check:** search `CLAUDE.md` for the title of one installed rule. It should not be there, because the rule loads from its own layer. Then open a fresh session, give it `CLAUDE.md` alone, and ask: *"What could you not rebuild from this file?"*

### Step 4. Approve the milestones

**Prompt:**

> Show me the milestones in TODO.md as a numbered list, with one line each on what I will be able to see working at the end of it.

Reorder them, cut some, or merge them, then tell the assistant what you changed. Commit the five documents, the rules and the generated files together. From here, follow [Working through the milestones](#working-through-the-milestones), starting at M0.

---

## Path 2: an existing project

This path is for a project that has code but none of the standing documents. The prompts in steps 1 to 3 only read. Everything the assistant writes comes from evidence in the code and its history.

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

### Step 1. Survey the project (read-only)

**Prompt:**

> Survey this repository and write nothing. Report:
> 1. What it is, in two sentences, judged from the code, not from guesses.
> 2. The language, framework and storage it uses, with the file that shows each.
> 3. The build, test and run commands. **Run each one** and show its exit status. A command you didn't run doesn't count.
> 4. The top-level modules or packages and what each one does, citing a file for each.
> 5. The tests that exist, the command that runs them, and whether CI runs them. Cite the CI file and line.
> 6. Anything you can't determine. List those as numbered questions for me.

**Check:** run the reported build and test commands yourself. A command that fails now must not go into any document as a working one.

### Step 2. Recover the decisions (read-only)

**Prompt:**

> List the design decisions this code embodies: the framework, persistence, the error-handling style, the module boundaries, the concurrency model, configuration, the API's shape, and anything else that would be expensive to change. For each one, give:
> - the evidence (file and line)
> - the commit that introduced it, found with `git log -S` or `git blame`
> - the reason, **only** if a commit message, a comment or a document states it; otherwise "not recorded"
>
> Write nothing.

**Expect:** a list where many reasons read "not recorded". That is correct: a decision written down long afterwards always comes with a flattering reason, so a gap is better than an invention.

**Check:** run `git show --stat <hash>` for two of the commits it cites. A hash that doesn't exist, or a commit that does something else, means the history was invented.

### Step 3. Find the conventions the code already follows (read-only)

**Prompt:**

> List the conventions this code follows consistently: how errors are typed and returned, how input is validated, how tests are laid out, naming, logging, anything else. For each one, give three examples (file and line) and every counterexample you find. Then match each convention against the rule library in `<relearn-checkout>`: name the layer and the rule tags that cover it, or mark it "a project rule to write". Write nothing.

**Expect:** a short list. A convention with many counterexamples isn't a convention yet: it's a decision to make, so it goes on the question list.

### Step 4. Install the rules that fit

Use the prompt in [Path 1, step 2](#step-2-install-the-rules-that-fit), matching the stack from step 1 and the conventions from step 3 in place of a description. For each project rule, the incident is `"Observed convention: <three files>."`

### Step 5. Write the standing documents

Write them in this order. Each draws on the ones before it, and `CLAUDE.md` comes last because it summarises the rest.

> Write `README.md`: what the project is, how to build or install it, and how to run it and its tests. Use only commands from step 1 that exited 0. No badges, no roadmap, and no claims about quality or performance.

> Write `ARCHITECTURE.md`: the structure from step 1, citing a file for every claim, and a decisions log with columns *Date, Decision, Why, Alternatives rejected*, holding the decisions from step 2. Mark every row *reconstructed* with today's date and its commit. Keep "not recorded" wherever step 2 had no evidence, and add a numbered question for each.

> Write `FEATURES.md`: one level-3 heading per user-visible feature the project provides **today**, each with one sentence on its behaviour and an `Enforced by:` line naming the test that enforces it and the command that runs it. For each named test, prove it: break the feature, show the test failing, restore the feature, and show it passing. If nothing enforces a feature, write `Enforced by: NOTHING YET — exposed`.

> Write `TODO.md` as numbered milestones. M1 closes the most important `NOTHING YET` gaps; later milestones come from the open questions and from anything I tell you I want next. Give each milestone three to six `[ ]` sub-items. Don't add feature ideas of your own.

> Write `CLAUDE.md` so that the project could be rebuilt from it alone: the purpose, the decisions with their reasons (keeping "not recorded"), the invariants, each saying whether FEATURES.md enforces it, the verified commands, one line pointing at each of the other four files, and the installed rule layers by name, without copying rule text. End it with the definition of done from [Path 1, step 3](#step-3-write-the-standing-documents).

**Expect:** most `FEATURES.md` entries will say `NOTHING YET — exposed` in a project that never had the file. That is the file doing its job. A ledger where every entry has a test is either a very well-tested project or an answer to check.

**Check:** confirm that every test named in `FEATURES.md` has a failing run in the transcript. Then run the recreation test: give a fresh session `CLAUDE.md` alone, and ask what it could not rebuild.

### Step 6. Commit, and guard the set

Commit the five documents, the rules and the generated files together. To keep a sixth standing document from appearing at the root, ask the assistant to add relearn's `five-root-documents` gate:

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

It is a warning gate, so it still exits 0. Then approve the milestones exactly as in [Path 1, step 4](#step-4-approve-the-milestones), and continue with [Working through the milestones](#working-through-the-milestones).

**If the project already has assistant instruction files**, such as a hand-written `AGENTS.md` or `.github/copilot-instructions.md`, `relearn build` will not overwrite them. It stops before writing anything, and names the file:

```
error: ".\\.github/copilot-instructions.md" exists but was not generated by relearn (no generated-by marker); refusing to overwrite unversioned content
```

Leave that target out of `--targets` until the file's rules have moved into `rules/`. Then remove the file with `git rm` and build.

---

## Working through the milestones

From here on, the assistant writes all the code. Your part is to approve each milestone's plan, read what the assistant reports, and run the **Check** lines. Every prompt below starts from `TODO.md`.

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
