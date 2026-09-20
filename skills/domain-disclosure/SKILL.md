---
name: domain-disclosure
description: "Engineering discipline for working in disclosure. Covers: employer identity not in public artefacts; names travel with the quote; no tool attribution in public artefacts; repository private by default; report the hit not the match"
---

# domain: disclosure rules

## An employer name, internal product name or role title never reaches a public artefact [R:employer-identity-not-in-public-artefacts]

An employer's name, an internal product name and a role title never reach an artefact with a public audience.

**The artefact is wider than the file.** A repository publishes its history, not its working tree: branch names, commit messages, commit *trees*, tags, issue text and release notes all travel, and a name removed from `main` today is still in the clone somebody took last month. Fix the working tree if you like, but the honest question is what the history contains.

**The protected list is machine-local, and that is not an oversight.** A committed list of terms to avoid is a published list of the things being protected — and a short name normalises to a handful of characters, so publishing the salt publishes the name. Keep the list out of the repository and have the gate read it from outside. A gate with no list must **fail**, not pass: a disarmed detector reporting clean is worse than no detector, because it ends the inquiry `[R:guarantee-needs-a-reader]`.

**Report a location and a length, never the match.** The output of a search for a protected name is another copy of it — in the matched text, a context line, a path whose last component *is* the name, or an error message. `[R:report-the-hit-not-the-match]` is the authority; this rule is the reason it matters here.

**This policy does not belong in a conditionally-loaded home.** A disclosure rule that is only in force when some unrelated workflow happens to load is in force on the days you did not need it. It belongs in the layer that loads for every session in every repository, and a pointer is what the specialised home should keep `[R:one-home-per-rule]`.

**Quoting is the leak nobody plans.** An incident description, a paper, a talk, a bug report — each is a place where text is copied *out* of the repository that was protecting it and into one whose detector has never heard of the term. `[R:names-travel-with-the-quote]`.

Failure-mode check, before any artefact becomes public: *does the history contain it, and which repository's detector covers the destination?* If the answer to the second is the repository the text came from, nothing covers where it is going.

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

## Tool attribution is off by decision and verified, never off by assumption [R:no-tool-attribution-in-public-artefacts]

Attribution a tool inserts on your behalf is **off by decision and verified**, never off by assumption.

**The default is on, and the artefact is published.** Commit trailers, generated-file headers, document properties and export metadata are written by tooling that defaults to identifying itself. Where a disclosure policy forbids that, the policy is not satisfied by anybody intending to comply: it is satisfied by the setting being off, and by someone having read the setting rather than a sentence about it.

**A note saying it is disabled is not the disabling.** The failure mode is specific and it is worse than simple omission: a line in an always-loaded document asserting *"attribution is disabled globally in the config"* ends the inquiry for every reader who meets it, including the reader who would otherwise have checked. `[R:guarantee-needs-a-reader]` is the general form. Name the key, and read the key.

**Check the artefact, not the intention** -- the real history, the generated header in a real output file, the document properties of a real export. A policy of this kind is falsifiable in one command, and until that command has been run the compliance position is unknown rather than good.

**Count the field, not the text, or the count is wrong the moment you write about it.** A search for the literal string matches prose that *mentions* it -- this rule, the commit that fixed it, the incident note -- so the instrument inflates exactly as the subject gets discussed `[R:detector-excludes-own-definitions]`. Ask git for the parsed trailer instead of grepping the message:

```
git log --format='%(trailers:key=Co-Authored-By)' | grep -c 'Co-Authored-By:'   # the field
git log --grep='Co-Authored-By' --oneline | wc -l                              # the text: over-counts
```

Measured on the day this rule was written, the two disagreed by two and the gap was widening with every commit describing the problem.

**An audit is a measurement of a day, not a description of a configuration.** "Zero found on the 16th" is a fact about the 16th. It becomes a false claim the moment it is carried forward as though it described a setting `[R:check-the-claim-you-inherit]`.

**Publication is the boundary that matters.** A trailer in an unpushed commit is an edit; the same trailer pushed is a disclosure that deletion does not undo, because clones, forks and mirrors do not take the deletion with them. Fix the default before the next push, and treat the already-published set as a separate decision with its own cost.

Failure-mode check, for any tool that can sign its own work: *what would I see if this were on, and have I looked at that rather than at a document about it?*

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

## A repository is created private; going public is an explicit decision [R:repository-private-by-default]

A new repository is created **private**. Going public is a decision somebody takes, states and can point at.

**The asymmetry is the whole argument.** Making a private repository public is reversible in the only sense that matters least — the button flips back, and the clones, forks, mirrors and archived crawls do not. Making a public repository private does not unpublish it. So the two defaults are not symmetric choices with different odds; one of them has an undo and the other has not.

**Default-public is publication by omission**, and by the time it is noticed the decision has already been taken by whoever typed the create command fastest. Pass the flag explicitly at creation rather than auditing visibility afterwards.

**An exception is named, not inferred.** Where a repository is genuinely meant to be public, say which one and why, so that "this one is public" is a fact a reader can check rather than a state somebody assumes was intended.

**Before any visibility change, the history is the artefact.** A repository about to go public publishes every commit tree, message, branch name and tag, not the tip. `[R:employer-identity-not-in-public-artefacts]` is what to check for, and a detector that has no list must fail rather than pass.

Failure-mode check, at creation: *did I choose this visibility, or accept it?*

<!-- relearn:generated v0.1.0 sha256=0dc11a2a12b58d744b6fd5f2fcd00656e1840dfd9e5802be5898dce852257201 rules=R:employer-identity-not-in-public-artefacts,R:names-travel-with-the-quote,R:no-tool-attribution-in-public-artefacts,R:report-the-hit-not-the-match,R:repository-private-by-default -- DO NOT EDIT; regenerate with `relearn build` -->
