---
name: global
description: "Rules for global. Covers: Update the doc in the same change as the thing it describes (A checked-in list or example (hygiene set, format sample, doc snippet) drifting from the reality it describes, so a reader trusts it and acts on stale guidance); Make illegal states unrepresentable (Designing types that permit contradictory or invalid states -- a bool beside an Option that can disagree, two fields that can contradict -- so the logic must defensively guard what the type should have forbidden); Measure cost-per-completed-task; never choose by price tier (Selecting a mechanism or model by its reputation or price tier rather than its measured cost to complete the task); No sentinel values: absent states are enum variants (Encoding a distinct state as a magic value of an existing type (0, -1, \"\", T::zero()) that downstream logic must remember to special-case); Never route judgment work to a weak model, and never embed a sub-tier local LLM (Wiring a meaningfully less capable model into a tool for work that needs judgment, on convenience or API-key-free grounds, so the tool is degraded wherever that model runs); Pin the line endings of text a machine executes or hashes (Leaving line endings to the checkout for a file whose bytes are load-bearing -- a script an interpreter runs, or generated content whose hash is compared -- so a clone on one platform silently produces a file that no longer works or no longer matches); Prefer by-construction impossibility over after-the-fact controls (Reaching for a runtime control (a check, a guard, a review step) to catch a mistake after it occurs when the design could have made that mistake impossible to express in the first place); Repair the artefact that made the false claim, not only the doc about it (Closing an incident by writing or correcting prose while the executable artefact that actually misled — a script's printed path, a status line, a generated header, a success message — goes on emitting the same false claim, so the defect stays fully operational behind a note that makes it look handled); Describe a practice from the artefact that defines it, never from the genre (Describing the user's own practice, system or process from domain convention or literature instead of the primary artefact that defines it, producing a fluent, plausible description of the wrong thing); Verify through the production path (Verifying a feature through a stand-in wiring (dev override, mock transport, alternate config channel) instead of the exact channel production uses)"
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

## Pin the line endings of text a machine executes or hashes [R:pin-eol-for-executable-text]

When a file's exact bytes are load-bearing, do not leave its line endings to whatever the
checkout decides. Two kinds of file qualify: text a machine executes, where a stray
carriage return on the shebang means the interpreter is not found; and generated text
whose content is hashed and compared, where a rewritten line ending changes the hash and
the file reports as modified by a human who never touched it.

Pin it structurally, in `.gitattributes`, for the paths that need it. This is a
by-construction fix applied once at the repository boundary, not a check that runs later
and reports what has already gone wrong.

Do not repair this by teaching the comparison to forgive line endings. Normalising before
hashing makes the check tolerant of a class of real edit, and it hides the platform
difference rather than removing it. The guarantee is about the bytes on disk, so it is the
bytes on disk that must be fixed.

The characteristic damage is that it is invisible from where the work was done. The
authoring platform stays green; the breakage appears only on the other one, and it appears
as a symptom that names something else entirely -- a missing interpreter, a hand-edited
file, a failing gate with nothing wrong in the diff.

This is the at-rest half of a pair. Its sibling governs bytes in flight -- what another
tool emits into a pipe at runtime, which no file attribute can reach, so that fix belongs
at the consuming end instead. A repository can satisfy either and fail the other.

## Prefer by-construction impossibility over after-the-fact controls [R:prefer-by-construction]

When a class of mistake can be designed out, design it out, rather than adding a control that catches it after the fact. A control that catches a mistake still admits the mistake; a design that cannot express the mistake retires the whole class. Rank the options by how little must be remembered for them to hold: a type the compiler enforces beats a test that samples beats a review step that relies on attention. R:make-illegal-states-unrepresentable is this rule in the type system.

## Repair the artefact that made the false claim, not only the doc about it [R:repair-the-lying-artefact]

When an incident traces to a false claim, find the artefact the person was actually
reading at the moment they were misled, and repair **that** first. Usually it is not the
document — it is something that printed during the work: a script's final "Binary:" line,
a deploy script's success message, a generated file's header, a status endpoint, a test
name. A document is consulted; an artefact is *emitted at you* while your attention is on
the task. That asymmetry decides who gets believed.

A doc and a script disagreeing is not a tie. People run the script. Correcting the doc and
stopping there leaves the defect fully operational and adds a paragraph that makes it look
handled — the worst of both, because the next occurrence now has a written warning standing
over it as evidence that someone already dealt with this.

Apply the test before closing: **could the same person be misled again in exactly the same
way without ever opening the document I just fixed?** If yes, the fix has not landed. Ask
also what else in the repo asserts this same fact — a README snippet, a CI summary, a
printed usage line — because a claim usually has more than one mouth.

Then push it down a layer rather than restating it: make the artefact *derive* what it
reports instead of asserting it (resolve the path, stat the file, read the version it
actually built), and add the mechanical check that fails any future artefact making the
unresolved claim. Deriving beats asserting for the same reason
`[R:prefer-by-construction]` prefers designs to guards — a derived claim cannot drift from
what it describes, so it cannot go stale the way `[R:doc-currency]` describes.

This is the reporting half of `[R:verify-through-production-path]`. That rule says to
exercise the real channel; this one says the real channel must also tell the truth about
what it produced — verifying through a production path that reports a path it never
resolved proves nothing.

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

<!-- relearn:generated v0.1.0 sha256=5720340030f697b0bb43e3f7a8e594eda1adeb4f034dd72a345f5e2995eb435e rules=R:doc-currency,R:make-illegal-states-unrepresentable,R:measure-cost-per-task,R:no-sentinel-values,R:no-weak-model-for-judgment,R:pin-eol-for-executable-text,R:prefer-by-construction,R:repair-the-lying-artefact,R:source-practice-from-its-artefact,R:verify-through-production-path -- DO NOT EDIT; regenerate with `relearn build` -->
