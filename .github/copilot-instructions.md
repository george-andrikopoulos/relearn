# Copilot instructions

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

> Has recurred 1 time(s) since it was written; most recently 2026-09-05.

Every safety claim in prose names the line, test, or check that enforces it -- or the sentence is deleted. A guarantee with no reader is worse than no guarantee, because it is read as coverage and it ends the inquiry.

A claim of completeness must also state what the check actually consumed. Not "verified", but "verified by grepping `model:` across N agent files" -- so the gap between the scope of the check and the scope of the claim is visible on the face of the entry rather than reconstructable only by rerunning it. Most false completeness claims are not lies; they are a narrow check reported in wide language.

**Error paths are where these hide.** A message asserting a state must be produced by *checking that state*, not by which branch printed it. The failure path is the one nobody exercises, so a confident sentence with nothing behind it survives there longest, and it is read at exactly the moment the reader is least able to question it. Ask of every error message: *did anything read the world before this printed?*

Where a mechanical reader exists, use it and leave the prose to what it cannot see: an unread binding is already a hard error under a compiler run with warnings denied. What no linter can see is a true-looking sentence with nothing behind it, and that is what this rule is for.

The test: *what would have to be true for this sentence to be false, and what reads that?* R:wired-artifact is the neighbouring failure -- there a check exists and accepts forgeable evidence; here there is no check at all.

## Make illegal states unrepresentable [R:make-illegal-states-unrepresentable]

Before writing logic, design the types so invalid states cannot be constructed: sum types over boolean flags, one field that cannot contradict another. If two fields can disagree, redesign until they cannot. R:no-sentinel-values is the corollary -- an absent or stopped state is an enum variant, not a magic value the surrounding logic must remember to special-case.

## Measure cost-per-completed-task; never choose by price tier [R:measure-cost-per-task]

Do not pick a mechanism or model by reputation or sticker price. State what it actually costs to complete the task -- tokens consumed times price, including retries -- and choose on that measured cost. After choosing, run one failure-mode check: under what configuration does this cause the exact harm it was chosen to prevent? Then bound that configuration.

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

## Verify a zero-cost claim; never assert it [R:verify-the-abstraction-compiled-away]

"Zero-cost" is a property of a particular abstraction, under a particular optimiser, on a particular build profile. In Rust it is usually true, which is precisely why it gets asserted instead of checked, and why the cases where it is false survive review.

Where the cost is load-bearing -- a hot path, a latency budget, an allocation-free claim -- look at what was emitted. `cargo asm` for the function, `cargo bloat` for the binary, a benchmark on the profile that actually ships. A debug build proves nothing about a release binary: a newtype that is free at `opt-level = 3` need not be at `opt-level = 0`, and the two are different programs.

Then run the failure-mode check R:measure-cost-per-task states in the general case -- under what configuration does this cost exactly what it was chosen not to cost? A `#[repr(transparent)]` newtype crossing an FFI boundary, an iterator chain that fails to fuse because the closure captures by reference, a generic that is never monomorphised because it went out through a trait object. Bound that configuration, or drop the claim.

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

<!-- relearn:generated v0.1.0 sha256=818374325fe513716e5b2a46a5ae9386b7dfa761e37d1c285700712e4bfb939b rules=R:case-collision,R:claude-md-recreates-the-project,R:decisions-log-records-rejected-alternatives,R:definition-of-done-every-change,R:detector-excludes-own-definitions,R:doc-currency,R:features-ledger-names-its-artefact,R:five-files-no-more,R:guarantee-needs-a-reader,R:make-illegal-states-unrepresentable,R:measure-cost-per-task,R:names-travel-with-the-quote,R:no-sentinel-values,R:no-silent-spend,R:no-stale-push-over-fresh,R:no-weak-model-for-judgment,R:pin-eol-for-executable-text,R:prefer-by-construction,R:price-every-dependency,R:reconcile-wiring-at-start,R:repair-the-lying-artefact,R:report-the-hit-not-the-match,R:revision-integrity,R:source-practice-from-its-artefact,R:verdict-survives-the-channel,R:verify-through-production-path,R:wired-artifact,R:answer-the-requirement-at-its-layer,R:attack-the-design-in-a-second-pass,R:async-all-the-way,R:borrow-in-signatures,R:design-types-first,R:errors-name-what-failed,R:justify-every-clone,R:module-visibility-is-deliberate,R:must-use-on-consequential-returns,R:newtype-liberally,R:no-anyhow-in-libraries,R:no-unwrap-in-production,R:parse-dont-validate,R:parse-wide-then-range-check,R:private-fields-only,R:seal-closed-trait-sets,R:typestate-builder-for-required-fields,R:typestate-for-protocols,R:verify-the-abstraction-compiled-away,R:xplat-fixtures,R:role-is-an-edge-property,R:seeded-data-needs-a-migration,R:verify-the-glyph-exists,R:generate-guards-unversioned,R:order-by-explicit-rank,R:no-secrets-in-config-repo,R:verify-tracked-after-move -- DO NOT EDIT; regenerate with `relearn build` -->
