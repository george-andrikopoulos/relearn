# claude-pack — the rule library as Claude Skills

A downloadable copy of this repository's rules, emitted as **Claude Skills**: one skill
per home layer, each a folder containing a `SKILL.md`. Drop it into Claude Code, or
install it on claude.ai, without cloning this repository or building anything.

```
claude-pack/
└── skills/
    ├── global/SKILL.md               24 rules — apply to every project and language
    ├── domain-rust/SKILL.md          18 rules — the Rust type-driven design discipline
    ├── project-relearn/SKILL.md       2 rules — specific to the relearn repository
    └── project-stochos-lab/SKILL.md   2 rules — specific to the stochos-lab repository
```

**Take the layers you want.** `global` is the one most people want on its own; add
`domain-rust` if you write Rust. The two `project-*` skills are included for completeness
and are almost certainly *not* what you want in another repository — they describe two
specific repos, and a rule that names a project you are not working in is noise.

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

### Claude Code — every project on the machine

The same folders, under `~/.claude/skills/` instead. These load in any repository, so put
only `global` (and a language layer you always use) here — a project layer installed
machine-wide is the one-home-per-rule violation this tool exists to prevent.

### claude.ai (web and desktop app)

A skill is uploaded as a folder — or a zip of that folder — with `SKILL.md` at its root,
which is exactly what each directory here is. Anthropic's own documentation is the place
to check where the upload lives in the current interface and which plans have Skills
enabled; it moves, and a menu path transcribed here would go stale silently.

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
silently stale.

## What a skill's `description` is for

The `description` in each `SKILL.md` front-matter is the **only** string Claude reads when
deciding whether to load that skill. It is generated as a lead sentence saying when the
layer applies, followed by as many rule titles as fit Claude's 1024-character cap, with any
remainder counted (`+7 more`). The full text of every rule is in the body of the file, so
nothing is lost — the cap bounds the trigger, not the content.

## Where this comes from

`relearn` stores an engineering team's correction-derived rules **once**, in a neutral
versioned form, and compiles them out to every assistant's native instruction layer. This
pack is the Claude Skills target. `../copilot-pack/` is the same library as GitHub Copilot
instructions.

Each rule carries its provenance: the date it was written, the incident that caused it, the
class of error it retires, and — where the rule has since fired again — a dated record of
each recurrence. A rule annotated *"Has recurred N times"* is one prose has demonstrably
failed to hold; weight it accordingly.

Source, and the reasoning behind the design: <https://github.com/george-andrikopoulos/relearn>
