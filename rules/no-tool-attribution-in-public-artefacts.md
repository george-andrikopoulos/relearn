+++
tag = "R:no-tool-attribution-in-public-artefacts"
title = "Tool attribution is off by decision and verified, never off by assumption"
error_class = "Authorship or attribution metadata a tool inserts by default reaching a published artefact -- a commit trailer, a generated header, a document property -- because the default was assumed off rather than read, so a disclosure policy is breached by omission and the artefact is public before anyone notices"
home = { kind = "domain", name = "disclosure" }
created = "2026-09-20"
origin = "mined"
status = { kind = "active" }
incident = "2026-09-20. A standing house rule forbids tool attribution in commits, tracked files and release artefacts. Measured across two repositories on this machine: 72 commits carry an assistant `Co-Authored-By` trailer and 67 of them are already pushed. The rule was not weakly enforced; it was not in force at all, for two reasons that compound. First, it lived only in a skill that loads when a particular vault workflow loads, so it was absent from every session that did not touch that vault -- a configuration-dependent home for an organisation-level disclosure policy. Second, and worse, the always-loaded layer carried a sentence reading \"Attribution disabled globally via settings.json\", and that setting does not exist in the file: the key is absent, the default is on, and nothing ever read the state the sentence asserted. An audit on 2026-08-16 had recorded zero such trailers, which was true of that day and was then carried forward as though it described the configuration. Both halves were found only because a rehoming task made someone open the file the sentence named. The commits cannot be unpublished; history rewriting is the only remedy for the 67 and it is not a cheap one."
+++

Attribution a tool inserts on your behalf is **off by decision and verified**, never off by assumption.

**The default is on, and the artefact is published.** Commit trailers, generated-file headers, document properties and export metadata are written by tooling that defaults to identifying itself. Where a disclosure policy forbids that, the policy is not satisfied by anybody intending to comply: it is satisfied by the setting being off, and by someone having read the setting rather than a sentence about it.

**A note saying it is disabled is not the disabling.** The failure mode is specific and it is worse than simple omission: a line in an always-loaded document asserting *"attribution is disabled globally in the config"* ends the inquiry for every reader who meets it, including the reader who would otherwise have checked. `[R:guarantee-needs-a-reader]` is the general form. Name the key, and read the key.

**Check the artefact, not the intention.** `git log --grep` over the real history, the generated header in a real output file, the document properties of a real export. A policy of this kind is falsifiable in one command, and until that command has been run the compliance position is unknown rather than good.

**An audit is a measurement of a day, not a description of a configuration.** "Zero found on the 16th" is a fact about the 16th. It becomes a false claim the moment it is carried forward as though it described a setting `[R:check-the-claim-you-inherit]`.

**Publication is the boundary that matters.** A trailer in an unpushed commit is an edit; the same trailer pushed is a disclosure that deletion does not undo, because clones, forks and mirrors do not take the deletion with them. Fix the default before the next push, and treat the already-published set as a separate decision with its own cost.

Failure-mode check, for any tool that can sign its own work: *what would I see if this were on, and have I looked at that rather than at a document about it?*
