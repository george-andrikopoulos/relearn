---
name: domain-agent-tooling
description: "Engineering discipline for working in agent-tooling. Covers: a model turn is for judgment"
---

# domain: agent-tooling rules

## Spend a model turn only where the next step needs judgment [R:a-model-turn-is-for-judgment]

> Written down from Programmatic tool calling, Claude API documentation, https://platform.claude.com/docs/en/agents-and-tools/tool-use/programmatic-tool-calling -- read 2026-10-24, re-read and every cited figure checked 2026-10-25.

For every edge in a chain of tool calls, ask what the next call reads from the previous
result: **a field, or a conclusion?** A model turn is worth its round trip only when the
answer is a conclusion.

If the next call reads a field -- an id out of the last response, a row count, a status, a
filter over a list -- the dependency is mechanical, and code can follow it. Routing it
through the model instead pays a full sampling round trip per call and pours every
intermediate result into context so that the model can do what one line of a script would
have done. This is the shape that tempts people to route through the model *because* it is
dependent: "each call needs the last result" sounds like a reason to stop and think, and
usually it is only a reason to read a key. Put the mechanical stretch in one script, and
return the model only what it has to judge.

If the next call reads a conclusion -- whether a result looks wrong, which of three
readings of an ambiguous record is meant, whether to stop -- the round trip cannot be
skipped. Moving that edge into a script does not remove the judgment; it hands it to
whatever the script hard-coded, and pays container and script overhead on top. The case
is real and measured: on workloads of one or two strictly sequential calls per turn,
scripting them left scores unchanged and cost more, while on a broad multi-tool workload
it cut billed input tokens by over a third. Neither direction is the default; the shape
of the chain decides.

A second cost hides in the mechanical direction and is the reason the line matters: a
result that stays inside a script never enters context, so the model never sees it. That
is the saving, and it is also a blindfold. A tool that can return an error string, a
partial page or an anomalous value in a field the script parses blindly belongs on a
judgment edge, or the script has to check that field and hand back what it found.

The independent fan-out -- the same lookup over an already enumerated list -- is the easy
case and is `[R:delegate-a-fan-out]`'s shape with a different mechanism: once the list
exists, walking it one model turn at a time buys nothing. This rule is the dependent case,
which that one does not reach. And because the trade turns on workload shape rather than
on the mechanism's reputation, decide it the way `[R:measure-cost-per-task]` says: billed
tokens with and without, on representative traffic, before enabling it everywhere.

Failure-mode check: **for each edge, name what the next call takes from the last result.**
If it is a value at a known path, the round trip bought nothing. If it is a decision, the
script cannot make it -- and if you cannot say which, you do not yet know the chain well
enough to choose either.

<!-- relearn:generated v0.1.0 sha256=5eb6c035100f041ba535891659985ee405124623536b417c58548dbf32c61811 rules=R:a-model-turn-is-for-judgment -- DO NOT EDIT; regenerate with `relearn build` -->
