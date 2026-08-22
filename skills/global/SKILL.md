---
name: global
description: "Rules for global. Covers: Update the doc in the same change as the thing it describes (A checked-in list or example (hygiene set, format sample, doc snippet) drifting from the reality it describes, so a reader trusts it and acts on stale guidance); Make illegal states unrepresentable (Designing types that permit contradictory or invalid states -- a bool beside an Option that can disagree, two fields that can contradict -- so the logic must defensively guard what the type should have forbidden); Measure cost-per-completed-task; never choose by price tier (Selecting a mechanism or model by its reputation or price tier rather than its measured cost to complete the task); No sentinel values: absent states are enum variants (Encoding a distinct state as a magic value of an existing type (0, -1, \"\", T::zero()) that downstream logic must remember to special-case); Never route judgment work to a weak model, and never embed a sub-tier local LLM (Wiring a meaningfully less capable model into a tool for work that needs judgment, on convenience or API-key-free grounds, so the tool is degraded wherever that model runs); Prefer by-construction impossibility over after-the-fact controls (Reaching for a runtime control (a check, a guard, a review step) to catch a mistake after it occurs when the design could have made that mistake impossible to express in the first place); Describe a practice from the artefact that defines it, never from the genre (Describing the user's own practice, system or process from domain convention or literature instead of the primary artefact that defines it, producing a fluent, plausible description of the wrong thing); Verify through the production path (Verifying a feature through a stand-in wiring (dev override, mock transport, alternate config channel) instead of the exact channel production uses)"
---

# global rules

## Update the doc in the same change as the thing it describes [R:doc-currency]

When you change a process, format, or artifact, update every checked-in description of it -- list, example, doc snippet -- in the same change. A doc that lags the code it describes is worse than no doc: a reader trusts it and acts on stale guidance. The behavior contract and its examples are only load-bearing if they are true.

Not to be confused with the sibling rule on a document's *internal* consistency
after an edit -- antecedents, cross-references, counts -- which currently lives
in the `~/.claude` skill layer under the tag this rule used to carry. This rule
is about a document's *external* currency: it still describes a reality that has
since moved. A file can pass one and fail the other.

The sibling is still named in prose rather than cited by tag, because it is not
in this library yet and a real citation would genuinely dangle. The *provenance*
above may now name it freely: since 2026-08-16 the linter reads citations from
rule bodies only, so recording why a tag changed no longer trips it.

## Make illegal states unrepresentable [R:make-illegal-states-unrepresentable]

Before writing logic, design the types so invalid states cannot be constructed: sum types over boolean flags, one field that cannot contradict another. If two fields can disagree, redesign until they cannot. R:no-sentinel-values is the corollary -- an absent or stopped state is an enum variant, not a magic value the surrounding logic must remember to special-case.

## Measure cost-per-completed-task; never choose by price tier [R:measure-cost-per-task]

Do not pick a mechanism or model by reputation or sticker price. State what it actually costs to complete the task -- tokens consumed times price, including retries -- and choose on that measured cost. After choosing, run one failure-mode check: under what configuration does this cause the exact harm it was chosen to prevent? Then bound that configuration.

## No sentinel values: absent states are enum variants [R:no-sentinel-values]

If "absent / stopped / unknown" is a real state, make it an enum variant, not a magic value of an existing type. Downstream code will forget to special-case a sentinel; it cannot forget a variant the compiler forces it to handle. If a range check reads a sentinel as a real quantity, it fails in the direction of the sentinel, not of safety.

## Never route judgment work to a weak model, and never embed a sub-tier local LLM [R:no-weak-model-for-judgment]

Work that needs judgment goes to a capable model. Do not embed a sub-tier local LLM (a 7B/13B behind Ollama, llama.cpp or similar) in a tool as a convenient, API-key-free fallback: a meaningfully dumber model degrades the tool it is wedged into, everywhere and silently, and the output looks like ordinary tool output rather than like a downgrade. When a tool needs intelligence, delegate to the capable model through the existing subscription -- the MCP server is the abstraction boundary and clients are peers. Note that the cost argument usually offered for the local model is the price-tier fallacy R:measure-cost-per-task names; but this rule is not an economic one and does not dissolve if the sums come out favourably. Mechanical, tool-restricted passes are a different matter and may be scoped tightly; the floor applies to work where the answer is a judgement.

## Prefer by-construction impossibility over after-the-fact controls [R:prefer-by-construction]

When a class of mistake can be designed out, design it out, rather than adding a control that catches it after the fact. A control that catches a mistake still admits the mistake; a design that cannot express the mistake retires the whole class. Rank the options by how little must be remembered for them to hold: a type the compiler enforces beats a test that samples beats a review step that relies on attention. R:make-illegal-states-unrepresentable is this rule in the type system.

## Describe a practice from the artefact that defines it, never from the genre [R:source-practice-from-its-artefact]

Before writing anything that *describes* how the user works -- their process, their file
layout, their tooling, their organisation, their role -- enumerate the primary artefacts
that define it, and read them first. The definition of a practice is the artefact that
embodies it: the skill that specifies it, the repository that implements it, the document
that governs it. Domain literature and industry convention describe the *genre*, not this
instance. A description sourced from the genre will be fluent, well-structured, plausible,
and about somebody else.

Run this check before drafting, not after: **what artefact would prove this description
wrong, and have I read it?** If the answer names a file that has not been opened, open it.
If no such artefact exists, the description is an assumption -- ask, and say plainly that
the question is being asked because nothing available settles it.

Clarifying questions must cover the *subject*, not only the framing. Asking about audience,
evidence and venue while silently assuming what the thing is produces a well-scoped,
well-sourced artefact about the wrong topic, and the correction costs a rewrite rather than
an edit. The cheapest question is the one that establishes what is being described.

This is the reading half of the sibling rule on building: search the existing artefacts
before constructing something new. That one prevents rebuilding what exists; this one
prevents *describing* what does not.

## Verify through the production path [R:verify-through-production-path]

Before declaring anything verified, run at least one check through the exact channel production uses: same env var, same startup script, same config file, same transport. A test that exercises a stand-in is evidence the stand-in works, not that the feature does. Ask: which line of production wiring did my test NOT execute? That line is where it breaks.

<!-- relearn:generated v0.1.0 sha256=7396ce72018d4ed0758664949e0473c3769b1a5beb0fc84d1a6e78c1f533b2c1 rules=R:doc-currency,R:make-illegal-states-unrepresentable,R:measure-cost-per-task,R:no-sentinel-values,R:no-weak-model-for-judgment,R:prefer-by-construction,R:source-practice-from-its-artefact,R:verify-through-production-path -- DO NOT EDIT; regenerate with `relearn build` -->
