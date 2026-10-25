+++
tag = "R:a-model-turn-is-for-judgment"
title = "Spend a model turn only where the next step needs judgment"
error_class = "A chain of tool calls whose next step reads only a field of the last result is run as one model turn per call, paying a sampling round trip and the full intermediate result in context for a decision code could make -- or, run the other way, a chain whose every step needs judgment over the last result is moved into a script, which cannot skip the round trip, adds overhead, and parses blindly what the model should have looked at"
home = { kind = "domain", name = "agent-tooling" }
created = "2026-10-25"
origin = "codified"
source = "Programmatic tool calling, Claude API documentation, https://platform.claude.com/docs/en/agents-and-tools/tool-use/programmatic-tool-calling -- read 2026-10-24, re-read and every cited figure checked 2026-10-25"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-10-25 from Anthropic's documentation of programmatic tool calling, read 2026-10-24 and re-read the next day with every cited figure checked against the page. The page's own guidance is two-directional, which is what made it a rule rather than a feature note: fan-out and filterable multi-call chains are a strong fit, and \"strictly sequential workflows where each call depends on Claude reasoning over the previous result\" are named a weak fit, with the vendor's own benchmark showing them costing about 8 percent more for no change in score. The codification pass weighed three candidate classes from that page and kept one. Treating `allowed_callers` as access control -- the page says it \"is not a hard API-level block on direct calls\" -- is already `[R:anchor-outside-the-checked-partys-reach]` in its degenerate form, where the party constrained is also the party doing the checking; that rule is mined, so a `source` note is unconstructible on it, and nothing was added. Leaving a tool's output format undescribed is held by a type in the consuming project -- render the description's output schema from the result type, so the documented and the returned shape have one home -- and was dropped. The independent half of the third candidate is `[R:delegate-a-fan-out]`; what survived is the dependent half, which no existing rule reaches."
published_incident = "Codified from a vendor's published documentation for letting a model call tools from inside a script rather than one turn at a time, not mined from a failure. What made it a rule rather than a feature note is that the documentation argues both directions with its own measurements: a multi-tool workload spent over a third fewer input tokens with no loss of accuracy, while a workload of one or two strictly sequential calls per turn cost more for no change in score. The deciding variable in both is the same, and it is not the mechanism: it is whether each next call needs a judgement about the previous result or only a value read out of it."
+++

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
