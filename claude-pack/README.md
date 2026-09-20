# claude-pack — the rule library as Claude Skills

A downloadable copy of this repository's rules, emitted as **Claude Skills**: one skill
per home layer, each a folder containing a `SKILL.md`. Drop it into Claude Code, or
install it on claude.ai, without cloning this repository or building anything.

```
claude-pack/
└── skills/
    ├── global/SKILL.md                          34 rules — every project, every language
    ├── domain-rust/SKILL.md                     18 rules — the Rust type-driven discipline
    ├── domain-low-latency/SKILL.md              15 rules — latency-sensitive systems work
    ├── domain-java/SKILL.md                     13 rules — the Java language discipline
    ├── project-relearn/SKILL.md                  2 rules — the relearn repository
    ├── project-stochos-lab/SKILL.md              2 rules — the stochos-lab repository
    └── project-design-architecture-tool/SKILL.md  3 rules — that repository
```

Those counts are hand-written, and since 2026-09-13 `tests/pack_counts.rs` reads each one
back off the skill it names and fails the build when they disagree — so they are hand-kept
rather than unchecked. The reading that needs no build at all is the file's own:

```bash
grep -c '^## ' claude-pack/skills/global/SKILL.md
```

**Take the layers you want.** `global` is the one most people want on its own; add
`domain-rust` if you write Rust. The three `project-*` skills are included for
completeness and are almost certainly *not* what you want elsewhere — they describe three
specific repositories, and a rule that names a project you are not working in is noise.

## Install

### Claude Code — one project

Copy the skill folders you want into the project's `.claude/skills/`:

```bash
mkdir -p .claude/skills
cp -r claude-pack/skills/global .claude/skills/
cp -r claude-pack/skills/domain-rust .claude/skills/
```

Claude Code discovers `.claude/skills/<name>/SKILL.md` and reads each skill's
`description` to decide when to load it. Commit them like any other project file.

**Check for a name collision first.** `global` and `domain-rust` are generic folder
names, and `cp -r` over an existing skill of the same name destroys it without a word —
a hand-authored skill has no `relearn:generated` footer, so nothing here can recognise
it as somebody's work and refuse. List the directory before you copy, and rename the
incoming folder if it is taken (the folder name is the skill name; nothing inside the
file depends on it).

### Claude Code — every project on the machine

The same folders, under `~/.claude/skills/` instead. These load in any repository, so put
only `global` (and a language layer you always use) here — a project layer installed
machine-wide is the one-home-per-rule violation this tool exists to prevent. The same
collision check applies, and matters more: a machine-wide clobber takes the skill out of
every repository at once.

### claude.ai (web and desktop app)

A skill is uploaded as a folder — or a zip of that folder — with `SKILL.md` at its root,
which is exactly what each directory here is. Anthropic's own documentation is the place
to check where the upload lives in the current interface and which plans have Skills
enabled; it moves, and a menu path transcribed here would go stale silently.

## Letting Claude do the install

Copying two folders by hand is faster than explaining it, in a layout you already know.
Use the prompt when you do not know the layout — an unfamiliar repository, a monorepo
where it is not obvious which directory owns `.claude/`, or an employer whose repository
template puts agent configuration somewhere of its own.

```
You are installing generated skill folders into this environment. They are build
artifacts from another project: copy them, never rewrite them.

STEP 1 — Before copying anything, tell me where skills actually load from here. Report
each of these, and write "unknown" rather than guessing:
  - which Claude surface this is (Claude Code CLI, an IDE extension, claude.ai) and,
    for Claude Code, whether this workspace is a single repository or a monorepo;
  - which skill directories already exist and are read here: .claude/skills/ at the
    workspace root, .claude/skills/ in any subdirectory, ~/.claude/skills/, and any
    skills supplied by an installed plugin;
  - the names of the skills already in each of those directories, and for each one
    whether its SKILL.md ends with a `relearn:generated` footer.

STEP 2 — Propose the destination and the exact folder names, then wait for my yes. Say
which directory you would copy into and what that scope means (this repository only, or
every repository on this machine). If a skill of the same name is already there, do not
overwrite it: report its name, its line count, and whether it carries the generated
footer, and stop — a skill without that footer is hand-authored and copying over it
destroys work nothing can recover. Propose a renamed destination folder instead.

STEP 3 — Copy, do not author. Copy each SKILL.md byte for byte, including the YAML
front-matter at the top and the HTML comment on the last line, which carries a hash of
the body. Do not reformat, reflow, re-order, summarise, rewrite in your own words, merge
two layers into one file, or split one across several. Create no other files.

STEP 4 — Report each destination path and its line count, and confirm that no existing
file was modified or replaced. Expected line counts: global 451, domain-rust 205,
project-relearn 16, project-stochos-lab 16, project-design-architecture-tool 95.
```

The expected line counts are in the prompt because they are the one check the person
pasting it can make without reading the files. Unlike the rule counts above, these are
hand-written **and unchecked** — `tests/pack_counts.rs` reads rule counts off each skill
and line counts only off the copilot file, so nothing fails when a number here drifts.
`wc -l claude-pack/skills/*/SKILL.md` is the reading that cannot rot.

## Confirming a skill loaded

Placement is not proof, and neither is an answer from the session that just did the
copying — the file is in its context either way. In a **fresh** session, ask something
only these rules answer:

> What does R:parse-wide-then-range-check require, and how does it differ from
> R:parse-dont-validate?

A correct answer distinguishes parsing into a type wide enough to *represent* the
out-of-range value from minting the narrow witness at the boundary. A generic answer
means the skill did not load, and the first thing to check is the `description` — see
below, it is the only string that decides.

## These files are generated — do not edit them

Every `SKILL.md` here carries a `relearn:generated` footer with a `sha256` of its body.
Edit the rule and re-emit; an edit made here is overwritten on the next build, and
`relearn verify` reports it as drift in the meantime.

The pack is a **second emission** of the same rules the repository root carries, committed
so this folder is downloadable on its own. Rebuild and check it with:

```bash
relearn build  --targets claude --out claude-pack
relearn verify --targets claude --out claude-pack   # exit 0 when in sync
```

CI runs that `verify` on every push, on Linux and Windows. It has to be a separate step:
the pack lives outside every relearn-owned path, so a bare `relearn verify` cannot see it,
and without the step a rule change would rebuild the root artefacts and leave this copy
silently stale. This README is hand-authored and sits outside that check — which is why
every number in it is paired with the command that measures the real one.

## What a skill's `description` is for

The `description` in each `SKILL.md` front-matter is the **only** string Claude reads when
deciding whether to load that skill. It is generated as a lead sentence saying when the
layer applies, followed by as many rule titles as fit Claude's 1024-character cap, with any
remainder counted as a `+N more` tail. The full text of every rule is in the body of the
file, so nothing is lost — the cap bounds the trigger, not the content. A layer with more
rules than the cap fits therefore advertises fewer titles, not fewer rules, and the tail is
how you tell the difference.

## Where this comes from

`relearn` stores an engineering team's correction-derived rules **once**, in a neutral
versioned form, and compiles them out to every assistant's native instruction layer. This
pack is the Claude Skills target. `../copilot-pack/` is the same library as GitHub Copilot
instructions.

Each rule carries its provenance: the date it was written, the incident that caused it, the
class of error it retires, and — where the rule has since fired again — a dated record of
each recurrence. A rule annotated *"Has recurred N times"* is one prose has demonstrably
failed to hold; weight it accordingly. A rule annotated *"Also enforced by ..."* has
graduated to a stronger control **in the author's own environment** — a hook, a test, a
gate that does not exist in yours. Treat it as fully active unless you have installed the
equivalent.

Source, and the reasoning behind the design: <https://github.com/george-andrikopoulos/relearn>
