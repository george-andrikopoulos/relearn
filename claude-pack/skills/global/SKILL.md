---
name: global
description: "Engineering discipline that applies to every project and language. Covers: guarantee needs a reader; verdict survives the channel; doc currency; no silent spend; signal needs a consequence; verify through production path; check the claim you inherit; claude md recreates the project; decisions log records rejected alternatives; delegate a fan out; detector excludes own definitions; features ledger names its artefact; five files no more; make illegal states unrepresentable; measure cost per task; measure the claim not a subset; no sentinel values; no stale push over fresh; no weak model for judgment; one home per rule; pin eol for executable text; prefer by construction; price every dependency; reconcile wiring at start; repair the lying artefact; review against contract not plan; revision integrity; search before you build; source practice from its artefact; wired artifact; definition of done every change; money is not a float; case collision"
---

# global rules

## Check for a case-differing sibling before creating a file [R:case-collision]

> Also enforced by hook:no-case-collision.

Before creating any file, check for an existing sibling whose name differs only by case or by Unicode normalisation. Where a repository is synced between a case-sensitive filesystem and a case-insensitive one, the case-insensitive semantics are the binding constraint: two such names are one file, and the loser is whichever was written second.

Merge into the existing file, with a section heading, rather than creating the colliding name. The instinct to create a properly-capitalised new file is the failure -- the convention is not worth a silent overwrite on the other machine.

The damage is invisible from where the work was done. On the authoring filesystem both files exist and everything looks correct; the collision appears only after a sync, as content that vanished with no diff and no error to attribute it to.

This rule has graduated: a write-time hook now blocks the collision deterministically, so the instruction layer no longer has to carry it for that path. It still applies to every creation path the hook does not see -- a shell redirect, a git operation, another tool -- which is why the guidance is kept rather than retired.

## Check an inherited claim about the world before repeating it [R:check-the-claim-you-inherit]

Before repeating a document's factual claim about live state, query the state.

A document can tell you two different kinds of thing, and they do not deserve the same
trust. A claim about *intent* -- why a threshold is 18, what was rejected and why, which
invariant a file exists to hold -- is only recorded in prose, and the document is the
authoritative source. A claim about *the world* -- a sensor is absent, a service is
disabled, a count is 23, a capability is unenforced -- describes something that can be
observed directly, and the document is a cached reading with no expiry date.

Repeating the second kind without observing is how a stale or never-true claim gets
laundered into a fresh report. It is worse than the original error, because the claim
now arrives with today's date and the authority of having just been "checked", and
because the report is the thing the reader acts on. Whoever wrote it may have been right
at the time, may have been guessing, or may have copied it from somewhere else; none of
that is visible by the time you are reading it, and a claim copied into four documents
looks four times corroborated while resting on one unverified assertion.

This is the reader's half of `[R:doc-currency]`. That rule tells the writer to update
the document in the same change as the thing it describes, and it will sometimes fail --
so the reader is the last position where the error can still be caught. It is also the
mirror of `[R:measure-the-claim-not-a-subset]`: that one guards the moment you are about
to *overwrite* a recorded figure, this one the moment you are about to *repeat* it.
Between them, a recorded number is never simply passed along unexamined in either
direction.

The check is cheap and it is bounded. You are not re-deriving the document; you are
spot-checking the specific assertions you are about to put your own name to, and only
those that name observable state.

Failure-mode check: **is this a claim about the world, and can I observe the world?**
If both, the document is a hypothesis and your report needs the observation, not the
quotation. If you cannot observe it, repeat it with its provenance attached -- "TODO.md
records X, unverified" -- rather than asserting it flat.

## The project charter is written to the recreation standard [R:claude-md-recreates-the-project]

Write the project charter so that from it alone the project could be rebuilt. That is the bar, and it is testable: hand the file to someone with the toolchain and nothing else, and ask what they could not reconstruct.

It holds the purpose, the core design decisions **with their reasons**, the invariants that must never break, the commands to build, run and test, and pointers to the other standing documents. The reasons are the part that decays first and matters most: a decision recorded without its why is indistinguishable from an accident, so the next reader either reverses it or preserves it superstitiously, and both are expensive.

Check it explicitly on every change that touches design, as a question rather than a glance: *could this file still recreate the project?* Skipping that check is how the standard becomes a lie -- not in one edit, but in twenty, each individually defensible.

The charter is the project layer and nothing else. Domain rules are referenced from it, never copied into it (R:five-files-no-more), and a claim it makes about an enforcing control is subject to R:guarantee-needs-a-reader like any other.

## The decisions log records the why and what was rejected, append-only [R:decisions-log-records-rejected-alternatives]

Keep an append-only decisions log in the architecture document: date, decision, why, alternatives rejected. Append; never rewrite an entry to match what was later believed, because the value of the log is that it records what was known at the time.

The rejected alternatives are the half that gets dropped and the half that pays. A decision with its reason tells the next reader why this path was taken; the rejected list tells them why *their* idea was not, which is the question they actually have. Without it, the obvious alternative is re-proposed on a cycle set by staff turnover, and each round costs the same argument with less context than the last.

Anonymous structure accumulates as sediment. Given a module boundary with no recorded reason, a reader must either treat it as sacred or treat it as arbitrary, and neither is true. Write the entry when the decision is made, while the alternatives are still in mind -- reconstructed later, the rejected list is always the flattering one.

Record the decision that surprised you most, not the one that was easiest to write down. This is the same discipline R:claude-md-recreates-the-project applies to the charter: what a future reader cannot reconstruct is exactly what must be written.

## The definition of done runs on every change, and skips are declared [R:definition-of-done-every-change]

> Partly enforced by hook:tdd-gate; checks 2 to 5 entirely -- the regression pass, the contract, the charter and the open-work list -- plus check 1 in every language but Rust, and check 1 wherever the enforcing artefact is a TYPE rather than a test file, which under a types-first discipline is the common case is held by this instruction alone.

A change is not done until five checks pass, in order:

1. **Code and its enforcing artefact ship together.** The type, property or test that locks the new behaviour is in the same change. Never "tests later" -- later is a different change, made by someone with less context, competing against new work.
2. **Regression pass.** Re-read the behaviour contract and *run* the enforcing artefacts of every feature this change could plausibly touch. A fix that breaks another documented feature is not a fix. This is the check that makes the contract protective rather than descriptive.
3. **The contract is updated** -- a new entry with its artefact, or an existing entry's artefact revised.
4. **Charter and architecture sync.** Ask explicitly whether the charter could still recreate the project and whether the architecture still describes it, and append any decision made to the log with its why.
5. **The open-work list is updated** -- done items cleared, discovered work added.

State the five results at the end of the task, briefly. The statement is not ceremony: it is what makes a skipped check visible to the person who can price the skip.

When a request conflicts with the discipline -- *just patch it quickly* -- do the patch, then say which checks were skipped and what exposure that creates. Never silently drop the discipline, and never block the user with process. The skip is the user's call to make; concealing that it happened is not.

Checks 1 and 2 do not apply to every change, and saying so is part of doing them. A documentation-only change has no enforcing artefact to ship. Declaring a check inapplicable is honest; quietly omitting it and reporting five greens is the failure this rule names.

## Delegate a fan-out once the work has been enumerated [R:delegate-a-fan-out]

Compiler-driven repair is genuinely serial while you are still deriving what is broken:
each fix changes the next error. It stops being serial the moment a build, a gate or a plan
has **enumerated** N independent follow-ups. From there the edits do not depend on each
other, and walking them one at a time buys nothing. The fan-out point is after the list
exists, not before.

The same shape covers a documentation set updated once per release, a review that reads
several files against one contract, and research needing several independent sources.

**When delegation is disabled, put the choice -- do not merely announce the cost.** This
rule is routinely overridden: a session's harness policy may forbid delegation outright,
and that policy wins by default. What it does not do is make the cost disappear, and a
notice issued after you have already chosen is information rather than a decision. At the
first real fan-out, state the fork in one line -- what would be delegated, roughly how many
serial round trips it replaces, and what turns it on -- then carry on serially unless told
otherwise, without repeating the offer. Pointing at a settings toggle does not count: a
global switch someone must go and change is a regime change, not a choice about *this*
task.

Failure-mode check: **has something already listed this work for me?** If a build, a gate
or a plan produced the list, the serial walk is a choice you are making on somebody else's
budget.

Sibling, deliberately not merged. `[R:no-silent-spend]` is the general form -- every trade
of the user's time against their money against thoroughness. This rule is kept to the
delegation instance so the two carry separate metrics: two rules with distinct surfaces can
be told apart by a review and each carries its own evidence, where one wide rule that fires
for either reason is falsifiable by nothing.

## A detector excludes its own definitions from its scan [R:detector-excludes-own-definitions]

Any check whose subject matter is text it must itself contain -- a linter, a secret scanner, a policy grep, a rules sweep -- matches itself by construction. Exclude the detector's own source, comments, and docstrings from its scan, strip comments before matching, and normalise paths to placeholders so documentation *about* a pattern is never read as an instance of it.

The reason is not tidiness. An always-red check gets muted, and a muted check is worse than no check, because the system still looks guarded. Alarm fatigue is negative value, not neutral: it spends the attention that a real finding will need, and it trains the reader to skip exactly the output that will one day matter.

This extends past the detector's own file to anything that quotes the pattern for a legitimate reason. A correction note explaining that a bad string was removed, written with the bad string in it, becomes a permanent false positive in the very sweep it was written to satisfy -- so describe the string, do not reproduce it, and where reproduction is genuinely necessary put it somewhere the scan excludes by rule rather than by luck.

Ask before shipping any scan: *does this check appear in its own corpus, and if so what does its output look like on a clean tree?* If the clean-tree output is not empty, the check does not yet work, however correct its logic.

## Update the doc in the same change as the thing it describes [R:doc-currency]

> Has recurred 1 time(s) since it was written; most recently 2026-09-13.

When you change a process, format, or artifact, update every checked-in description of it -- list, example, doc snippet -- in the same change. A doc that lags the code it describes is worse than no doc: a reader trusts it and acts on stale guidance. The behavior contract and its examples are only load-bearing if they are true.

Not to be confused with R:revision-integrity, the sibling on a document's
*internal* consistency after an edit -- antecedents, cross-references, counts.
This rule is about a document's *external* currency: the text is self-consistent
and still describes a reality that has since moved. A file can pass one and fail
the other.

That sibling was named here in prose rather than by tag from 2026-08-16 until
2026-08-25, because it lived only in the `~/.claude` skill layer -- under the tag
this rule used to carry -- and a real citation would have dangled. It was ported
into this library on 2026-08-25, so the citation above is now a live reference
and this paragraph is the record of why it was not one before.

## Every feature-ledger entry names the artefact that enforces it, or declares itself exposed [R:features-ledger-names-its-artefact]

Every entry in the behaviour contract carries the artefact that enforces it: the type that makes violation unrepresentable, the property test that covers the space, or the named unit test that pins the case. No entry without one.

Where a feature genuinely cannot be enforced yet, the entry says so in those words -- **exposed, nothing yet** -- and the gap goes to the open-work list until it closes. That is not a failure of the ledger; it is the ledger working. What must never happen is a row that reads like a guarantee because it is written in the same voice as the rows beside it.

Prefer the artefact highest in the hierarchy that can hold the guarantee: a type beats a property beats a unit test. And the artefact must exercise the **wired** behaviour, not the component alone -- R:wired-artifact is the sharp edge here, because a type's own unit tests prove the type works and say nothing about whether anything produces or consumes it.

Record the **invocation**, not only the artefact. A test that exists and is never run is enforcement on paper: name the command, and require that the command named is the one the pipeline actually takes. A gate whose green invocation needs a flag teaches every reader that the default is noisy, and that is how a control decays into decoration.

The ledger is only load-bearing if it is read before every change, which is what makes R:definition-of-done-every-change its second check rather than a courtesy.

## A project carries five standing documents, and resists a sixth [R:five-files-no-more]

A project keeps exactly five standing documents, each with one job:

- `CLAUDE.md` — the recreation standard: purpose, design decisions with their *why*, invariants, build and test commands.
- `ARCHITECTURE.md` — modules, data flow, perimeter, and an append-only decisions log.
- `FEATURES.md` — the regression ledger, every entry naming the artefact that enforces it.
- `TODO.md` — open work, including every unenforced gap the ledger declares.
- `README.md` — the public face, and the only one of the five written for outsiders. Keep the other four written for the maintainer and the machine.

Resist the sixth. Every resident document taxes attention on the others, and the tax is paid by the documents that matter most, because they are the longest. New standing knowledge goes *into* one of the five, or into a domain rule if it generalises past this project -- never into a new file whose creation felt tidy at the time.

The two failure classes the set exists to kill are staleness and unenforced guarantees. A file claiming completeness that has drifted is a confident lie, and confident stale context is read and acted on; a documented feature with no executable check is shipped and inert. Both are worse than the absence of the document, because both stop people looking.

Do not duplicate domain rules into the project layer. Reference them. A rule stated in two places forks and rots, and the copy is always the one someone reads.

## A stated guarantee names what enforces it, or is deleted [R:guarantee-needs-a-reader]

> Partly enforced by test:tests/ledger.rs::every_enforced_by_row_names_an_artefact_or_declares_itself_exposed + test:tests/pack_counts.rs; any guarantee stated outside FEATURES.md and the pack READMEs is held by this instruction alone.

> Has recurred 3 time(s) since it was written; most recently 2026-09-20.

Every safety claim in prose names the line, test, or check that enforces it -- or the sentence is deleted. A guarantee with no reader is worse than no guarantee, because it is read as coverage and it ends the inquiry.

A claim of completeness must also state what the check actually consumed. Not "verified", but "verified by grepping `model:` across N agent files" -- so the gap between the scope of the check and the scope of the claim is visible on the face of the entry rather than reconstructable only by rerunning it. Most false completeness claims are not lies; they are a narrow check reported in wide language.

**Error paths are where these hide.** A message asserting a state must be produced by *checking that state*, not by which branch printed it. The failure path is the one nobody exercises, so a confident sentence with nothing behind it survives there longest, and it is read at exactly the moment the reader is least able to question it. Ask of every error message: *did anything read the world before this printed?*

Where a mechanical reader exists, use it and leave the prose to what it cannot see: an unread binding is already a hard error under a compiler run with warnings denied. What no linter can see is a true-looking sentence with nothing behind it, and that is what this rule is for.

The test: *what would have to be true for this sentence to be false, and what reads that?* R:wired-artifact is the neighbouring failure -- there a check exists and accepts forgeable evidence; here there is no check at all.

## Make illegal states unrepresentable [R:make-illegal-states-unrepresentable]

Before writing logic, design the types so invalid states cannot be constructed: sum types over boolean flags, one field that cannot contradict another. If two fields can disagree, redesign until they cannot. R:no-sentinel-values is the corollary -- an absent or stopped state is an enum variant, not a magic value the surrounding logic must remember to special-case.

## Measure cost-per-completed-task; never choose by price tier [R:measure-cost-per-task]

Do not pick a mechanism or model by reputation or sticker price. State what it actually costs to complete the task -- tokens consumed times price, including retries -- and choose on that measured cost. After choosing, run one failure-mode check: under what configuration does this cause the exact harm it was chosen to prevent? Then bound that configuration.

## Match the scope before contradicting a recorded figure [R:measure-the-claim-not-a-subset]

A figure is already written down, your measurement disagrees, and you are about to declare
the document wrong. Usually it is. When it is not, you replace a true figure with a false
one -- and the false one now carries the authority of having been measured, pushed there
by the very rules that tell you to keep documents current (`[R:doc-currency]`,
`[R:repair-the-lying-artefact]`). This is their failure mode.

State the scope the recorded figure claims, then show that your measurement covers exactly
that scope -- no wider, no narrower. A filter, an include pattern, a package flag, a module
path, a date range: each one silently defines a subset, and comparing a subset's count to a
superset's claim manufactures a discrepancy out of nothing.

**Prefer an enumeration you can read to a count you can only trust.** A list flag, printing
the matches, thirty lines you can eyeball -- all beat one integer you believe, because the
excluded set is usually visible at a glance in the enumeration and invisible in the total.

Failure-mode check: **what would my measurement miss that the claim includes?** If you
cannot name the excluded set, you have not established the scope, and you are not entitled
to overwrite the number.

Where a figure is load-bearing enough to be worth arguing about, it is worth a check that
owns its own denominator. The durable fix moves the scope definition out of the reader's
hands and into a gate, so the claim and the measurement can no longer drift apart --
`[R:prefer-by-construction]` applied to a number.

## Money is an exact quantity, and no binary float holds one [R:money-is-not-a-float]

> Partly enforced by gate:scripts/no-float-money.sh (pre-push and CI, both platforms); the founding decision itself, which is the incident this rule was mined from -- a greenfield build has no declaration yet for a scan to read; every language but Rust, where `double` and `float` fail identically; a monetary field the vocabulary does not name; the whole division half -- a money type exposing `Div`, and a rounding mode defaulted on a caller's behalf; a cross-currency addition, which needs the currency in the type and cannot be read from a field name; a fixed-point type with a binary radix, which the scan reads as clean while the rounding is merely relocated; and any line marked `// money-scan: measured` is held by this instruction alone.

Money is an exact decimal quantity. No binary floating-point type holds one, and a money type does not expose division.

**Money is not a real number.** `0.1` has no finite binary expansion, so a `f64` holding an amount is already wrong before any arithmetic happens. Every operation compounds it, and the error is invisible at every point where anyone looks — it surfaces at reconciliation, against a system that counted in integers, as a discrepancy nobody can attribute to a line of code. The size of the error is not the argument. A quantity that must agree exactly with another party's number cannot be stored in a representation that cannot express the number.

**The newtype is necessary and not sufficient.** `Price(f64)` satisfies `[R:newtype-liberally]` completely — and that rule's own example is `Miles(f64)` beside `Kilometers(f64)`, which is the same shape. It stops two concepts being swapped; it says nothing about what is inside one of them. A reader who has applied it has not applied this. The defect here is the representation the newtype wraps, so the two rules compose rather than overlap: give the concept its own type, **and** give that type an exact representation.

**The representation.** A fixed-scale decimal type, or integer minor units with the scale declared once in the type and never re-derived at a call site. In Rust that is `rust_decimal` — 128 bits, `Copy`, no allocation, so it is usable on a latency-bounded path and the usual excuse for reaching back to `f64` does not apply. In Java it is `BigDecimal`, never `double`. **A fixed-point type with a binary radix is not a fix**: it relocates the rounding without making the decimal quantity representable, which is the defect.

**Division is the second half, and no representation fixes it.** £100 split three ways has no exact answer in currency. A flawless decimal type still cannot divide it, because the answer does not exist — the problem is *allocation*, not precision, and a type that offers `Div` invites the caller to accept a quotient that quietly discards a penny. So **the money type does not implement `Div`.** Splitting goes through a function that returns the parts with the remainder distributed by a stated policy, so the parts sum to the original by construction and "divided money and lost a penny" is unconstructible rather than a review item somebody might catch. Rounding mode is an explicit argument at every site that needs one; a default rounding mode is a decision taken on behalf of a caller who never saw it.

**Currency is part of the type.** `Money<GBP>` and `Money<USD>` are different types and adding them does not compile. This costs nothing at runtime and retires a class of production incident that review does not reliably catch, because a cross-currency addition reads correctly in every line of code that contains it.

**The failure-mode check, before declaring any numeric field:** *is this quantity counted or measured?* A **counted** quantity — money, shares, basis points, anything that must reconcile exactly against another system — is never a float. A **measured** one — a latency, a temperature, a ratio, a rate — may be, and usually should be. The question is about the quantity, not about the precision you think you need.

**What holds this rule today: a scan, and it holds one half.** A source check reads the *concept* — an identifier in a monetary role declared as `f32`/`f64` — across three shapes: a field or parameter, the tuple newtype this rule names by name, and the type alias that hides the representation. It runs on every push and in CI, it carries its own probe in both directions so a detector that has stopped detecting cannot report a clean tree, and it excuses a genuinely measured quantity by an explicit marker, because a check that goes red on the first honest false positive is a check that gets muted.

**What the scan cannot hold is the half this rule was mined from.** It reads Rust only; it reads a *name*, so a monetary field called `x` is invisible to it; it says nothing about division, rounding or currency mixing; and above all it cannot see a decision that has not been written down yet. The incident was a *greenfield* build, where the type layer is itself being authored: layers 1 to 3 of the hierarchy of controls have nothing to stand on, because there are no types yet to make the state unrepresentable and no tests that could cover types that do not exist. A scan is the interim and says so.

**The fix that closes the class is a starting point, not a check.** A scaffolding crate that already contains `Money<C>`, `Price` and `Qty` with an exact representation and no `Div`, so a financial project begins with the types present and the representation is *selected* rather than invented. That is the only move that works when the type layer is the thing being authored, and it is the one that turns this rule from prose plus a net into a compile error.

## No sentinel values: absent states are enum variants [R:no-sentinel-values]

If "absent / stopped / unknown" is a real state, make it an enum variant, not a magic value of an existing type. Downstream code will forget to special-case a sentinel; it cannot forget a variant the compiler forces it to handle. If a range check reads a sentinel as a real quantity, it fails in the direction of the sentinel, not of safety.

## Put the costed fork; never resolve a trade of the user's resources silently [R:no-silent-spend]

> Has recurred 1 time(s) since it was written; most recently 2026-09-16.

Time, money and thoroughness trade against each other on nearly every task: a longer
commit message, a fuller documentation pass, re-running a whole verification suite where
a fast subset would have served, ten sequential steps where one batched pass would do.
Each of those spends one of the user's resources to buy another, and only the user knows
the exchange rate on the day.

Resolving that trade silently is the error, and the error is **symmetrical**. Quietly
spending more to be thorough is exactly as wrong as quietly spending less to be quick;
both substitute your preference for a judgement that is theirs.

State the fork in one line at the moment it arises -- the two options, roughly what each
costs, and your recommendation -- then act on the answer. One sentence they can ignore,
not a survey and not a blocking question. A default you have already acted on and
reported afterwards is a receipt; a fork they can still turn is a decision.

Failure-mode check: **whose resource am I about to spend, and do they know the other
option existed?** If the answer to the second half is no, you did not make a judgement
call -- you made their call for them.

Two things this is NOT. It is **not** a licence to ask about everything: a trade inside
your own remit -- which library, which algorithm, how to split a module -- is yours to
settle, and escalating it is the same failure pointing the other way. And it is **not**
discharged by a documented default or a settings toggle. A switch someone must go and
change is a regime change, not a choice about *this* task; the lever has to be pullable
in the moment, or it is not a lever.

## Never push a stale copy over a fresher target [R:no-stale-push-over-fresh]

Any script that writes a mirror, template, or snapshot onto a live target must refuse when the target is newer than the source, and must say which side is behind. Direction is the property that has to be checked; sameness is not.

Showing a diff is not a guard. A diff reveals *what* differs and never *which side is authoritative*, and a human clicking through a prompt cannot see modification times, provenance, or which copy anyone has been editing. Approving a diff feels like a decision and supplies none of the information the decision needs.

Where a live authority exists -- a global configuration, a production dataset, a running deployment -- the template must lose unconditionally, and the force flag must not reach it. A flag that overrides a directional guard will be used, by the person who is most sure and least informed, on the day the guard was right.

Ask before writing such a push: *if my copy is the stale one, what does this destroy, and would anything tell me?* The characteristic damage is that the answer is nothing: the operation succeeds, reports success, and the loss is discovered later by someone looking for a change they know they made. This is the directional sibling of R:generate-guards-unversioned -- that rule guards against destroying content that exists nowhere else, this one against destroying the newer of two copies that both exist.

## Never route judgment work to a weak model, and never embed a sub-tier local LLM [R:no-weak-model-for-judgment]

Work that needs judgment goes to a capable model. Do not embed a sub-tier local LLM (a 7B/13B behind Ollama, llama.cpp or similar) in a tool as a convenient, API-key-free fallback: a meaningfully dumber model degrades the tool it is wedged into, everywhere and silently, and the output looks like ordinary tool output rather than like a downgrade. When a tool needs intelligence, delegate to the capable model through the existing subscription -- the MCP server is the abstraction boundary and clients are peers. Note that the cost argument usually offered for the local model is the price-tier fallacy R:measure-cost-per-task names; but this rule is not an economic one and does not dissolve if the sums come out favourably. Mechanical, tool-restricted passes are a different matter and may be scoped tightly; the floor applies to work where the answer is a judgement.

## One home per rule; every other copy is a reference or is generated [R:one-home-per-rule]

Every rule, fact or definition has exactly one home, and every other place that needs it holds a **reference or a generated copy**, never a second original.

**The test is not "are these files identical" but "if this changed, how many places would I have to edit".** More than one is a fork already, whether or not the copies have drifted yet. Drift is the symptom that makes a fork visible; it is never what makes it a fork.

**An index is a second home too, and it is the one that hides.** A list of what exists -- which rules, which files, which tests, which hooks -- read beside the thing that defines them is a duplicate of the most load-bearing fact in the system, and it fails silently in a way a duplicated body does not. Two copies of a rule's TEXT eventually contradict each other and someone notices. Two copies of the INDEX never contradict: the stale one is a strict subset, so every entry it holds is correct and the reader sees a coherent, smaller world. Ask of any list: *what would it look like if this were out of date?* If the answer is "exactly like this, only shorter", nothing can tell you which you are looking at.

**Prefer deriving to synchronising.** A generated copy is not a second home: it is an artefact with a stated source, regenerable, and safe to delete. A synchronised copy is a second home with a chore attached, and the chore is what stops being done. If a consumer needs the data, give it a way to ask rather than a copy to keep.

**Where a second original is genuinely unavoidable,** say so at both sites, name which one is authoritative, and give the copy a check that fails when it diverges. That is worse than one home and much better than two that both look right.

Failure-mode check, before adding any list, table, registry or inventory: *does something else already know this, and can I read it from there instead?*

`[R:doc-currency]` is the downstream half -- when the source moves, the descriptions of it move in the same change. This rule is upstream of that: it asks why a second description existed to go stale.

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

## Every dependency is priced in the decisions log, and so is every dependency refused [R:price-every-dependency]

Every dependency gets an entry in the decisions log: what it is for, why this one, and
what was refused. The refused list includes **writing it yourself**, with a rough size,
whenever that was a live option -- and it usually was. A crate arrives as one line in a
manifest: no design discussion, no diff worth reading, nothing that looks like a choice
was made. The entry is what makes it one.

A decision *not* to take a dependency gets an entry too, and that is the half which
disappears. Forty lines written by hand in place of a crate leave no manifest line, no
lockfile churn, nothing a later reader can trip over. The refusal exists only in the head
of whoever made it, so the next session proposes the same crate, nobody can tell it was
already priced and declined, and in it goes. A refused dependency and one nobody ever
considered look identical from the outside; only a deliberate entry tells them apart.

This is sharper with an assistant than without one. Reaching for a crate is what the
training data does: asked to parse a date, hash a string, or retry a request, an
assistant will propose a dependency before it proposes twenty lines, because published
code overwhelmingly takes the dependency. The refusal is therefore the option that has to
be written down on purpose, precisely because it is the option nothing else records.

Feature flags are part of the price. `default-features = false` belongs in the entry
together with its reason: what the default set drags in, and why that tree is not wanted.
A crate taken with its defaults off is a different dependency from the same crate taken
whole, and an entry that does not say which one was taken has not priced anything.

Dev-only is part of the price, and it lowers it. An entry that can say *nothing shipped
depends on this* is materially different from one that cannot: the blast radius is a
build machine rather than every user, and the bar the alternatives had to clear was lower
because of it. Say which it is.

The manifest comment points at the entry; it does not restate it. A one-line purpose
beside the dependency is where a reader actually meets it and is worth having -- what
this is for, and where the decision lives. Standalone reasoning in a manifest comment is
a second home for the rationale, and two homes drift: the comment is trimmed, the log is
appended to, and neither reader can tell which one is current.

Record the entry when the dependency is added, while the alternatives are still in mind.
Reconstructed later, the rejected list is always the flattering one -- the same
observation `[R:decisions-log-records-rejected-alternatives]` already makes, and the
reason this rule is that one's sibling rather than its replacement. That rule requires a
decision already recognised as a decision to carry its why and its refusals. This one
says a dependency is such a decision despite arriving as a manifest line, and that
choosing to take none is another.

The asymmetry is real and must not be papered over. A gate can parse every manifest and
fail when a dependency is not named in the log, so the half about dependencies **taken**
is mechanisable and cheap. Nothing mechanical can check the half about dependencies
**refused**: there is no artefact to compare the log against, which is exactly why that
half is the one that goes missing. There the human reader is the only detector, and a
gate covering the first half must never be read as covering the second.

## Reconcile declared against active wiring on a schedule the guard cannot break [R:reconcile-wiring-at-start]

The entry condition is **declared is not active**. It is not "the wiring is unversioned". A tracked control goes dark just as quietly as an untracked one: a commented-out entry, a moved path, a lost executable bit. Version control yields a diff, but a diff does not tell you a guard is inert *now*, and nobody diffs a file they have no reason to suspect. Being unversioned is an aggravating factor -- no diff, no revert, no attribution -- never the qualifier.

So check the declared-versus-active gap on a schedule that does not depend on the guard being alive, and report **by name** which control is inert. A count is not actionable and reads as noise; a name is a work item.

**Prove the alarm in the dark state.** An alarm never observed firing is not known to fire. Exercise it against a deliberately broken configuration -- delete the hook block in a temporary home directory and confirm the check names each drifted control. Without that drill the reconciliation becomes an unverified control one level up and the regress simply moves; the drill is what terminates it, which is R:wired-artifact applied to the thing doing the watching.

Ask: *if this control switched itself off, what would tell me, and when?* If the honest answer is "the next audit", the control is off for as long as audits are apart, and that interval is the real guarantee -- not the control.

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

## Review a change against the behaviour contract, never against the plan that produced it [R:review-against-contract-not-plan]

Review a change against the **behaviour contract**, never against the plan or the template that produced it.

Plan code, scaffold code and reference implementations are **unreviewed input**. A faithful transcription of a defective template is still a defect, and the transcription is the part review is worst at seeing: the diff matches its instructions exactly, so every local question a reviewer asks has a satisfying answer.

**Arm the reviewer with the contract, not the intent.** The question is *what does this system now promise, and does this change keep every one of those promises* -- not *does this match what we said we would build*. Those two questions diverge precisely where a plan is wrong, which is the only case where review had anything to catch.

**A plan cannot grade its own work.** If the same document supplies both the instruction and the standard of correctness, review reduces to checking transcription accuracy. Where a plan is the only artefact, that is worth saying out loud in the review rather than letting the approval imply more than it checked.

**The failure survives a per-item review and dies at the whole-change one.** Each task, judged against its own slice of the plan, is correct; the contract violation only becomes visible against the feature set entire. Where a change spans several tasks, one pass must read the whole of it against the whole contract.

Failure-mode check, before approving: *what did I compare this against, and could that thing itself be wrong?* If the answer is the plan, the specification or the ticket, the contract has not been consulted yet.

## After restructuring, verify references as a distinct pass [R:revision-integrity]

Editing a structured artefact silently breaks references that the previous version made true. The edit raises no error, and rereading does not catch it: the author restores the deleted context from memory and reads a coherent passage that is not on the page.

So run referential integrity as a **distinct pass**, after the restructuring and not during it. Every pronoun and comparative -- *this*, *neither*, *the former*, *the more important* -- must resolve within the current text. Every cross-reference must point where it claims. Every announced count must match what follows. Every term must be defined before it is used.

Two methods defeat author blindness where rereading cannot. Give the passage to a fresh reader instructed to report **comprehension failures only**, not content or style -- they have no deleted draft to autocomplete from. Or translate it into another language: if it cannot be rendered without adding words, the words are missing in the original.

Prefer structure that cannot carry the defect. A heading that states a count goes stale on the next addition, so write the heading without the count rather than remembering to update it -- R:prefer-by-construction applied to prose.

This is the prose sibling of R:wired-artifact: a locally correct change with a silent non-local effect. Ask, every time: *what did this edit quietly leave pointing at nothing?*

## Search your own work before specifying a component [R:search-before-you-build]

Before writing the requirement for any component, ask whether it already exists -- and
search **your own work first**.

This is a different question from "does this work?", and it fails at a different moment:
earlier, before the mechanism is evaluated at all, while its need is still merely
asserted. A design review that starts once the requirement is written has already
accepted the premise that something must be built.

The usual research step points outward -- public code search, vendor documentation,
package registries -- and that is the wrong end of the search space. For a tool built to
your own discipline, *you* are the most likely author of the thing you are about to
specify. Grep your own repositories first, and read the charter of anything adjacent,
including whatever is already open in this session.

Failure-mode check: **if this already existed, where would it be -- and have I looked
there?** If you cannot name the place you looked, you have not searched; you have
assumed.

Note what this is not. It is not an argument against building, and it is not satisfied by
a vague sense that something similar exists somewhere. It asks for a location and a
result: the path you grepped, the repository you read, and what was or was not there.

## Give a repeated signal a consequence, or stop emitting it [R:signal-needs-a-consequence]

> Has recurred 1 time(s) since it was written; most recently 2026-09-20.

Attach a consequence to a signal, or stop emitting it. A check that is right, whose verdict arrives whole, and that changes nothing is read once out of novelty and never again -- and the condition it reports then degrades quietly behind a line of text that says it is degrading.

**The consequence is what makes a signal a control.** Exit non-zero. Fail the build. Block the write. Refuse the push. Until one of those is attached, what exists is a description of a problem, and a ledger that counts it as coverage is overstating what the system holds.

**Attach it to something that already must happen.** A commit, a push, a build, a test run. A consequence bolted to a command someone has to remember to invoke is the same defect one level up, and it fails the same way: the gate that would have caught the incident behind this rule existed and was not run, because running it was also a choice. Ask which unavoidable event this fires on. If the answer names a person's intention rather than an event, nothing has been attached yet.

**Advisory is a legitimate design, and it is the expensive one.** Sometimes a signal must not block: the judgement is genuinely human, or the input comes from strangers and failing CI on it would hand them your build. Then the consequence is a *named reader on a schedule* -- whose job, how often, and what they do with it. "It is printed at startup" is not a reader; nobody is on the hook for a line of output. Choosing advisory means choosing to spend somebody's attention every time, forever, rather than deciding the escalation rule once. Say so when you choose it, and say who pays.

**Two shapes, one cause.** A signal that never fires is ignored because it is invisible; a signal that always fires is ignored because it is furniture. An always-red gate is a muted gate exactly as a never-red one is. What both lack is a state change that costs somebody something.

The test, at the moment you write the warning: *what breaks if nobody reads this?* If the answer is "nothing breaks, it just stays wrong", it is not a control and must not be counted as one.

Fourth position in an existing family, and the one where every link holds. `[R:guarantee-needs-a-reader]` -- nothing checks the claim. `[R:wired-artifact]` -- something checks it and accepts evidence anything could produce. `[R:verdict-survives-the-channel]` -- the check is right and its verdict is destroyed in transit. `[R:reconcile-wiring-at-start]` -- the control goes dark without announcing it. Here the check is correct, the wiring is live, the verdict is intact and delivered, and the chain still ends, because its last link was a person with nothing at stake.

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

## A check's verdict reaches the decision intact, or the check did not run [R:verdict-survives-the-channel]

> Partly enforced by hook:gate-verdict-intact (shell pipeline half) + hook:multiline-pattern-eol (shell edit half); write paths that are not shell commands -- an API or service call whose caller-side return value reports dispatch rather than effect; and a verdict read from a side channel that is structurally incapable of carrying a failure is held by this instruction alone.

> Has recurred 2 time(s) since it was written; most recently 2026-09-20.

Read a check's own verdict, never a status that merely travelled beside it.

A gate can be entirely correct and still certify a failure, because the verdict is lost
between the check and the decision. `gate | tail` exits with the status of `tail`. An
`&&` placed after such a pipeline tests the filter rather than the gate, so the action it
was meant to guard runs regardless. And a filter narrow enough to be readable will crop
the verdict out of the very output it was meant to summarise: the header carrying the
word FAILED scrolls away, and what remains is a few detail lines that read like ordinary
progress.

Nothing is forged here and nothing is unenforced -- the check ran, it was right, and its
answer never arrived. That is what separates this from its two neighbours:
`R:guarantee-needs-a-reader` fires when no check exists, `R:wired-artifact` when one
exists and accepts the wrong evidence, and this one when a correct check's answer does
not survive the trip to the reader.

The same failure runs in the other direction for commands that change things. A
substitution whose pattern matches nothing, an in-place edit against an absent string, a
rename with no candidates: every one of them exits zero. An exit code reports that the
program ran, not that the work happened.

So: run a gate unfiltered and let its own status stand. Where the output must be
filtered, make the pipeline report the gate -- enable pipefail, or read the first stage's
status explicitly -- and never place a filter between a gate and an `&&`. After a command
whose purpose is to change a file, verify the change by reading the file back, not the
status the command returned.

Failure-mode check: **what does this exit code actually measure?** If the answer names
the last stage of a pipeline, or names "the program ran" rather than "the work was done",
there is no evidence yet.

## Verify through the production path [R:verify-through-production-path]

> Has recurred 1 time(s) since it was written; most recently 2026-09-04.

Before declaring anything verified, run at least one check through the exact channel production uses: same env var, same startup script, same config file, same transport. A test that exercises a stand-in is evidence the stand-in works, not that the feature does. Ask: which line of production wiring did my test NOT execute? That line is where it breaks.

## A success check consumes a sentinel nothing else can produce [R:wired-artifact]

A success check must consume a write-once sentinel that is unique to the artefact class it verifies, emitted as the final act of the success path, and producible by nothing else in the system. Pattern-matching on a date, a header, a log line, or the presence of a file is not verification -- it is a check that something happened, which is a different claim from the one being made.

Ask before wiring any check: *what else in this system can produce the string my check accepts?* If the answer is anything at all, the check accepts forgery, and it will accept it silently on the day it matters -- because the failure path is the one that regenerates headers and re-emits dates.

The same test applies to enforcement claimed on a component: *what produces this, what consumes it, and does the cited artefact cross that seam?* If nothing crosses it, the feature is inert and the honest ledger entry says so rather than naming the component's own tests.

A green check that cannot fail is worse than no check. It converts an open question into a settled one, so nobody looks again, and the thing it was protecting degrades behind a signal that says it is fine. This is the type-level and tooling-level sibling of R:verify-through-production-path, and it shares a family with R:guarantee-needs-a-reader: that rule fires when nothing enforces the claim, this one when something does and accepts the wrong evidence.

<!-- relearn:generated v0.1.0 sha256=ca8ddd1e0e03668731ff65bc19a86eda0b00357d7a5d117314f608d85664f693 rules=R:case-collision,R:check-the-claim-you-inherit,R:claude-md-recreates-the-project,R:decisions-log-records-rejected-alternatives,R:definition-of-done-every-change,R:delegate-a-fan-out,R:detector-excludes-own-definitions,R:doc-currency,R:features-ledger-names-its-artefact,R:five-files-no-more,R:guarantee-needs-a-reader,R:make-illegal-states-unrepresentable,R:measure-cost-per-task,R:measure-the-claim-not-a-subset,R:money-is-not-a-float,R:no-sentinel-values,R:no-silent-spend,R:no-stale-push-over-fresh,R:no-weak-model-for-judgment,R:one-home-per-rule,R:pin-eol-for-executable-text,R:prefer-by-construction,R:price-every-dependency,R:reconcile-wiring-at-start,R:repair-the-lying-artefact,R:review-against-contract-not-plan,R:revision-integrity,R:search-before-you-build,R:signal-needs-a-consequence,R:source-practice-from-its-artefact,R:verdict-survives-the-channel,R:verify-through-production-path,R:wired-artifact -- DO NOT EDIT; regenerate with `relearn build` -->
