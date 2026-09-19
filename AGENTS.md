# Agent instructions

## Check for a case-differing sibling before creating a file [R:case-collision]

> Also enforced by hook:no-case-collision.

Before creating any file, check for an existing sibling whose name differs only by case or by Unicode normalisation. Where a repository is synced between a case-sensitive filesystem and a case-insensitive one, the case-insensitive semantics are the binding constraint: two such names are one file, and the loser is whichever was written second.

Merge into the existing file, with a section heading, rather than creating the colliding name. The instinct to create a properly-capitalised new file is the failure -- the convention is not worth a silent overwrite on the other machine.

The damage is invisible from where the work was done. On the authoring filesystem both files exist and everything looks correct; the collision appears only after a sync, as content that vanished with no diff and no error to attribute it to.

This rule has graduated: a write-time hook now blocks the collision deterministically, so the instruction layer no longer has to carry it for that path. It still applies to every creation path the hook does not see -- a shell redirect, a git operation, another tool -- which is why the guidance is kept rather than retired.

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

> Has recurred 2 time(s) since it was written; most recently 2026-09-19.

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

## Quoting an incident carries its names past the gate that was holding them [R:names-travel-with-the-quote]

Before an incident, a log line, a trace or a path leaves the repository that holds it,
scan the destination with the source's own detector -- not the destination's.

A detector for private identifiers is scoped to a tree. It is a list of names somebody
enumerated, matched against the files of one repository, and it is correct exactly there.
Every quotation moves content across that boundary and none of it moves the check: the
paper, the public rule corpus, the issue comment and the conference slide each inherit
the source's *names* and none of its *gates*. The source stays green, because nothing
about it changed.

The trap is that writing the incident down is the discipline working. A rule with no
provenance cannot be audited, so the incident narrative is mandatory -- and a narrative
is verbatim by nature, because the specifics are what make it interpretable. The very
field that makes a correction durable is the one that carries the name out.

So the check belongs at the boundary the content crosses, which is the publication, not
the repository. Where the destination is public, the term list usually cannot be
committed alongside it: a salted digest of a short name is a few million candidates and
publishing the list discloses what it detects. Keep the matcher in the public artefact
and the list outside it, and make the absence of the list an ERROR rather than a pass, or
the gate arrives disarmed and reports the same green as a clean tree
`[R:guarantee-needs-a-reader]`.

Failure-mode check, before anything private is quoted anywhere: **which repository's
detector covers the file I am about to write into?** If the answer is the repository the
quotation came FROM, nothing covers the destination.

This is the sibling of `[R:report-the-hit-not-the-match]`, which governs the moment a
search prints what it found. That one is about output; this one is about content coming
to rest in a second artefact, where it is committed, pushed, indexed and mirrored. Same
identifier, different surface, and the fixes do not substitute for one another.

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

## Report the hit, never the match [R:report-the-hit-not-the-match]

> Also enforced by hook:banned-name-in-output.

When what you are searching for is a thing whose whole problem is that it exists, your
search output is another copy of it. Report **location, count and length**; never the
matched text, and never a field that can contain it.

Run the check before the output leaves your hands: *for every field I am about to print
-- path, parent directory, filename, context line, error message, commit summary -- can
the thing I am hiding be inside it?* If you cannot answer for one of them, drop that
field. A path whose last component IS the name defeats a redaction that prints the
parent, and `find | xargs` prints the path whole.

**This binds ad-hoc work exactly as it binds a committed detector, and that is the half
that fails.** A one-off shell pipeline, a `grep -o`, a loop written to answer one
question, and the sentence you type afterwards are all publication surfaces -- and so is
the transcript. A carefully built scanner in the same repository does not cover you; it
covers its own output.

Sibling, deliberately not merged: `[R:detector-excludes-own-definitions]` is the
FALSE-POSITIVE half of self-reference -- a check that matches its own text is always red,
gets muted, and leaves the system looking guarded. This is the DISCLOSURE half. Same
shape, opposite failure, different fixes: strip comments there, never emit the match
here. `[R:names-travel-with-the-quote]` is the third face of the same identifier -- not
printing it, but writing it down somewhere with a wider audience.

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

> Also enforced by hook:gate-verdict-intact (pipeline half) + hook:multiline-pattern-eol (edit half).

> Has recurred 1 time(s) since it was written; most recently 2026-09-06.

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

## A path optimised on yesterday's inputs deoptimises when today's arrive [R:a-speculated-path-deoptimises-when-the-input-changes]

> Written down from Aleksey Shipilev, "JVM Anatomy Quarks", on profile pollution, inlining and deoptimisation; HotSpot's uncommon-trap mechanism.

A JIT does not compile your code. It compiles a bet about your code, and the bet is settled at the
moment the bet stops being true.

HotSpot compiles on profile: this call site has only ever seen one receiver type, so inline it and
guard; this branch has never been taken, so do not emit it; this field has never been null, so
skip the check; this class has never been loaded, so assume no subclass exists. Each assumption
gets a guard, and the compiled path is fast precisely because the guards are cheap and the bodies
are absent. When a guard fails the frame is deoptimised: an uncommon trap, interpreter frames
rebuilt from the compiled ones, execution continuing interpreted, and a wait to be recompiled with
the new profile.

The timing is what makes this a latency rule rather than a throughput one. The assumptions break
when the input changes -- the unusual order type, the first exception of the day, a new venue's
message format, the second implementation of an interface loaded at hour six. So the worst latency
the path will ever produce is delivered on the least ordinary event, which is generally the event
that mattered most. Load testing with representative-on-average traffic reproduces none of it.

It is also silent by default. Nothing is logged, no metric moves, and the only trace is a latency
outlier indistinguishable from a collection pause or a scheduling delay. `-XX:+PrintCompilation`
and `-XX:+TraceDeoptimization` say what happened; JFR carries the events with less overhead and is
the one worth leaving on.

So warm the path with the **unusual** cases as well as the common ones -- which is where this
meets `[R:a-measurement-matches-the-regime-it-reports]`: a warm-up built only from typical traffic
produces a profile that typical traffic confirms and atypical traffic destroys. Keep hot call
sites deliberately monomorphic; a megamorphic interface call on the hot path is a permanent
inlining barrier rather than a one-off trap. Load the classes the path will need before the path
is live. And treat a lambda or a dynamic call site added to a hot path as a new profile that has
to be earned again.

Failure-mode check: **what has the JIT assumed about this path, and what happens the first time
one of those assumptions is false?** If the honest answer is that nobody knows what was assumed,
that is the finding -- the assumptions are readable, and reading them is the work.

## A collection view is a window onto someone else's data [R:a-view-is-not-a-copy]

Know whether you are holding data or a window onto data. The static type is `List<T>`
either way and will never tell you.

The library returns views far more often than most code assumes, and each has its own way
of surprising:

* **`Arrays.asList(a)`** is a fixed-size wrapper *over the array*. `set` writes through to
  the array; `add` and `remove` throw `UnsupportedOperationException`. `List.of(...)` is
  immutable and independent, and is usually what was meant.
* **`list.subList(from, to)`** is a live window. Writes go through, and any structural
  change to the parent makes the sublist throw `ConcurrentModificationException` on its
  next use — including the common idiom of taking a sublist and then clearing the parent.
* **`map.keySet()`, `values()`, `entrySet()`** are views. `keySet().remove(k)` removes the
  entry from the map. That is occasionally what you want and always worth being deliberate
  about.
* **`Collections.unmodifiableList(l)`** wraps without copying: the wrapper refuses writes
  and the original still works, so it defends against the caller and not against you. It
  is not a defensive copy, which is why `[R:no-reference-to-internals-escapes]` reaches for
  `List.copyOf` instead.
* **`Stream.toList()`** is unmodifiable; **`collect(Collectors.toList())`** gives no such
  guarantee and may or may not be. Do not rely on either being mutable.

So state which you want at the point you create it. `List.copyOf` when the receiver must be
independent, the view when sharing is the intent and the lifetime is short. A view stored
in a field is the shape to look at hardest: it outlives the expression that made it, and
the backing collection can change under it at any point afterwards.

The same distinction governs iteration. Modifying a collection while iterating it throws
`ConcurrentModificationException` even single-threaded, because the iterator is a view too
— use `removeIf`, or iterate a copy, or collect the removals and apply them after.

Failure-mode check, for any collection received or returned: **if somebody writes to this,
what else changes?** If you cannot answer without opening the factory method's Javadoc,
copy it.

## A wrapper type costs an object until escape analysis removes it [R:a-wrapper-type-is-not-free-here]

Give domain concepts their own types here too — an `OrderId` that cannot be passed where
an `AccountId` belongs is worth as much in Java as anywhere. But do not carry over the
*cost* argument with the *design* argument, because only one of them travels.

In Rust a newtype is a compile-time construct: `struct OrderId(u64)` is a `u64` at run
time and the compiler proves it. In Java, `record OrderId(long value)` is an **object** —
a header, a field, a reference to chase, and a separate cache line from whatever pointed
at it. The JIT can remove it: escape analysis plus scalar replacement will flatten a
wrapper that provably does not escape its compilation unit, which covers a great deal of
ordinary code. What it does not cover is the case you care about — a wrapper stored in a
field, put in a collection, handed across a method the JIT declined to inline, or in an
array, where `OrderId[]` is an array of pointers and `long[]` is an array of numbers.

So the discipline splits by where the value lives:

* **At an API boundary, and anywhere correctness is the concern**, use the wrapper. The
  swapped-argument bug it prevents is worth an allocation the JIT will usually remove
  anyway.
* **Inside a hot loop, in a large array, or in a per-message structure**, measure before
  assuming the wrapper vanished. This is `[R:verify-the-abstraction-compiled-away]`
  arriving in a language where the optimiser's decision is dynamic: `-XX:+PrintInlining`,
  a JFR allocation profile, or an escape-analysis dump says what actually happened, and
  the answer can differ between runs of the same binary.
* **Where it does not vanish and the budget is real**, keep the primitive and put the
  safety somewhere that costs nothing — a named parameter, a builder, a static factory
  whose signature cannot be called wrongly.

The version of this rule that is coming is worth knowing about rather than waiting for:
Project Valhalla's value classes are designed to make exactly this wrapper flatten by
specification instead of by the optimiser's discretion. Until the code runs on a JVM where
that is true, the cost is dynamic and must be measured rather than assumed.

Failure-mode check, before wrapping a primitive on a path with a budget: **does this
object still exist after the JIT has finished, and what told me?** "It is just a wrapper"
is the Rust answer, and it is the one that does not transfer.

## A resource is closed by the construct that opened it, on every path [R:close-what-you-open]

Open a resource in a `try`-with-resources header and nowhere else. The construct closes on
every exit, in reverse order, and — the part a hand-written `finally` cannot do — it keeps
the original exception when the close also fails.

```java
try (var conn = pool.take(); var stmt = conn.prepare(sql)) {
    return stmt.execute();
}
```

The failure that a `finally` block introduces is worth stating plainly because it looks
correct. If the body throws and `close()` also throws, the `finally`'s exception replaces
the body's: the program reports a failure to close a connection and says nothing about the
query that failed first. Try-with-resources instead **suppresses** the close failure and
attaches it to the original, so both survive and the cause is the one you wanted. A leak
costs you a handle; a lost cause costs you the investigation.

Three cases where the construct does not apply and the discipline still does:

* **A resource whose lifetime is a field**, not a block — a pool, a client, an executor.
  The owning object becomes `AutoCloseable` and the discipline moves up a level; the thing
  to refuse is a field that is opened and never closed by anyone in particular.
* **An `ExecutorService`** is not closed by `shutdown()` alone. `shutdown()` then
  `awaitTermination()` then `shutdownNow()` is the sequence, and skipping it leaves
  non-daemon threads holding the JVM up.
* **A `Stream` over a file** (`Files.lines`, `Files.walk`) holds a handle and must be
  closed. It is the one stream that leaks, and it looks exactly like the ones that do not.

Do not reach for a finalizer or `Cleaner` as the primary mechanism. They run at an
unspecified time or never, and they exist to catch the case where the discipline already
failed.

Failure-mode check, for every resource: **which construct closes this if the next line
throws?** If the answer is a `close()` call further down the method, nothing does.

## A class is final, or its inheritance is designed and documented [R:design-for-inheritance-or-forbid-it]

Make the class `final` unless you have designed for a subclass, and if you have, say
exactly what a subclass may override and what it must not.

Inheritance is not the reuse mechanism it looks like. A subclass depends on which of the
superclass's methods call which others — `addAll` calling `add` is the canonical example
— and that is an implementation detail the superclass is entitled to change in a patch
release. So a working subclass breaks when the parent is refactored, without either author
doing anything wrong. Composition does not have this property: a wrapper depends on the
public contract and nothing else.

The constructor case is worse than fragile, it is broken from the start. A constructor
that calls an overridable method runs the subclass's override **before the subclass's own
fields are assigned**, so the override sees `null` and `0` in fields it declared `final`
and initialised. Nothing warns. This is the same publication hazard
`[R:no-reference-to-internals-escapes]` names as `this` escaping, reached by a route that
looks like ordinary method dispatch — and `[R:publish-safely-or-not-at-all]` is why it
matters even when there is only one thread.

So:

* **`final` by default** on classes, and on any method a subclass has no business
  replacing. Sealing (`[R:seal-the-alternatives]`) is the stronger form where the set of
  subtypes is closed and known.
* **Never call an overridable method from a constructor**, an initialiser, or `clone()` or
  `readObject()`, which are constructors wearing other names.
* **If a class is designed for inheritance**, document the self-use: which methods call
  which, which are safe to override, what a subclass must call. That documentation is part
  of the contract, and the cost of it is the honest reason most classes should be `final`
  instead.
* **Prefer composition.** A wrapper that holds the instance and forwards is longer to
  write, immune to the parent's internals, and the thing to reach for when the motivation
  was reuse rather than substitutability.

Failure-mode check, for every non-final class: **which of my methods call each other, and
would a subclass overriding one of them still be correct after I reorder them?** If that
cannot be answered, the class was extensible by accident.

## equals and hashCode are one decision, and a mutable key breaks both [R:equality-is-one-contract]

Decide equality once, implement both halves in the same change, and derive them from
fields that cannot change while the object is reachable from a collection.

The contract is mechanical: equal objects must have equal hash codes. Break it and a
`HashMap` looks in the wrong bucket, so `get` returns null for a key it contains and
`remove` cannot find an entry to remove -- a leak whose size grows with traffic. Nothing
detects this. The collection is behaving exactly as specified; the specification was
handed an object that lied.

The mutable-key half is the one that survives review, because the class is correct when
it is written and becomes wrong when someone adds a setter. An object whose `hashCode`
depends on a field that is later mutated has moved bucket without the map being told, and
is now unreachable through the very key it is filed under. **So a type used as a key is
immutable, or it is not used as a key.** A `record` gives you both halves generated from
the components and final fields by construction, which is why it is the default shape for
a value type and why hand-writing these two methods should feel like a decision rather
than boilerplate.

Two more that follow from the same contract and are worth stating because each has its
own way of going wrong:

* **`compareTo` must agree with `equals`** wherever the type reaches a `TreeMap` or a
  sorted set, which use ordering and never call `equals` at all. A comparator saying two
  objects are equivalent while `equals` says they differ produces a set that silently
  holds one of them.
* **Inheritance and equality do not compose.** An `equals` written with `instanceof`
  accepts a subclass and is then asymmetric; one written with `getClass()` is symmetric
  and rejects every subclass. There is no third option, which is one of the reasons
  `[R:design-for-inheritance-or-forbid-it]` prefers `final`.

Failure-mode check, before a type is used as a key or put in a set: **which fields does
its equality read, and can any of them change while it is in there?**

This is `[R:make-illegal-states-unrepresentable]` where the language will not help: Java
cannot stop you writing an inconsistent contract, so the remedy is to generate it
(`record`) rather than to write it carefully.

## Catch what you can answer; never catch Exception [R:exceptions-name-what-failed]

A `catch` clause is a claim that you can do something about what you caught. Catch the
type you can answer and let the rest go up.

`catch (Exception e)` claims you can answer everything, which is never true: it absorbs
the `IllegalStateException` from a bug three frames down and the
`InterruptedException` that was asking the thread to stop, and it treats both the same as
the `IOException` the author was thinking about. The narrower the caught type, the more
the code says. Multi-catch (`catch (IOException | TimeoutException e)`) is how you handle
two without widening to their common supertype.

Three refusals that carry most of the value:

* **Never swallow.** An empty catch block, or one that logs and continues, converts a
  failure into a wrong answer computed quickly. If there is genuinely nothing to do,
  rethrow; if the method cannot throw, that is the design problem to fix rather than to
  hide.
* **Never lose the cause.** `throw new ServiceException("failed")` discards the stack that
  says why. Wrapping is fine and often right — `throw new ServiceException("loading " + id, e)`
  — as long as the cause travels. A chain printed once at the edge is worth more than
  every layer restating what the layer beneath it already said.
* **Never catch `InterruptedException` without restoring the flag.** Either propagate it or
  `Thread.currentThread().interrupt()`; absorbing it silently is how a shutdown request
  disappears and a thread pool refuses to stop.

On checked exceptions, the choice is about the caller, not about taste: a condition the
caller can plausibly recover from is checked, and a programming error — a broken
precondition, an impossible state — is unchecked. The failure mode to avoid is declaring
`throws Exception`, which is checked in form and unanswerable in practice, and the one to
avoid next is making everything unchecked so the signature stops mentioning failure at
all. That is the Rust library's ban on erasing a typed error contract, arriving in a
language where the erasure is free and needs no crate. (Named in prose rather than by
tag: that rule has graduated to a hook, and citing a retired rule is a `lint` finding —
the same precedent `[R:doc-currency]` set.)

Exception types are types. One per condition a caller could treat differently, each
carrying the values that identify the instance — the id, the path, the elapsed time — so
one log line is enough to act on rather than the start of an investigation. A message
built by string concatenation with no fields is prose wearing a class name.

Failure-mode check, at every `catch`: **what will this clause do with a failure I have not
thought of?** If the answer is "the same thing", the caught type is too wide.

## Comparing boxed numbers with == works until the value exceeds 127 [R:identity-is-not-equality-for-boxes]

`==` on a reference asks whether these are the same object. On a boxed number or a string
that is almost never the question, and the language will not stop you asking it.

What makes this a rule rather than a known gotcha is *where* it is wrong. `Integer` caches
boxes for −128 to 127, so `==` returns the right answer for every small value and the
wrong one above it:

```java
Integer a = 127, b = 127;   a == b   // true
Integer a = 128, b = 128;   a == b   // false
```

A unit test written with an id of `1` or a quantity of `10` passes forever. Production
arrives with an order id of 5000 and the comparison silently starts returning false — and
it returns false *correctly*, in the sense that the two really are different objects, so
there is nothing to find in a debugger except two values that look identical.

String literals behave the same way for the same reason: literals are interned, so `==`
works for them and fails for any string that was built, read from a socket, parsed, or
concatenated at run time.

So:

* **Use `equals`** — or `Objects.equals(a, b)`, which also handles a null on either side
  and is the right default at a boundary where either may be absent
  (`[R:null-is-not-a-value]`).
* **Use primitives where the value cannot be absent.** `int` rather than `Integer` removes
  the question entirely, along with the allocation and the possible
  `NullPointerException` on unboxing. A boxed type in a field or a signature should be
  there because absence is meaningful, not by default.
* **Compare enums with `==`** — that one is correct, deliberate, and null-safe, which is
  part of why enums are the right shape for a closed set
  (`[R:seal-the-alternatives]`).
* **Never unbox in a comparison chain.** `Integer x = null; if (x == 1)` throws; the
  unboxing is invisible in the source.

Failure-mode check, at every `==` between two references: **is this asking about identity?**
If the answer is that it is asking whether the values are the same, it is the wrong
operator and it will pass the tests.

## A class that hands out its own mutable state has no invariant [R:no-reference-to-internals-escapes]

`private` is about who may *name* the field, not about who may change what it points at.
A getter returning the field hands out the field.

```java
private final List<Leg> legs;

public List<Leg> legs() { return legs; }               // callers can add to your order
public List<Leg> legs() { return List.copyOf(legs); }  // they cannot
```

Same access modifier, same `final`, opposite guarantees. `final` on a reference field
freezes the reference and says nothing about the object at the end of it, which is the
distinction that makes this worth a rule: a class can be entirely `private final` and
entirely mutable from outside.

Copy on the way in as well as on the way out. A constructor that stores the caller's list
has given the caller a handle to its own state, so the object can be mutated after
construction by code that has nothing to do with it — and the defect surfaces far from
both. `List.copyOf`, `Map.copyOf` and `Set.copyOf` do the copy and return something
unmodifiable in one step; `Collections.unmodifiableList` wraps without copying, so the
original still works as a back door and the wrapper is a view rather than a defence
(`[R:a-view-is-not-a-copy]`).

Two Java-specific escapes that are easy to miss:

* **A record's components are shallow.** A record with a `List` component is not immutable;
  it has a final reference to a mutable list, and the generated accessor returns it. Copy in
  the compact constructor, or the record is a value type in name only.
* **`this` escaping during construction.** Registering a listener, starting a thread, or
  calling an overridable method from a constructor publishes a half-built object — the
  fields assigned after that point are not yet visible, and a subclass's override runs
  before its own fields are initialised. The object exists before it is finished; do not let
  anyone else learn about it until it is.

Failure-mode check, for every method returning a reference and every constructor taking
one: **can the caller change what this points at, and would the class survive it?**

`[R:private-fields-only]` is the same argument in Rust and the two are deliberately
separate: satisfying that one is nearly automatic there and is only the first half here.

## Never return null to mean absent [R:null-is-not-a-value]

A method that can return nothing says so in its type. `null` does not say it; it only
does it.

The contract is the whole problem. `Order find(String id)` and `Order get(String id)` have
the same signature, and one of them returns `null` on a miss while the other throws --
the caller cannot tell which without reading the body, so it either checks everywhere or
checks nowhere, and both are wrong. `Optional<Order>` states the answer in the type: the
caller cannot reach the value without deciding what absence means.

Three shapes, and the right answer differs:

* **A method that may legitimately find nothing** returns `Optional<T>`. Use it at the
  return position and nowhere else -- an `Optional` field costs a second allocation and a
  second dereference on every read, and an `Optional` parameter forces every caller to wrap,
  which is three states (`null`, `empty`, `present`) where the signature promised two.
* **A method returning a collection** returns an empty one, never `null`. `Collections.emptyList()`
  allocates nothing and removes the check entirely; a null collection makes the caller write
  a guard before a loop that would have run zero times by itself.
* **A method that cannot meaningfully continue** throws. Absence and failure are different
  answers and must not share a return value.

Accepting `null` is a separate decision from returning it. At an API boundary, reject it
loudly -- `Objects.requireNonNull(x, "x")` in the constructor -- so the failure lands at
the boundary that was handed the bad value rather than at the first dereference three
layers in. That is `[R:parse-dont-validate]` in a language whose type system will not
carry the witness: the check happens once, at the perimeter, and the field is trusted
afterwards because nothing else can write it (`[R:private-fields-only]`).

Where the codebase tolerates nullable references, annotate them and turn the analysis on.
`@Nullable`/`@NonNull` with a checker in the build is the difference between a convention
and a control; unenforced annotations are comments that look like types
(`[R:guarantee-needs-a-reader]`).

Failure-mode check, for any reference-returning method: **what does the caller do on a
miss, and what in the signature told them?** If the answer is a Javadoc line, nothing told
them.

This is `[R:no-sentinel-values]` in the language where the sentinel is built into every
reference type, which is why it needs its own rule rather than an instance of that one:
you cannot stop `null` existing, only stop it meaning something.

## An object handed to another thread is published safely, or it arrives half-built [R:publish-safely-or-not-at-all]

Handing a reference to another thread is not the same as handing it the object. Name the
edge that makes the fields visible, or the reader may see the reference before the
contents.

The failure is specific and not intuitive: a thread can observe a **non-null** reference to
an object whose final assembly it cannot yet see, because the writes that filled the fields
and the write that published the pointer are not ordered with respect to each other. The
reader gets a real object with zeros and nulls in it. Nothing throws at the moment of
publication; the NullPointerException happens later, in code that did check for null, on a
field that was definitely assigned.

The mechanisms that establish the edge, cheapest first:

* **`final` fields.** A field assigned in the constructor and never after is guaranteed
  visible to any thread that sees the reference, without synchronisation — provided `this`
  did not escape during construction, which is exactly what
  `[R:no-reference-to-internals-escapes]` forbids and is why the two rules are not
  separable in practice. An immutable object with all-final fields is safe to hand to
  anyone, and this is the reason to prefer one.
* **`volatile`**, for the mutable case: a write to a volatile field happens-before every
  subsequent read of it, which publishes everything written before it.
* **A lock**, an `AtomicReference`, a `ConcurrentHashMap`, or a `BlockingQueue` — each
  carries the edge as part of its contract. This is why handing an object through a proper
  queue is safe and handing it through a plain field is not.
* **`VarHandle`** where the ordering must be stated precisely rather than taken at
  `volatile`'s strength.

What does **not** establish it: a `synchronized` block the reader does not also enter, a
`HashMap` shared between threads, a field that merely happens to be assigned before the
thread starts but is not final, and "it works every time we run it" — which is
`[R:verify-ordering-on-the-weakest-target]` arriving in Java, since x86 hides most of this
and aarch64 does not.

Failure-mode check, for every object crossing a thread boundary: **what happens-before
edge makes its fields visible, and can I name it?** If the answer is that the reference is
assigned before the reader starts, that is an argument about time, and the JMM makes no
promises about time.

## A closed set of alternatives is a sealed hierarchy, matched exhaustively [R:seal-the-alternatives]

When the set of cases is fixed, say so in the type and let the compiler find every place
that must decide.

```java
public sealed interface Order permits Limit, Market, Stop {}

// no default: adding a permitted type stops this compiling
String describe(Order o) {
    return switch (o) {
        case Limit l  -> "limit at " + l.price();
        case Market m -> "market for " + m.quantity();
        case Stop s   -> "stop at " + s.trigger();
    };
}
```

The value is entirely in what it refuses. An open interface can gain an implementation in
another module, so no switch over it can ever be complete and every one needs a `default`
— and a `default` is a decision taken in advance on behalf of a case that did not exist
yet, which is the definition of a decision nobody made. Sealing closes the set, and an
exhaustive `switch` over a sealed type is then checked: add a permitted subtype and every
incomplete switch **fails to compile**, which is the report you want and the one no test
gives you.

So the shape to reach for is a sealed interface whose permitted types are records: the
cases are closed, each carries exactly its own data, and none carries a field belonging to
another. The shape to stop writing is the one it replaces — a class with a `type` field
and eight nullable columns, seven of which are null for any given instance, where the
combinations that cannot occur are representable and guarded by convention.

Two disciplines follow:

* **Never write `default` over a sealed type**, and do not add one to silence a warning.
  It converts a compile error into a runtime branch, which is the whole guarantee,
  discarded for a line. Where a genuine catch-all is needed, name the remaining cases.
* **An `enum` is the degenerate case and gets the same treatment.** Java checks
  exhaustiveness on a switch expression over an enum with no default, so the same
  compile-time report is available and a `default` throws it away.

Failure-mode check, for any type with a set of alternatives: **what happens when somebody
adds a case tomorrow, and where does the compiler tell them what to update?** If nothing
tells them, the set was not closed.

`[R:make-illegal-states-unrepresentable]` is the general form and this is how Java spells
it; `[R:seal-closed-trait-sets]` is the same decision in Rust, where the mechanism is a
private supertrait rather than a `permits` clause.

## Implementing Serializable adds a constructor that checks nothing [R:serializable-is-a-second-constructor]

`implements Serializable` is not a marker. It is a second, invisible, public constructor
that takes a byte array and performs no validation.

Deserialization does not call your constructor. It allocates the object and writes the
fields directly from the stream, so every check the constructor makes — the non-null, the
range, the "these two fields must agree" — is skipped, and an attacker or a corrupted file
produces an instance the class's own author believes cannot exist. That is
`[R:private-fields-only]` and `[R:parse-dont-validate]` defeated by a language feature
rather than by anyone's code: the perimeter was built, and this walks around it.

Two consequences beyond the invariant:

* **The field layout becomes a published API.** Once instances are serialized anywhere
  durable, renaming or removing a private field is a compatibility break. A `serialVersionUID`
  controls only whether the break is detected, not whether it happened.
* **Deserializing untrusted data is remote code execution**, not a theoretical risk. The
  stream chooses which classes to instantiate, and a gadget chain assembled from whatever
  is on the classpath does the rest. The JDK's own filtering
  (`ObjectInputFilter`) exists because the mechanism cannot be made safe by being careful
  with it.

So the default is: **do not implement it.** Where an object must cross a process boundary
or reach a disk, use an explicit format with an explicit parser — JSON, a schema, a binary
codec you wrote — so the reconstruction runs through a constructor and the perimeter
holds. The serialization format is then a decision with a version, rather than a shadow of
the class's private fields.

Where it cannot be avoided, the obligations are real and none of them is optional: a
`readObject` that validates exactly what the constructor validates, `readResolve` for a
type that must be a singleton — an enum is `Serializable` correctly and for free, which is
one more reason to prefer one — and a declared `serialVersionUID` so the break is at least
visible. Treat every one of those as evidence that the type should not have been
serializable.

Failure-mode check, before adding the interface: **what does this class's constructor check,
and am I content for a byte stream to skip it?**

## A measurement states whether it ran warm or cold, and matches production [R:a-measurement-matches-the-regime-it-reports]

> Written for the rust and java audiences.

> Written down from JMH and its samples (Aleksey Shipilev), whose existence is the standing claim that a hand-rolled microbenchmark on the JVM is wrong by default.

State which regime the measurement ran in, and check that it is the regime production runs in.
Two errors live here and only one of them has a reputation.

**Measuring cold when production is warm.** A first execution runs interpreted or from cold
instruction cache, with an untrained branch predictor, unpopulated page tables and, on a managed
runtime, no compiled code at all. The figure can be one to two orders of magnitude worse than the
steady state, and it describes the warm-up rather than the work. On the JVM the escalation is
staged -- interpreter, then C1, then C2, with on-stack replacement for long-running loops -- and
takes thousands of iterations, which is why a hand-rolled timing loop is wrong by default and why
a harness exists.

**Measuring warm when production is cold**, which is the half that gets no attention because it
looks like rigour. A path that runs once a minute is never warm. A market-open burst arrives into
cold code. An error handler runs for the first time that day at the worst possible moment. For
every one of those, the warm figure is a number no user will ever see, and the warm-up that was
carefully discarded *is* the production behaviour. Discarding it is not hygiene there; it is
deleting the measurement.

So the rule is a scope match rather than a procedure: say which regime was measured, say which
regime production is in, and show they are the same. Where production is genuinely both -- a
steady stream with cold bursts -- that is two measurements and two reported numbers, not one
averaged into meaninglessness.

This is `[R:measure-the-claim-not-a-subset]` applied to time rather than to a set: the warm-up
samples are a subset, discarding them is a scope decision, and a scope decision taken by reflex
is the one that manufactures a wrong number with the authority of having been measured.

Failure-mode check: **is the production path warm or cold when it matters, and which did I
measure?** If the benchmark discarded the first N iterations, name what production has that
corresponds to those iterations. If nothing does, the discarded part was the answer.

Two neighbours, deliberately separate. `[R:no-coordinated-omission]` is the instrument failing to
take samples during the events that matter; this is the instrument taking them in the wrong
regime. And `[R:verify-the-abstraction-compiled-away]` holds the case where the optimiser removed
the work the benchmark was timing -- a warm measurement of nothing at all.

## An unbounded queue converts overload into latency instead of refusal [R:a-queue-without-a-bound-has-no-overload-behaviour]

> Written for the rust and java audiences.

> Written down from Little's Law (J. D. C. Little, 1961): L = lambda W, so for a queue that never refuses, wait grows without bound once arrival rate exceeds service rate.

Decide what the queue does when consumption falls behind production. Growing is not a decision.

Queue depth *is* latency -- that is Little's Law and not a heuristic: with arrivals at rate lambda
and mean queue length L, the mean wait is L/lambda, so an unbounded L is an unbounded wait. A
queue with no capacity limit therefore has no latency limit, and a latency requirement stated
anywhere upstream of it is not held by anything.

The failure has a characteristic shape that makes it late to diagnose. Producers never block, so
nothing upstream reports a problem; throughput looks correct, because every item is eventually
processed; and the only symptom is that latency climbs monotonically. By the time it is visible
the backlog is doing work nobody wants -- every item consumed is already stale, so capacity is
being spent delivering answers to questions whose moment has passed, which is why the system does
not recover when load returns to normal. It has to drain first, at the same rate that fell behind.

So bound the queue, and choose the behaviour at the bound. Four are legitimate and the choice is
a domain decision, not a technical one:

* **Block the producer.** Real backpressure -- the slowest consumer sets the rate for everyone,
  and the pressure propagates to whoever can actually shed it. Wrong where the producer must not
  be stopped, which is the regime `[R:no-stall-inside-a-publication-window]` is about.
* **Drop the oldest.** Correct for market data, sensor readings, telemetry: a stale tick has
  negative value and the newest is the only one worth having.
* **Drop the newest.** Correct where the backlog is a fair queue and latecomers have not been
  promised anything.
* **Reject, and say so.** Correct for order entry and anything a caller must know the fate of. A
  silently dropped order is worse than a refused one by the whole width of the trust the caller
  placed in the call.

Not choosing selects a fifth behaviour that nobody would choose deliberately: absorb everything,
degrade every item's latency equally, and fail eventually by exhausting memory -- or, on a managed
runtime, by a collection pause caused by the backlog itself, which slows the consumer further and
is the closest thing in production to a feedback loop with the wrong sign.

The unbounded structure remains the right choice where the bound genuinely lives elsewhere -- a
queue whose producers are rate-limited upstream, or whose consumer is provably faster than any
possible arrival rate. Say which, in the code, next to the queue. An unbounded queue with a stated
reason is a decision; one without is the absence of a decision wearing the same shape.

Failure-mode check: **if consumption falls behind production for ten seconds, what does this queue
do and who finds out?** If the answer is "it grows" and "nobody", there is no overload behaviour
here, only an overload symptom waiting.

## A structure is correct only under the concurrency regime its proof assumed [R:a-structure-keeps-the-regime-it-was-proved-under]

> Written for the rust and java audiences.

> Written down from Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores -- where the regime is stated in the algorithm's name and nowhere in its code.

Read the regime out of the structure's correctness argument, then name what in the code prevents
violating it. "The documentation says single-consumer" is not a mechanism.

MPSC means multi-producer, **single**-consumer, and the single is a precondition rather than a
description. Vyukov's `mpscq_pop` reads `self->tail`, walks from it, and writes it back -- a plain
load and a plain store, unsynchronised, because exactly one thread was assumed to be executing
them. Two consumers interleave there and both can return the same node, or one can lose a node
entirely. Neither outcome raises anything. There is no assertion, no lock to contend, no error
path: the queue returns plausible values and the program continues with a duplicated or vanished
item.

That silence is the whole problem, and it is compounded by load. Two consumers running a hundred
operations may never interleave in the window that matters; the test suite passes, the staging
soak passes, and the interleaving arrives with production traffic. A defect that only appears
under the conditions you cannot reproduce is indistinguishable, from the outside, from a defect
somewhere else entirely.

So put the regime in the type, where the language allows it. Hand out exactly one consumer handle
at construction; make it neither `Clone` nor `Sync`; take `&mut self` on `pop` so the borrow
checker enforces exclusivity. Then "two consumers" is not a bug to find but a program that does
not compile -- `[R:make-illegal-states-unrepresentable]`, and `[R:typestate-for-protocols]` where
the regime changes across a structure's lifetime. This is the highest-leverage instance of that
discipline in the whole domain, because the alternative control is a sentence in a header
comment and the failure it prevents is unattributable.

Where the language will not carry it -- Java has no such handle -- the control has to be
something that actually runs: record the owning thread on first use and assert it on every
subsequent call in a debug build, or route all consumption through a single object whose
construction is the only place the invariant is stated. A comment is not a control
(`[R:guarantee-needs-a-reader]`).

The class is wider than queues, and the other members fail the same way. An SPSC ring with two
producers. A structure documented as not reentrant, entered from a signal handler or a callback.
A `HashMap` reached from two threads because the field that held it became shared three
refactorings later. An iterator held across a mutation. In each case the regime was a hypothesis
of the correctness argument and nothing in the artefact records it.

Failure-mode check: **what regime does this structure's correctness argument assume, and what
would fail if I violated it right now?** If the honest answer is "the name says so" or "we are
careful", the structure is unguarded and the next person to reach for it will not know.

## Memory that has never been touched is not yet memory [R:allocated-is-not-resident]

> Written for the rust and java audiences.

> Written down from Ulrich Drepper, "What Every Programmer Should Know About Memory" (2007), on demand paging and the cost of a first touch.

Pre-allocating is not preparing. A successful allocation buys address space; the page arrives on
first touch, and the touch is what costs.

`malloc` and `mmap` hand back virtual addresses. Physical pages are supplied lazily: the first
write to each page traps into the kernel, which finds a frame, zeroes it -- because the previous
tenant's data must not leak -- and maps it. So a one-gigabyte buffer allocated at startup and
first written on the hot path contains roughly a quarter of a million page faults, arriving one
per page, spread across the first pass over it. Every allocation-free assertion on that path is
satisfied and every one of them is measuring the wrong thing: nothing called the allocator.

Residency is also not permanent once achieved. Pages can be reclaimed under memory pressure or
swapped, so a long-lived process that touched its buffers at startup and then left them idle can
fault again on a path that has been fault-free for a week. That is why the remedy has two halves
and why only doing the first is a common and expensive mistake.

So, for anything on a path with a deadline: write to every page the path will touch, at startup,
before any real work arrives -- the touch has to be a write, since a read of an untouched page may
be served by the shared zero page and prove nothing. Then pin it: `mlockall(MCL_CURRENT |
MCL_FUTURE)`, and no swap on the box. On the JVM, `-XX:+AlwaysPreTouch` does the first half for
the heap and is worth the slower startup precisely because the cost is moved off the path that
has a bound.

Huge pages change the arithmetic in both directions and deserve their own decision rather than a
default. Fewer, larger pages means fewer faults and less TLB pressure; each fault is bigger, and
transparent huge pages bring a defragmentation path that can stall a thread for milliseconds at
exactly the wrong moment. Explicit huge pages reserved at boot avoid that; leaving THP on
`always` and hoping is the shape this domain keeps having to name.

**Touch from the thread that will own the memory, not from the thread doing the setup.** On a
multi-socket machine the first write also decides *placement*: a page is allocated on the node
of whoever faults it, so the obvious way to satisfy this rule -- one loop in the initialisation
thread walking every buffer -- puts every page on that thread's node, and a worker pinned to the
other socket then reads all of it across the interconnect at roughly twice the local latency.
The path is now fault-free and entirely remote: a slower steady state bought with a faster first
message. Pin the owning threads first, then let each of them touch what it will use.

Failure-mode check: **has every page this path will touch been written to, by the thread that
will read it, and what stops it being reclaimed?** Three questions, because answering only the
first leaves a system that is fast until it goes idle, and answering only the first two leaves
one that is fast on a single socket.

`[R:no-allocation-on-the-hot-path]` is the sibling and neither subsumes the other -- that one is
about entering the allocator, this one about the memory it already returned.
`[R:answer-the-requirement-at-its-layer]` is why the remedies are all a memory policy, a lock, or
a boot parameter: no choice of container or crate reaches this.
`[R:no-stall-inside-a-publication-window]` is where it bites hardest, because a page fault taken
between a writer's two stores holds the window open for its whole duration.

## Answer a requirement at the layer it lives at [R:answer-the-requirement-at-its-layer]

> Written for the rust and java audiences.

State which layer a requirement lives at before choosing anything, and satisfy it there.

A requirement about the machine -- do not perturb a neighbouring isolated core, do not
fault a page on the send path, do not take a lock a real-time thread contends for -- is
not answered by picking a faster crate, a better-reputed transport, or an asynchronous
API. Those are library-level decisions. They may be correct and they may even help, but
they do not *hold* a systems-level constraint, and treating them as though they do
converts an open question into a closed one with nothing behind it.

The failure is not the wrong choice; it is the wrong ledger entry. A constraint marked
satisfied stops being examined. An open constraint is still visible to the next person,
which makes leaving it open strictly better than closing it at a layer that cannot keep
it.

So name the layer explicitly, then name what holds the requirement there: a CPU affinity
mask, an isolated core list, a memory policy, a pre-faulted and locked mapping, a thread
priority, a cgroup. If the answer to "what holds this?" is the name of a library, the
requirement is unheld.

Two rules meet here and neither subsumes this one. `[R:measure-cost-per-task]` governs
*how to choose* a mechanism -- by what it actually costs and touches, never by
reputation -- and its failure-mode check asks under what configuration the mechanism
causes the exact harm it was chosen to prevent. That check is necessary and it is not
sufficient: a mechanism can pass it and still be the wrong layer to have asked at.
`[R:guarantee-needs-a-reader]` is the general form of the ledger failure -- a claim with
nothing reading the state it asserts -- and this is the shape that claim takes when the
state in question is a property of the machine rather than of the code.

Then run the pass the sibling rule requires: `[R:attack-the-design-in-a-second-pass]`
exists because the person who chose the mechanism is the last person able to see that
they answered at the wrong layer.

## Attack the design in a separate pass, and build nothing in it [R:attack-the-design-in-a-second-pass]

> Written for the rust and java audiences.

> Has recurred 1 time(s) since it was written; most recently 2026-09-14.

When a component is finished, do a second pass whose only job is to attack it. Build
nothing in that pass.

The pass that produced a design cannot review it. It carries every assumption that went
into the design, including the ones nobody stated, and it will read the output as
confirmation because that is what it was optimising for. A review folded into the build
is not a weak review; it is the build, wearing a different label.

So separate it in time and in intent, and give it something to do:

- **Justify the choice against the simpler alternative that was not taken.** Not "is this
  good" but "why not the obvious thing" -- and if the answer is that the obvious thing was
  never considered, that is the finding.
- **Walk the failure modes out loud.** Under what configuration does this do the thing it
  was chosen to prevent? What does it do under saturation, on a cold cache, on the first
  message after an idle period, when the peer is slow rather than absent?
- **Prove the stated constraint, do not restate it.** "It does not perturb the isolated
  cores" is a claim; the affinity mask, the thread inventory and the measurement are the
  proof. `[R:answer-the-requirement-at-its-layer]` is what this pass is usually checking.
- **Ask what the evidence would look like if the opposite were true.** An artefact that
  exists, a command that exited zero and a test that passed are all compatible with the
  work never having happened -- which is `[R:verdict-survives-the-channel]`, and it is the
  single most common thing this pass catches.

**Mandatory for anything latency-sensitive.** In that domain the failure does not announce
itself: a design that perturbs a neighbouring core, allocates on a send path, or faults a
page under load still passes every functional test, and the cost appears as unexplained
tail latency somewhere else entirely -- attributable to nothing, days later, by somebody
who was not there.

## A latency-bounded path does not enter the allocator [R:no-allocation-on-the-hot-path]

> Written for the rust and java audiences.

A path with a latency bound owns its memory before the path starts.

The bound you can state about an algorithm is a bound on the algorithm. Allocation is how
it stops being a bound on the program: the fast path of a modern allocator is a few
nanoseconds and a handful of instructions, and its slow path takes a lock, asks the kernel
for pages, faults them in on first touch, or -- on a managed runtime -- stops the
application. Those are not rare events that can be amortised into an average. They are the
tail, and the tail is the number the requirement was written about.

A concurrent collector does not remove this; it changes what you pay and when. ZGC and
Shenandoah move the bulk of the work off the stop-the-world pause, which is why their
pause figures are quoted in sub-milliseconds and why those figures are then read as
"allocation is free here". They are not the same claim. The collector now runs
*alongside* the application, competing for cores and memory bandwidth with the path that
has the deadline, and when the allocation rate outruns what it can reclaim concurrently
the allocating thread is stalled until it can -- an allocation stall, which does not
appear in the pause metric at all. So the number that governs a bounded path is the
allocation **rate** the path sustains, not the collector's advertised pause. Quoting the
pause is the managed-runtime instance of `[R:measure-the-claim-not-a-subset]`: a real
measurement of one thing, offered against a claim about another.

This is why a lock-free queue worth using is *intrusive*: the node is a field inside the
caller's message, already allocated, already resident, so enqueue stores pointers and
never asks anyone for memory. Wrapping that queue in an API that boxes each message
undoes its entire argument while leaving every benchmark of the queue itself intact --
`[R:answer-the-requirement-at-its-layer]`, with the requirement held one layer above where
it was measured.

Allocation is easy to reintroduce without writing the word. A vector that grows, a hash
map that rehashes, a formatted log line, a boxed closure, a trait object returned by
value, a collected iterator, an error type that carries a `String` -- and in Java, every
one of those plus autoboxing and varargs. So the control cannot be "remember not to
allocate": pre-size every collection and assert its capacity never changes, take the
buffer as a parameter instead of returning an owned one, and where the platform allows it,
install an allocator that aborts when called from a thread marked hot. A path that cannot
allocate without failing loudly is `[R:prefer-by-construction]`; a path that merely should
not is prose.

Failure-mode check, for every call on a bounded path: **what does this do when the
allocator takes its slow path, and what proved this cannot reach the allocator at all?**

Pre-faulting and locking the pages is the same argument one level down, and it has to be
made separately: memory that is allocated but never touched is not resident, so the first
write on the send path is a page fault whether or not anything called the allocator.

## A load generator that waits for the system deletes the system's worst latencies [R:no-coordinated-omission]

> Written for the rust and java audiences.

> Written down from Gil Tene, "How NOT to Measure Latency"; and HdrHistogram's recordValueWithExpectedInterval, which exists to correct it.

Ask, before reporting any latency figure: **when the system under test stalls, does my load
generator stall with it?** If it does, the figure is not a percentile of the system's behaviour.

A closed-loop generator sends a request, waits for the response, then sends the next. That is the
natural way to write one and it is measuring the wrong thing. Suppose the intended rate is one
request every 10ms and the system stalls for one second. A closed-loop generator issues **one**
request during that second and records **one** sample of 1000ms. An open-loop generator, or a real
user population, would have issued a hundred: the first waits 1000ms, the next 990ms, the next
980ms, down to the last. The true contribution of that stall is a hundred samples averaging about
500ms. The closed-loop generator recorded one.

The consequence is not a small error and not a noisy one. Ninety-nine of the hundred worst samples
were never taken, so they cannot appear at any percentile, and the tail -- the only part of the
distribution anyone asked about -- is computed over a sample set from which the bad events were
systematically removed. This is how "the 99.9th percentile is 2ms" and "the 99.9th percentile is
800ms" can both be measurements of the same system on the same afternoon.

Three properties make it worse than ordinary measurement error, and they are the reason this needs
a rule rather than care:

* **It is systematic, not random.** Running longer, repeating the experiment, or averaging across
  runs does not reduce it. Every run omits the same events for the same reason.
* **It always flatters.** The bias has one direction. A number that is wrong in an unknown
  direction invites suspicion; a number that is always optimistic gets believed.
* **It scales with the badness it hides.** The worse the stall, the more samples the generator
  fails to take, so the measurement degrades fastest exactly where the system does.

So: drive load open-loop at a stated rate, and record each request's latency from the time it
**should have been sent**, not from the time the generator got round to sending it. Where an
existing closed-loop harness cannot be replaced, correct the recording --
`recordValueWithExpectedInterval` exists for precisely this and is not an optional refinement.
And report the intended rate beside the percentile: a tail latency with no load attached to it is
not a measurement of anything, which is the same point `[R:no-retry-loop-on-a-contended-path]`
makes about producer counts.

Failure-mode check: **what stops my instrument from taking a sample, and would it stop at the
moments I care about?**

`[R:measure-the-claim-not-a-subset]` is the near neighbour and the two are deliberately separate.
There, an analyst measures a narrower set than the claim covers and manufactures a discrepancy;
the remedy is to state the scope. Here the *instrument* silently deletes samples while the analyst
does everything right, the scope looks identical, and the remedy is in the harness rather than in
the reasoning. Same family -- a subset wearing the whole -- different mechanism and different fix.

## Two hot values on one cache line contend without sharing anything [R:no-false-sharing-on-a-hot-line]

> Written for the rust and java audiences.

> Written down from Ulrich Drepper, "What Every Programmer Should Know About Memory" (2007), §3.3 on cache coherency and §6.4 on multi-threaded optimisation.

Know which cache line each hot field lands on, and who writes each line. Sharing a line is sharing,
whatever the program thinks it is doing.

Coherence is maintained per line, not per variable -- 64 bytes on x86-64 and most ARM, 128 on some
Apple parts. Two independent counters declared next to each other occupy one line, so a write by
core A invalidates core B's copy and B's next write must fetch it back. The threads never touch a
common value and never take a lock; they simply pass one line back and forth at the cost of a
cross-core transfer each time. Nothing in the source says "shared", which is why this survives
review: the code is correct, the design is correct, and the layout is the defect.

The instance worth memorising is the one in the structure this domain keeps reaching for. A ring
buffer or queue has a head written only by producers and a tail written only by consumers. Declare
them as two adjacent fields and every enqueue invalidates the line the consumer is reading its
tail from. The queue was chosen so that two threads would not contend, and the field order gave
the contention back.

The other reliable sites: an array of per-thread or per-core counters indexed by id, a lock placed
immediately before the data it protects, a flag beside the buffer it guards, and any struct where
a hot mutable field sits next to a hot read-only one -- the readers are invalidated by every write
even though they never write.

Separate what different cores write. `#[repr(align(64))]` on the field's wrapper type in Rust,
padding either side in C, `@Contended` on the JVM -- which does nothing unless
`-XX:-RestrictContended` is set, a detail that turns the fix into a comment if it is missed. Then
**verify the separation survived**: alignment attributes can be dropped by an allocator that
returns a less-aligned block, by a `Vec` of padded elements whose base is not aligned, or by a
field the compiler reordered. `[R:verify-the-abstraction-compiled-away]` applies exactly -- the
padding is a claim about the emitted layout, so read the layout rather than the source.

Pad what is measurably contended and nothing else. Every padded field is cache the rest of the
program does not get, and a struct padded everywhere on principle trades a contention problem for
a capacity problem, which is the same mistake with better intentions.

Failure-mode check: **which line does each of these hot fields land on, and which cores write
that line?** If the answer needs a diagram of the struct, draw it -- that diagram is the design.

## A contended path completes in a fixed number of steps, or its worst case is unbounded [R:no-retry-loop-on-a-contended-path]

> Written for the rust and java audiences.

> Written down from Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores (sites.google.com/site/1024cores).

On a path many writers contend for, prefer a structure whose writer side is a single
unconditional atomic operation over one that retries until it wins.

The Vyukov MPSC enqueue is the shape to reach for: swap the new node into the head with
one atomic exchange, then store the back-link. The exchange cannot fail, so it cannot be
retried, so the number of attempts is one whether one producer is pushing or sixty.

**That is a bound on steps, not on time, and the difference is the whole of what this rule
does and does not promise.** An exchange under contention still has to take the cache line
exclusively, and the line can only be in one core's cache at a time, so sixty producers
hammering it queue for it: the *time* for one enqueue grows with the number of contenders
even though the *step count* does not. What wait-freedom buys is that the growth is linear
and every attempt makes progress. A retry loop has no such floor -- an unlucky writer can
lose repeatedly, and the arrival of more contenders both lengthens each round and makes
losing more likely, so the tail grows faster than linearly and has no bound the code can
state. Choose the exchange because its worst case is a queue you can reason about, not
because it is free.

A Treiber-style CAS loop is the shape to justify before using. Read the head, build the
node, compare-and-swap, and go round again if somebody else got there first. Every
individual operation is cheap and the structure is lock-free by the textbook definition,
which guarantees that *the system* makes progress -- it guarantees nothing whatever about
*this writer*, which may lose arbitrarily many times. Lock-freedom is a liveness property
of the ensemble; latency is a property of the individual, and the two are routinely
confused because the word "free" is doing work it was never asked to do.

Read the progress guarantee per role, never for the structure as a whole. Vyukov's own
summary is "wait-free and fast producers", and it is exact -- about producers. The
consumer of the same queue is **obstruction-free**: it can be held up by a producer that
was descheduled mid-push, which is the listed disadvantage of the algorithm rather than a
subtlety of it. A structure described by the guarantee of its best-served role will be
adopted for the role it serves worst.

The measurement trap is what makes this worth a rule rather than a preference. A retry
loop's mean is almost unaffected by contention, because most attempts succeed first
time. What moves is the tail, and it moves superlinearly, so a benchmark at two producers
and a production deployment at thirty-two are not the same experiment. Do not report a
producer-side latency without reporting the producer count it was taken at.

Failure-mode check, before adopting any lock-free structure on a contended path: **how
many times can one writer go round before it succeeds, and what bounds that number?** If
the answer is "in practice, not many", the path is unbounded and nothing holds it.

`[R:measure-cost-per-task]` is the general form -- choose by what a mechanism actually
costs, never by its reputation -- and "lock-free" is exactly the kind of reputation it
warns about. `[R:answer-the-requirement-at-its-layer]` is the sibling that catches the
other half: a bounded producer path is a property of the algorithm, and it does not
survive being wrapped in something that allocates or takes a lock.

## A writer that can be stopped mid-publication makes the reader's latency the scheduler's [R:no-stall-inside-a-publication-window]

> Written for the rust and java audiences.

> Written down from Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores (sites.google.com/site/1024cores).

Before adopting a structure whose publication is two stores, name everything that can stop
the writer between them -- and treat the reader's worst case as however long that lasts.

The window is the same one the reader-side rule describes, seen from the other end. A
producer in Vyukov's MPSC queue performs the exchange, and at the point his source marks
`(*)` it has become the head while the previous node does not yet point to it. If the
producer is stopped there, the consumer cannot reach an item that is already in the queue.
His own summary is blunt about it: *"If producer blocked in (*), then consumer is blocked
too."*

So the consumer's latency bound is not a property of the queue. It is a property of
whatever can suspend a producer:

* **Cancellation.** A thread cancelled inside the window never closes it. The item is
  unreachable permanently, not slowly -- this is a deadlock, not a delay.
* **Preemption.** The producer loses its timeslice, or a higher-priority thread takes the
  core. The consumer waits for the producer to be scheduled again.
* **A safepoint or a collection pause.** On a managed runtime the producer is stopped by
  the runtime rather than by anything in the code, and the window is held for the whole
  pause.
* **A page fault** on the node's own memory, which is why `[R:no-allocation-on-the-hot-path]`
  and its pre-faulting paragraph are load-bearing here rather than merely adjacent.
* **Migration, steal time, a debugger, a signal handler that runs long.**

The sharp case is priority inversion, and it is sharp because the usual remedy is
unavailable. A low-priority producer stopped in the window holds up a high-priority
consumer -- the textbook shape -- but there is no lock, so there is no priority-inheritance
protocol to enable, and nothing in the structure to hand a priority to. The mitigation has
to be arranged outside the algorithm or it does not exist: producers at no lower priority
than the consumer, or cooperative scheduling, or both. A "lock-free" label reliably reads
as "immune to inversion", and here it is the opposite -- it removes the mechanism that
would have solved it.

The remedies are all systems-layer, which is the point:

* Do not enqueue from a thread that can be cancelled, and keep insertions outside
  cancellable sections -- the C11 implementation of this algorithm says exactly this, and
  recommends a cooperative threading model instead.
* Do not enqueue from a signal handler, a finalizer, or any context the runtime may stop.
* Give producers a priority and an affinity that bound how long they can be off-core, and
  state the bound.
* Where the reader has a hard deadline, prefer a structure whose publication is a **single**
  release store -- a bounded slot array with a sequence counter, the shape of Vyukov's own
  bounded MPMC queue -- and pay for it in capacity. An unbounded queue with a two-store
  publication trades a memory bound for a latency bound, and that trade should be made on
  purpose.

Failure-mode check, before a two-store publication goes on a path with a deadline: **what
can stop this thread between the two stores, and what is the reader's latency while it is
stopped?** If the answer names the scheduler or the collector, the deadline is held by
neither.

`[R:answer-the-requirement-at-its-layer]` is why none of the remedies above is a library
choice: every one of them is a priority, an affinity, a threading model or a page
residency decision, and swapping the queue implementation answers none of them.
`[R:no-retry-loop-on-a-contended-path]` is what the window bought -- a wait-free producer --
and this rule is the invoice. `[R:transient-state-is-not-a-terminal-state]` is the reader's
obligation inside the same window; satisfying it is necessary and does nothing whatever
about a writer that has stopped.

## A syscall on a bounded path hands the deadline to the kernel [R:no-syscall-on-a-bounded-path]

> Written for the rust and java audiences.

Know which calls on the path enter the kernel. The answer is not readable from the source, because
nothing about the spelling distinguishes a syscall from a function call.

A mode switch is the floor, not the cost. On top of it sits whatever the kernel does inside the
call, plus what it evicts: the return comes back to a cache and TLB partly filled with kernel
working set. The floor itself moved within living memory -- the Spectre and Meltdown mitigations
(page-table isolation, indirect-branch controls) multiplied the entry and exit cost several times
over, so a great deal of latency-sensitive code was measured and written in a world that no longer
exists and has never been re-measured in this one.

The calls that reach the kernel without announcing it are the ones to look for:

* **The clock.** `clock_gettime` is served from the vDSO -- no syscall, tens of nanoseconds --
  only while the clocksource is TSC. With the clocksource at HPET or `acpi_pm`, the same call
  traps, and costs a microsecond or more. Read
  `/sys/devices/system/clocksource/*/current_clocksource` on the actual machine rather than
  assuming. A timestamp taken per message is the commonest way this lands on a hot path.
* **Logging.** A log call that formats and writes is a write syscall, and a blocking one if the
  sink is a pipe or a full buffer. Format and hand off; never write from the path.
* **Allocation.** The allocator's fast path is userspace; its slow path is `mmap` or `brk`. This
  is the kernel-side half of `[R:no-allocation-on-the-hot-path]`, and `[R:allocated-is-not-resident]`
  is the same trap reached without any call at all -- a page fault is a trap into the kernel that
  no audit of call sites will find.
* **Locks.** An uncontended futex-based mutex stays in userspace; a contended one enters the
  kernel to sleep. So the lock's cost is a function of the contention, which is a function of
  load, which is why it is absent from every unloaded measurement.
* **Sockets.** Every send and receive, unless the path is kernel bypass or busy-poll.

Remedies are per-call and mostly mean moving the work off the path rather than making it cheaper:
batch and defer logging to another thread, take the timestamp once and pass it, pre-allocate and
pre-fault, keep locks uncontended or remove them, and use bypass or busy-polling where the I/O
itself is the bound.

Failure-mode check: **which of the calls on this path enters the kernel, and what did I read to
know that?** If the answer is that it looks like a library call, nothing has been established --
and the clock is the one that will be wrong.

## A sampling profiler reports where it was allowed to stop, not where the time went [R:profiler-samples-where-it-can-stop]

> Written for the rust and java audiences.

> Written down from The Java safepoint-bias literature (Nitsan Wakart, Psy-Lob-Saw); async-profiler's rationale for AsyncGetCallTrace over JVMTI stack walks.

Before acting on a profile, ask what the profiler was **able** to sample -- then assume everything
it could not sample is missing from the answer.

A sampling profiler does not interrupt a thread wherever it likes. It stops it where stopping is
legal, and on a managed runtime that means a safepoint: a poll the JIT inserted at a method return
or a back-edge it could not prove bounded. A counted `int` loop is exactly the shape the JIT
proves bounded, so it may contain **no safepoint poll at all** -- and a thread spinning inside one
cannot be sampled while it is there. The sampler waits, the thread leaves the loop, the sample is
taken at the next legal point, and the time is attributed to whatever runs next.

The result is not a blurred picture. It is a picture of a different program. The method consuming
most of the wall clock can appear nowhere in its own profile, while the small method after it
appears to dominate. The remedy people reach for -- sample more often, run longer -- does nothing,
because the bias is deterministic: the same code is unsamplable on every run, so more samples
means more samples of the same lie. This is the shape `[R:verify-ordering-on-the-weakest-target]`
names in a different domain: the defect is not *rare* in the instrument, it is *absent* from it,
and repetition cannot find what the instrument cannot represent.

So choose the instrument by where it can sample, and say which one was used beside any profile
that decides something:

* On the JVM, a sampler built on `AsyncGetCallTrace` (async-profiler) takes its sample from a
  signal handler rather than at a safepoint, which is the whole reason it exists. Hardware PMU
  sampling via `perf` with a JIT symbol map is the same argument one layer down.
* Anything that walks stacks through JVMTI or `Thread.getStackTrace` -- which includes most
  IDE-bundled and APM profilers -- is safepoint-biased by construction. It is not useless; it is
  answering a different question than the one being asked of it.
* Outside a managed runtime the same structure applies with different names: a signal-based
  sampler cannot sample where signals are blocked or deferred, and an instrumenting profiler
  changes the code it measures, so inlining decisions differ between the profiled build and the
  shipped one.

Failure-mode check, before believing any profile: **where can this profiler not take a sample, and
what would code living there look like in its output?** The answer is that it would look like the
code that runs immediately afterwards -- which is indistinguishable from a real finding.

`[R:measure-cost-per-task]` is the general form, measure rather than assume, and this is the case
where measuring is not sufficient because the measurement itself carries the assumption. The
sibling in this domain is `[R:no-coordinated-omission]`: there the instrument fails to take
samples during the events that matter, here it fails to take them in the code that matters.

## A window between two stores is not a state the reader may act on [R:transient-state-is-not-a-terminal-state]

> Written for the rust and java audiences.

> Written down from Dmitry Vyukov, Intrusive MPSC node-based queue, 1024cores; and grivet/mpsc-queue, a C11 implementation whose poll() splits the window out as MPSC_QUEUE_RETRY.

When a writer publishes in two steps, the reader can see the middle. Decide what the
middle means before writing the read path, and never let it share a return value with a
settled state.

The Vyukov MPSC enqueue is an exchange followed by a store: the new node becomes the head
first, and only then does the previous node learn to point at it. Between those two
instructions the queue is fully populated and partly unreachable -- a consumer walking
from the tail finds a successor link that is still null. That null does not mean empty. It
means *not yet linked*, and the producer that owns the gap is a handful of cycles from
closing it.

A consumer that returns "empty" there has not merely returned early; it has converted a
few nanoseconds of writer-side latency into however long it takes for something else to
wake it up. If the consumer parks on an empty result, the item sits in a queue that
already contains it, and the symptom is an occasional multi-millisecond delivery in an
otherwise microsecond system. It reproduces under contention and vanishes under a
debugger, which is what makes it expensive to find later and cheap to prevent now.

**The exchange rate is what makes this expensive rather than untidy.** Waking a parked
thread is a futex into the kernel, a scheduler decision that need not be immediate, and a
restart with cache, TLB and branch predictors belonging to whatever ran in the meantime:
single-digit microseconds at best, tens to hundreds under load, against an enqueue costing
tens of nanoseconds. So misreading the window does not cost the window's width; it costs
three orders of magnitude more than the entire queue.

Which means the consumer's behaviour on an empty queue is a design decision in its own
right, and it has to be made rather than defaulted. Pure spinning gives the lowest and most
predictable delivery latency and costs a whole core -- and on a hyperthreaded sibling it
steals issue slots, so an idle spinner can slow the producer it is waiting for, and
spinning on a core the producer also runs on is worse than parking outright. Pure parking
costs nothing while idle and pays the wake on every item. Between them sit spin-then-yield,
and spin-briefly-then-park with the spin window sized to the observed inter-arrival gap,
which is usually the right answer for bursty arrivals: the burst is served at spin latency
and the quiet period costs nothing. Whether spinning is affordable at all is a question
about the machine's core budget and isolation, not about the consumer's code --
`[R:answer-the-requirement-at-its-layer]`.

A consumer that distinguishes the two states can take the cheap option for the one and the
expensive option for the other: spin through "not yet", which resolves in nanoseconds, and
park only on "drained", which is the state that might last.

Distinguish the two cases explicitly. If the successor is null *and* the tail is the
head, the queue is genuinely drained. If the successor is null and the tail is not the
head, a producer is mid-push: spin briefly, return a distinct "not yet" answer, or both --
but do not return the same value you return for drained, because the caller's correct
response is different in each case and a shared return value makes choosing impossible.
That is `[R:no-sentinel-values]` in a concurrent setting: "empty" and "a writer is in the
window" are two states, and collapsing them into one null is exactly the sentinel the rule
forbids.

**This rule is deliberately stronger than the reference implementation, and the evidence
is that the reference was corrected downstream.** Vyukov's own `mpscq_pop` reaches exactly
this fork -- `mpscq_node_t* head = self->head; if (tail != head) return 0;` -- and returns
the same `0` it returns for drained. He documents the consequence rather than fixing it,
as the algorithm's single listed disadvantage: *"Push function is blocking wrt consumer.
If producer blocked in (*), then consumer is blocked too."* The null is load-bearing and
uninformative at the same time. A later C11 implementation of the same algorithm
(`grivet/mpsc-queue`) splits the return into `MPSC_QUEUE_ITEM`, `MPSC_QUEUE_EMPTY` and
`MPSC_QUEUE_RETRY` -- the third value is this window, given a name. Do that.

The precise vocabulary is worth keeping, because it is what makes the window findable in a
specification rather than in a debugger: the queue is **serializable but not
linearizable**. Insertions are consistently ordered across producers, but an insertion is
two memory transactions, so the state can be found inconsistent *within* the series. The
consumer is correspondingly **obstruction-free, not wait-free** -- its progress depends on
another thread's, which is the asymmetry the headline "wait-free producers" conceals.

Failure-mode check, for any structure whose publication is more than one store: **what
does a reader see between them, and which of my return values is it currently sharing?**

The general form is a reader acting on a state the writer never intended to publish. It
arises anywhere a commit is not a single instruction -- a two-phase update, a length
written after a payload, a version counter bumped after the data it protects -- and the
remedy is always the same: make the intermediate state nameable, then refuse to let it
alias a terminal one.

## Memory ordering is verified on the weakest architecture it ships to [R:verify-ordering-on-the-weakest-target]

> Written for the rust and java audiences.

State the weakest memory model the binary will run under, and verify the ordering there --
not on the machine the code was written on.

x86 is total-store-order: loads are not reordered with loads, stores are not reordered
with stores, and an ordinary access already carries most of what release-acquire asks for.
So a missing acquire on the consumer side of a queue, or a relaxed store where a release
was needed, is not an intermittent bug on that hardware -- the processor will not reorder
what the code got wrong. Every run passes, the stress test passes, the loop runs overnight
and passes. On aarch64 -- a laptop, a cloud instance, an embedded target -- the same source
reorders and the queue hands out a node whose contents are not yet visible.

**There are two reorderers, and only one of them is the processor.** A relaxed access
where an acquire was needed licenses the *compiler* to move other accesses across it, and
the compiler is the same compiler on every target. So x86 is not a safe platform for a
wrong ordering; it is a platform on which one of the two mechanisms cannot expose it. The
hardware half of the defect is absent there and the software half is not, which is why the
failure on x86 is rare and confusing rather than impossible -- and why "we have never seen
it in production" is evidence about the optimiser's mood on a particular build, not about
the ordering. The correct ordering is what makes both reorderers behave; the architecture
only decides which one catches you.

This is the exact shape `[R:verify-through-production-path]` warns about, with the
production path being a processor rather than a wiring channel: the test exercised a
stand-in whose hardware guarantees are strictly stronger than the real one, so a pass
carries little information about the case that matters. And the usual defence against an
untestable property -- run it more -- buys almost nothing here, because repetition on TSO
samples the same hardware guarantee every time and varies only the compiler's choices,
which are fixed for a given build.

So the ordering is held by an argument and a tool, never by a green test on one machine:

* Name the pairing at the point of use. Every release has a named acquire that reads it
  and the comment says which -- an ordering with no partner is either a mistake or a
  relaxed access that has not admitted it.
* Run the model checker or the race detector rather than the program. `loom` in Rust,
  `jcstress` on the JVM, TSAN under the C++ model: these explore the orderings the
  hardware is permitted to produce rather than the one it happened to produce.
* Where the target is known, build and run the test suite on it. An aarch64 CI runner is
  ordinary infrastructure now, and it is the only cheap thing on this list that tests the
  actual machine.

Failure-mode check, before accepting any atomic that is not sequentially consistent:
**which architecture would show me this was wrong, and have I run it there?** If the
answer is that it passes locally, the ordering is unverified -- `[R:guarantee-needs-a-reader]`,
with the unread state being the hardware's.

## Async is async all the way down [R:async-all-the-way]

No `block_on` inside async code. An async runtime multiplexes many tasks onto few threads, so a blocking call does not delay one task -- it removes a worker from the pool for the duration and delays every task that would have run there. The victims are unrelated to the code that blocked, which is why the symptom is unexplained tail latency somewhere else entirely, and why it is close to unattributable after the fact.

`block_on` is correct only at the boundary where synchronous code enters async: `main`, a test, a callback from a C library. Once inside, stay inside -- async I/O, an async-aware lock wherever a guard must survive an `await`, and `spawn_blocking` for work that genuinely blocks, such as CPU-bound compute or a synchronous third-party client.

The same applies to a `std::sync::Mutex` guard held across an `await`. Nothing flags it as blocking, but it holds a lock while the task is descheduled, so the contending task blocks its own worker thread and the failure presents as a deadlock rather than as a lock.

Nested `block_on` on a current-thread runtime does not degrade -- it deadlocks outright. That is the honest failure; the multi-threaded case merely hides the same mistake behind a thread count, until load removes the hiding place.

## Take a borrow in the signature unless the function needs to own [R:borrow-in-signatures]

Take `&str` over `String`, `&[T]` over `Vec<T>`, `&Path` over `PathBuf`. A signature is a statement about what the function needs, and an owned parameter says "I will keep this" -- so when the body only reads, the signature is untrue and every caller pays for it in an allocation.

Own the parameter exactly where the function stores it. There, `impl Into<String>` is the courteous form: a caller holding a `String` moves it in for nothing, and a caller holding a `&str` allocates once, knowingly. On the way out, return `&str` rather than `&String` -- the extra indirection buys the caller nothing and pins the field's representation into the public API.

The cost is rarely the single allocation. It is that an owned parameter propagates: the caller clones to satisfy it, its caller clones to satisfy that, and a signature chosen without thought becomes a column of clones that each look locally necessary. R:justify-every-clone is where those clones surface; this rule is how they are never created.

## Design by writing the types first, before any logic [R:design-types-first]

Sketch the types until the design falls out, then write the logic. The type-level sketch is the executable specification a test-first red phase is reaching for, and it is the stronger one: a test samples points of the behaviour space, a type constrains the whole space and the compiler proves it everywhere, at compile time, for as long as the code exists. When the types are right much of the implementation writes itself, and many wrong implementations stop compiling. In Rust this ordering supersedes any test-first default -- tests are not removed, they are demoted to the layer where they are the right tool: property tests for behavioural laws the types cannot encode, unit tests as regression pins for past bugs. Apply each guarantee at the strongest layer that can hold it, and when reviewing, ask first not "does it pass" but "which of these guarantees could move up a layer?"

## Error enums carry the values that identify the failure [R:errors-name-what-failed]

Every distinct failure is a variant of a `thiserror` enum, and every variant carries the values that identify the instance:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("TCP connect to {addr} failed")]
    Tcp { addr: SocketAddr, #[source] source: io::Error },
    #[error("timed out after {elapsed:?}")]
    Timeout { elapsed: Duration },
}
```

Two things follow that a string cannot give. The caller can `match` -- retry a `Timeout`, surface a `Tcp` -- rather than parsing prose that changes the next time someone edits a message. And the message names the actual address, so one log line is enough to act on rather than the start of an investigation.

Keep the cause in `#[source]` instead of interpolating it into the text. The chain then prints once, at the edge, without each layer restating the layer beneath it.

One variant per condition a caller could plausibly treat differently. Collapsing four causes into `Other(String)` re-creates the string error inside an enum: it reads as a type and behaves as prose, and it is the shape this rule exists to catch.

## Every clone() carries its reason, or the design is wrong [R:justify-every-clone]

A borrow-checker error is a question about ownership. `clone()` does not answer it; it pays to avoid answering it. Try the answers first: restructure so one owner is obvious, take a borrow with a named lifetime, split the borrow across smaller fields, or share with `Arc`/`Rc` where the value is genuinely shared rather than copied.

Where a clone survives that examination, write the reason beside it, in the house form used throughout this codebase:

```rust
let sources: Vec<RuleTag> = rules.iter().map(|r| r.tag().clone()).collect(); // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
```

The comment is what makes the rule reviewable. Unannotated clones are indistinguishable from one another, so a reviewer has to re-derive the ownership argument for every one and in practice re-derives it for none. An annotated clone states a claim that can be checked, and that can be found and removed the day the design around it changes.

The reason must be about ownership or lifetime. "To make it compile" is the error class restated, not a justification. Two copies of a value the code believes is one value is a correctness bug waiting for whoever mutates the wrong one.

Most clones that survive review were created by a signature, not by a call site: R:borrow-in-signatures is where the pressure comes from, and fixing the parameter usually deletes the clone rather than annotating it.

## Modules are the encapsulation boundary; pub is a deliberate export [R:module-visibility-is-deliberate]

Use `pub(crate)` and `pub(super)` freely; reserve bare `pub` for what the crate deliberately exports. Visibility is not paperwork. It is the statement of what may still be changed freely, and it is the only such statement a compiler can check.

The default matters because `pub` is cheap to add and expensive to remove. Once an item is public an external caller may depend on it, so the module can no longer be reorganised, the field can no longer be renamed, and the helper written for one call site is now a supported API. None of that was decided; it was defaulted into.

A module holds an invariant in the same way a type does. A helper that may only be called after a check belongs beside that check, private to the module, so "only after" is enforced by nobody elsewhere being able to call it at all. Making it public converts that guarantee into a doc comment.

Re-export the intended surface explicitly at the crate root with `pub use`, and let everything behind it be as private as it can be. The public API is then a list someone wrote, rather than the residue of where the code happened to live.

R:private-fields-only is the same argument one level down, at the field rather than the item. The two are separable -- a private field on a `pub` type leaks structure, a `pub(crate)` type with public fields leaks none -- so satisfying either says nothing about the other.

## Mark consequential return values #[must_use] [R:must-use-on-consequential-returns]

Put `#[must_use]` on every function that returns a `Result`, and on every function whose return value is the point of calling it. The compiler then flags a discarded outcome at every call site, present and future, instead of leaving it to a reviewer to notice one bare statement among a hundred.

Prefer the attribute on the *type* over the attribute on the function. `#[must_use] struct Receipt;` travels to every function that returns a `Receipt`, including the ones written later by someone who never read this rule; the function-level attribute has to be remembered each time. Attach it to the type whenever the type is always consequential, and fall back to the function only for the narrower case where the same type is sometimes worth discarding.

Escalate the resulting warning to an error in CI. A `#[must_use]` whose violation prints a note nobody reads is a comment with extra syntax, and the guarantee it claims is held nowhere.

## Newtype liberally: distinct concepts get distinct types [R:newtype-liberally]

Give every domain concept its own type, even when the underlying representation is identical. `Miles(f64)` and `Kilometers(f64)`, `Host(String)` and `Port(u16)` -- never two bare `f64`s or a `String` and a `u16` whose order only a human remembers. Newtypes are ordinarily zero-cost: they compile to the same machine code as the primitive, so the only thing they add is the compile error you want -- but where that cost is load-bearing, R:verify-the-abstraction-compiled-away requires the claim to be checked rather than repeated. The unit mix-up that cannot be written is cheaper than the one caught in review, and far cheaper than the one that is not. R:parse-dont-validate is where the newtype comes from -- the boundary mints it as a witness; this rule is about carrying it everywhere afterwards instead of unwrapping back to the primitive.

## No anyhow in library return types [R:no-anyhow-in-libraries]

> Also enforced by hook:no-anyhow-in-lib.

Library crates return typed error enums (thiserror), so a Result says exactly what can go wrong and callers can match on it. anyhow belongs only at the outermost binary edge. This rule has graduated: the no-anyhow-in-lib hook now enforces it deterministically at write time.

## No unwrap() in production code [R:no-unwrap-in-production]

> Also enforced by hook:no-unwrap-in-src.

No unwrap() in production code. Use expect() only with a meaningful panic message that names the resource and the invariant, or return the error with `?` and let the caller decide how to surface it. This rule has graduated: the no-unwrap-in-src hook now enforces it deterministically at write time, so the instruction layer no longer has to.

## Parse, don't validate [R:parse-dont-validate]

Transform raw input into a rich domain type at the outermost boundary, producing a witness newtype whose existence proves the check happened. Interior code takes the witness and never re-checks -- one perimeter, one check, enforced everywhere after by the compiler. R:parse-wide-then-range-check sharpens this: the boundary must be able to see the illegal value in order to name it.

## Parse wide, then range-check [R:parse-wide-then-range-check]

Parse into a type wide enough to *represent* the out-of-range value, then range-check to mint the narrow newtype. The perimeter must be able to see the illegal value in order to name it illegal; parsing directly into the target type collapses "out of range" into "not a number" and makes the OutOfRange class a lie the compiler will not catch.

## Struct fields are private; construction goes through a constructor [R:private-fields-only]

Domain types have private fields. The only way to construct one is a smart constructor that enforces the invariant and returns a `Result`; the only way to read one is an accessor.

A `pub` field is a second constructor that checks nothing. Every guarantee the smart constructor establishes is void the moment a caller can write the field directly, and the type's name goes on claiming it. This is the same defect as a validator that some call sites skip, moved from the function layer down to the field layer, where it is harder to see.

Hand out the narrowest borrow the caller can use: `fn host(&self) -> &str`, not `&String`, and never `&mut` on a field the invariant depends on. Where a caller genuinely must change the value, give it a method that re-establishes the invariant, or one that consumes the value and mints a new one.

R:parse-dont-validate is what the constructor does; this rule is what makes it the only door.

## Seal a trait whose set of implementors is closed [R:seal-closed-trait-sets]

When a trait exists to describe a fixed family -- the kinds of unit, the supported wire formats, the stages of a protocol -- seal it, so only this crate can implement it:

```rust
mod private { pub trait Sealed {} }
pub trait UnitKind: private::Sealed { fn path_suffix() -> &'static str; }
```

The point is not to be unwelcoming. It is that "closed" is either a fact the compiler enforces or a sentence in a doc comment. An unsealed trait can never gain a required method without a breaking change, can never be reasoned about as a whole set, and can never assume it has seen every implementor -- yet the code around it will be written as though it can.

Seal by default where the set is closed today, and leave the trait open only where third-party implementations are a deliberate feature. Opening a sealed trait later is additive and painless. Closing an open one is a breaking change, which in practice means it never happens.

## Builders make a missing required field a compile error [R:typestate-builder-for-required-fields]

Track each required field in a phantom type parameter, and implement `build()` only for the fully populated combination:

```rust
pub struct Missing;
pub struct Present;
pub struct ServerBuilder<HasHost, HasUser> { /* ... */ }
impl ServerBuilder<Present, Present> { pub fn build(self) -> Server { /* ... */ } }
```

An incomplete build is then not an error value to handle but a method that does not exist. Compare the alternative: `build()` returns `Result<_, MissingField>`, every caller writes the same `?`, and the one caller who writes `unwrap_or_default()` ships a server pointing at nothing.

This is R:typestate-for-protocols applied to construction rather than to sequence -- the same remedy, a different illegal thing. Optional fields stay plain setters; only what is genuinely required earns a parameter.

Beyond three or four required fields the parameter list costs more than the problem. Take a required-arguments struct in `new()` instead: the same omission is still a compile error, with none of the machinery.

## Typestate for protocols: out-of-order calls should not compile [R:typestate-for-protocols]

When a sequence has rules -- connect before authenticate, init before run, configure before start -- encode the stage in the type, not in a field. Each step consumes the value in one state and produces it in the next, so a method that is invalid in the current state simply does not exist and calling it is a compile error. This is R:make-illegal-states-unrepresentable applied to time rather than to structure: the illegal thing is not a contradictory pair of fields but an operation at the wrong moment, and the same remedy applies -- make it unrepresentable rather than guarded. A runtime `if !self.authenticated { return Err(...) }` in a method that should not exist yet is the shape to look for.

## Verify an absent cost; reputation and a fast benchmark both lie [R:verify-the-abstraction-compiled-away]

"Zero-cost" is a property of a particular abstraction, under a particular optimiser, on a particular build profile. In Rust it is usually true, which is precisely why it gets asserted instead of checked, and why the cases where it is false survive review.

Where the cost is load-bearing -- a hot path, a latency budget, an allocation-free claim -- look at what was emitted. `cargo asm` for the function, `cargo bloat` for the binary, a benchmark on the profile that actually ships. A debug build proves nothing about a release binary: a newtype that is free at `opt-level = 3` need not be at `opt-level = 0`, and the two are different programs.

Then run the failure-mode check R:measure-cost-per-task states in the general case -- under what configuration does this cost exactly what it was chosen not to cost? A `#[repr(transparent)]` newtype crossing an FFI boundary, an iterator chain that fails to fuse because the closure captures by reference, a generic that is never monomorphised because it went out through a trait object. Bound that configuration, or drop the claim.

The same optimiser runs the other way, and that half is more dangerous because it arrives carrying a measurement. A microbenchmark whose result is never used, or whose input is a compile-time constant, is dead code: the optimiser deletes the work and the harness times an empty loop. The reading is not "this is fast" but "this did not happen", and the two are indistinguishable in the output -- an implausibly good number is the only signal, and an implausibly good number is exactly what the author was hoping for. So read the emitted code for a benchmark as readily as for a claim, consume every result through a black box the optimiser cannot see through (`std::hint::black_box`, JMH's `Blackhole`), and treat a figure at or near zero as a defect report on the harness until the assembly says otherwise.

Both halves are one act: an absent cost is a fact about emitted code, so it is established by reading emitted code. Reputation asserts it without measuring; a deleted benchmark measures without establishing it.

This is R:verify-through-production-path applied to code generation: a claim measured on a build the user never runs is evidence about that build alone. Where the cost is not load-bearing, do not make the claim at all -- an unverified performance assertion in a doc comment is read as a measured one.

## A test fixture must work on every OS the repository runs on [R:xplat-fixtures]

A test that passes only on the machine that wrote it is a latent lie, and it is a
particularly expensive one: it does not fail, it certifies. Where a repository is worked
on from more than one operating system, every fixture that spawns a process, locates a
binary, compares filesystem paths, or parses another tool's output must be written for
both, because the one that is not will report success on the authoring OS indefinitely.

Locate a sibling binary from the running test, never from a constructed path.
`CARGO_BIN_EXE_*` exists only for the bins of the crate under test; for anything else,
start at `current_exe()`, pop `deps`, pop the profile directory, and join the name with
`std::env::consts::EXE_SUFFIX`. A literal `target/<profile>/<name>` ignores both
`CARGO_TARGET_DIR` and the platform's executable suffix, and the failure it produces is
an exec error rather than a missing-file error, which reads as a broken binary rather
than a broken path.

A fixture that must run as a child process is a small program in the language of the
repository, compiled once per test run into `CARGO_TARGET_TMPDIR` behind a `OnceLock`.
The toolchain is guaranteed present wherever the tests run; an interpreter is not. When
generating such a program's source, write the payload through a byte-level write rather
than a formatting macro, or braces in the payload are parsed as format placeholders.

Canonicalize both sides before any path comparison. On Windows `canonicalize` returns the
extended-length form, so a prefix or equality assertion against a raw path fails for a
path that is in fact correct.

Strip carriage returns from another tool's output before comparing it. Many ports
terminate lines with CRLF; capturing a command's output removes the trailing newline but
leaves the final line's CR, so exactly one record per stream carries a stray byte and
never matches its twin. The result looks like real drift, is invisible on the other OS,
and an always-red check is a muted check.

Treat a green run as evidence for the operating system it ran on and no other. A fixture
recorded as an enforcing artefact on the strength of a single-platform run is a claim
about a guarantee that was never tested where it was most likely to break.

This is the in-flight half of a pair. Its sibling governs bytes at rest -- what a checkout
puts on disk, fixed once and structurally in `.gitattributes`. This rule governs bytes in
flight, what a tool emits into a pipe at runtime, which no file attribute can reach, so
the fix belongs at the consuming end. A repository can satisfy either and fail the other.

## A role belongs to an edge endpoint, never to the thing at the end of it [R:role-is-an-edge-property]

> Also enforced by test:the_repository_derives_no_queue_role_from_a_spin_mode (documents) + test:spin_mode_does_not_change_a_single_queue_end_letter (renderers).

> Has recurred 2 time(s) since it was written; most recently 2026-08-30.

Read a role from the DIRECTION of an edge, never from a property of a service.

A service that writes into a queue is that queue's producer; one that reads from it is a
consumer. Any service is routinely both at once -- a mid-chain hop consumes from its
inbound queue and produces into its outbound one -- so a role assigned to a whole service
is only ever correct for a pure source or a pure sink, and the diagram is wrong
everywhere else.

Spin mode says how a service WAITS, not what it does with a queue. What it legitimately
says is how often an endpoint pays a socket crossing: a cost, not an identity. That is
`SpinCost` and `HandoffSeverity` in `shared/src/model/handoff.rs`, and neither type can
express a role, so neither can drift back into asserting one. Making the wrong statement
unrepresentable is what ended this, after prose had failed three times.

The recurrences are the lesson, not the original error. A derivation removed from the
code lives on in the doc comments, and removed from the doc comments lives on in the
domain-knowledge section -- each fix landing where the last reader complained rather than
everywhere the claim is made. When a correction is to a *concept*, grep the whole
repository for the claim before calling it fixed, and pin it with a test that reads the
concept rather than the surface: `spin_mode_does_not_change_a_single_queue_end_letter`
would have failed on day one.

## Seed data reaches only the installations that did not exist yet [R:seeded-data-needs-a-migration]

> Also enforced by test:the_catalogue_has_not_grown_without_a_migration_to_carry_it.

Adding a row to the built-in catalogue under `shared/src/model/ipc/catalog/` is **not**
a code change on its own. It needs a new version function in `server/src/db/migrations.rs`
that calls `seed_builtin_library`, and a bump of `SCHEMA_VERSION`, in the same change.
Do not merely update the count -- add the version first.

The reason it is easy to miss is that every test agrees with you. `test_db()` builds its
schema by running `migrate()` from v0, so a test database is always a *fresh install* and
the upgrade path has no test unless one is written deliberately. Pin the upgrade, not
just the outcome: `v9_carries_the_chronicle_event_loop_row_into_an_existing_database` is
the shape -- open a database at the previous version, migrate it, and assert the row
arrived.

Seed on a version bump, never on every open. Seeding on open would resurrect every row a
user deleted, on every restart, and deletion would stop meaning anything -- which is why
`ipc_library_deleted` exists and why the seeder's insert carries `WHERE NOT EXISTS`
against it. A migration must respect a deletion; `restore-builtins` is the route that
exists to undo one, and it clears the tombstones first.

Failure-mode check: **which databases does this change actually reach?** If the answer is
"the ones created after it ships", the migration is missing and the suite will not tell
you.

`SEEDED_CATALOGUE_ROWS` fails the moment the row count moves without a version to carry
it, so the omission cannot ship quietly. That check exists because the prose version of
this rule did not hold.

## A character the UI draws must have a glyph in the faces the app actually loads [R:verify-the-glyph-exists]

> Also enforced by test:client/src/app/font_tests.rs + gate:verify.sh emoji_ban.

Ask the font stack, not the document. A character that renders in an editor, a terminal
or a markdown preview proves nothing about the four faces egui loads at runtime, and the
failure is silent -- an empty box, not an error.

Prose cannot hold this, and this repository has the proof: the rule existed, was read,
and prescribed a codepoint that produced the exact defect it forbade. When the
instruction itself is the defect, re-reading it does not help, which is the argument for
pushing a rule down a layer rather than restating it more firmly.

Two controls now hold it, and they cover different halves:

* `client/src/app/font_tests.rs` asks the REAL `FontDefinitions::default()` -- the very
  stack the app builds -- whether each non-ASCII character the UI draws has a glyph, in
  the proportional and monospace families both. This is the positive half: what the table
  in CLAUDE.md claims is now asserted against the thing it claims about.
* `scripts/verify.sh` `emoji_ban()` blocks two whole codepoint blocks outright in
  non-comment lines under `client/src` -- astral emoji (U+1F000-U+1FAFF) and fullwidth
  forms (U+FF00-U+FFEF). The BLOCK is banned rather than the individual codepoint,
  because every member of it is equally uncovered and banning one at a time is how the
  second one gets in.

Failure-mode check, before any non-ASCII character reaches a widget: **which of the four
loaded faces has this glyph, and what asserted that?** If the answer is that it looked
fine where it was typed, nothing has been verified.

## A generator never overwrites content it did not generate [R:generate-guards-unversioned]

A generator that writes into a directory shared with hand-authored files must never overwrite a file it did not itself generate. Stamp every generated file with a marker the generator can recognise on the next run, refuse to overwrite any target lacking it, and make the check all-or-nothing: abort the whole write before touching disk if any target is unversioned, rather than leave a half-generated tree. A dropped rule is a lost correction; a clobbered human file is a lost correction the tool itself destroyed.

## Order by an explicit rank, not an incidental string sort [R:order-by-explicit-rank]

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.

## Keep secret-bearing, machine-local config out of a shared config repo [R:no-secrets-in-config-repo]

When a repository versions configuration that is meant to be shared, keep the secret-bearing and machine-local parts out of it: credentials, permission allowlists, and per-machine settings are not shareable configuration and do not belong in shared history, even (especially) when the repo is otherwise a config repo. Version the shareable wiring separately from the secrets, so tracking the former never commits the latter. A secret in history is a secret to rotate, not a secret to delete.

## Verify a moved file is still tracked in a whitelist-gitignore repo [R:verify-tracked-after-move]

After moving or renaming a tracked file in a repository whose .gitignore is a whitelist, verify the file is still tracked before considering the change done. A whitelist ignore silently drops anything outside its re-included paths, so a relocation can remove a file from version control with no error and no diff line to notice. Run the repo's tracking/deploy verification as the gate: the failure mode is invisible precisely when you most assume the move was safe.

<!-- relearn:generated v0.1.0 sha256=c9724e6c26facbb7d12c6642379aeb3f357208f7358827f8665b7bd0cfa90f0a rules=R:case-collision,R:claude-md-recreates-the-project,R:decisions-log-records-rejected-alternatives,R:definition-of-done-every-change,R:delegate-a-fan-out,R:detector-excludes-own-definitions,R:doc-currency,R:features-ledger-names-its-artefact,R:five-files-no-more,R:guarantee-needs-a-reader,R:make-illegal-states-unrepresentable,R:measure-cost-per-task,R:measure-the-claim-not-a-subset,R:names-travel-with-the-quote,R:no-sentinel-values,R:no-silent-spend,R:no-stale-push-over-fresh,R:no-weak-model-for-judgment,R:pin-eol-for-executable-text,R:prefer-by-construction,R:price-every-dependency,R:reconcile-wiring-at-start,R:repair-the-lying-artefact,R:report-the-hit-not-the-match,R:revision-integrity,R:search-before-you-build,R:source-practice-from-its-artefact,R:verdict-survives-the-channel,R:verify-through-production-path,R:wired-artifact,R:a-speculated-path-deoptimises-when-the-input-changes,R:a-view-is-not-a-copy,R:a-wrapper-type-is-not-free-here,R:close-what-you-open,R:design-for-inheritance-or-forbid-it,R:equality-is-one-contract,R:exceptions-name-what-failed,R:identity-is-not-equality-for-boxes,R:no-reference-to-internals-escapes,R:null-is-not-a-value,R:publish-safely-or-not-at-all,R:seal-the-alternatives,R:serializable-is-a-second-constructor,R:a-measurement-matches-the-regime-it-reports,R:a-queue-without-a-bound-has-no-overload-behaviour,R:a-structure-keeps-the-regime-it-was-proved-under,R:allocated-is-not-resident,R:answer-the-requirement-at-its-layer,R:attack-the-design-in-a-second-pass,R:no-allocation-on-the-hot-path,R:no-coordinated-omission,R:no-false-sharing-on-a-hot-line,R:no-retry-loop-on-a-contended-path,R:no-stall-inside-a-publication-window,R:no-syscall-on-a-bounded-path,R:profiler-samples-where-it-can-stop,R:transient-state-is-not-a-terminal-state,R:verify-ordering-on-the-weakest-target,R:async-all-the-way,R:borrow-in-signatures,R:design-types-first,R:errors-name-what-failed,R:justify-every-clone,R:module-visibility-is-deliberate,R:must-use-on-consequential-returns,R:newtype-liberally,R:no-anyhow-in-libraries,R:no-unwrap-in-production,R:parse-dont-validate,R:parse-wide-then-range-check,R:private-fields-only,R:seal-closed-trait-sets,R:typestate-builder-for-required-fields,R:typestate-for-protocols,R:verify-the-abstraction-compiled-away,R:xplat-fixtures,R:role-is-an-edge-property,R:seeded-data-needs-a-migration,R:verify-the-glyph-exists,R:generate-guards-unversioned,R:order-by-explicit-rank,R:no-secrets-in-config-repo,R:verify-tracked-after-move -- DO NOT EDIT; regenerate with `relearn build` -->
