# Worked example: one incident → one rule → five instruction layers

This walks a single real correction all the way through `relearn`: the incident that
caused it, the one neutral rule file it becomes, and the five vendor instruction files
`relearn build` compiles it out to. Every emitted block below is **actual `relearn build`
output**, not a mock-up — see [Reproducing this](#reproducing-this) to regenerate it.

## 1. The incident

While building `relearn` itself, two of the emitters ordered rules by their home-slug
*string*. Sorting `domain-rust` and `global` lexically puts `domain-rust` first (`'d' < 'g'`),
so the broadest layer landed in the middle of the file — an ordering nobody chose. It sat
undetected until two ordering tests were found asserting *opposite* orders. The lexical sort
was an alphabetical accident standing in for a decision, and it would have changed silently
the day a slug string changed.

The fix was an explicit `home_rank` (general → specific). The *correction* — the thing that
must outlive the session so the next engineer does not re-derive it — is the rule below.

## 2. The rule (the one neutral source)

One file, `rules/order-by-explicit-rank.md`: TOML front-matter carrying the rule's identity,
home, provenance (the date and the triggering incident), and status, followed by an
imperative markdown body.

```toml
+++
tag = "R:order-by-explicit-rank"
title = "Order by an explicit rank, not an incidental string sort"
error_class = "Deriving a semantic ordering from an incidental lexical sort (e.g. sorting home layers by their slug string), so the order is an alphabetical accident rather than a stated intent"
home = { kind = "project", path = "relearn" }
created = "2026-08-13"
status = { kind = "active" }
incident = "relearn (2026-08-13): the copilot and AGENTS.md emitters ordered rules by home slug string, which sorts domain-rust before global ('d' < 'g') -- putting the broadest layer in the middle by accident and leaving two ordering tests asserting opposite orders. Caught in integration review; fixed with an explicit home_rank (global -> domain -> project)."
+++

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.
```

`relearn` refuses to accept a rule without that provenance: a rule that cannot say where it
came from cannot be audited, and an unauditable rule library becomes the sediment the tool
exists to fight.

## 3. The five instruction layers (`relearn build`)

This rule's home is a **project** layer, so it compiles out to all five targets — including
the Claude project layer at `.claude/rules/<home-slug>.md`, which carries that home's rules
only. (That target wrote to the repository-root `CLAUDE.md` until 2026-08-22; the path is now
derived from the rule's `Home` so it cannot collide with a hand-authored charter — see the
ARCHITECTURE decisions log.) Each emitted file carries a
generated-by header naming its source rule(s) and a `sha256` of the body; `relearn build`
refuses to overwrite any file lacking that marker, and `relearn verify` reports any that
drifted from it.

### Claude skill — `skills/project-relearn/SKILL.md`

One skill **per home layer** (not per rule), so the always-resident skill metadata stays
bounded. The YAML `description` aggregates the home's rules for trigger coverage.

```markdown
---
name: project-relearn
description: "Rules for project: relearn. Covers: Order by an explicit rank, not an incidental string sort (Deriving a semantic ordering from an incidental lexical sort (e.g. sorting home layers by their slug string), so the order is an alphabetical accident rather than a stated intent)"
---

# project: relearn rules

## Order by an explicit rank, not an incidental string sort [R:order-by-explicit-rank]

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.

<!-- relearn:generated v0.1.0 sha256=f99fa86bf9ad30f3dabe1d3f53472f4b2ef60e484728e31a2ae61fff3a9e9464 rules=R:order-by-explicit-rank -- DO NOT EDIT; regenerate with `relearn build` -->
```

### Cursor rule — `.cursor/rules/order-by-explicit-rank.mdc`

One `.mdc` per rule. `globs`/`alwaysApply` are **derived from the rule's `Home`**, never
carried on the rule: a project rule applies across its tree, so `alwaysApply: true` with no
globs. The filename is the colon-free tag body.

```markdown
---
description: Order by an explicit rank, not an incidental string sort (Deriving a semantic ordering from an incidental lexical sort (e.g. sorting home layers by their slug string), so the order is an alphabetical accident rather than a stated intent)
globs: 
alwaysApply: true
---

# Order by an explicit rank, not an incidental string sort [R:order-by-explicit-rank]

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.

<!-- relearn:generated v0.1.0 sha256=555f328912de4fc00f8db481b98e8cd8ce3a83fb4a2f1058d9d948b45dc9ab86 rules=R:order-by-explicit-rank -- DO NOT EDIT; regenerate with `relearn build` -->
```

### GitHub Copilot — `.github/copilot-instructions.md`

A single concatenated instructions file (here, one rule).

```markdown
# Copilot instructions

## Order by an explicit rank, not an incidental string sort [R:order-by-explicit-rank]

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.

<!-- relearn:generated v0.1.0 sha256=7175014d41e8a7f48054ee2895cc610e42c930dbac57b4b1fc22a76770d7ea2f rules=R:order-by-explicit-rank -- DO NOT EDIT; regenerate with `relearn build` -->
```

### `AGENTS.md` (cross-assistant convention)

```markdown
# Agent instructions

## Order by an explicit rank, not an incidental string sort [R:order-by-explicit-rank]

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.

<!-- relearn:generated v0.1.0 sha256=62122865a5cc009cc59e1fe348aa5d884bb90b10bc380a45910574c65cd4b394 rules=R:order-by-explicit-rank -- DO NOT EDIT; regenerate with `relearn build` -->
```

### Claude project layer — `.claude/rules/project-relearn.md` (that home's rules only)

```markdown
# Project rules

## Order by an explicit rank, not an incidental string sort [R:order-by-explicit-rank]

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.

<!-- relearn:generated v0.1.0 sha256=2d0ec6025276eda4a2589e39ee9266413a7c109ac8f0f11ad868257e09213a9a rules=R:order-by-explicit-rank -- DO NOT EDIT; regenerate with `relearn build` -->
```

## The loop this closes

The correction was made once, written once, and now travels to every assistant the team uses
— Claude, Cursor, Copilot, any tool that reads `AGENTS.md`, and the project's own Claude layer
— from a single audited source. When the rule graduates to a stronger control (a lint, a
commit hook, a type), its status changes in the one file and the change propagates on the next
build; when it is retired, `relearn` stops emitting it so no assistant is left instructing
withdrawn guidance. That is the governance loop: **persist or perish**, and **one home per
rule**.

## Reproducing this

Every block above is real output. To regenerate it from a checkout:

```sh
mkdir -p /tmp/one-rule && cp rules/order-by-explicit-rank.md /tmp/one-rule/
relearn build --rules /tmp/one-rule --out /tmp/example
relearn verify --rules /tmp/one-rule --out /tmp/example   # exit 0: the tree matches the rule
```

(The `sha256` in each header is a hash of that file's body, so the values above are stable as
long as the rule body is unchanged.)
