+++
tag = "R:one-home-per-rule"
title = "One home per rule; every other copy is a reference or is generated"
error_class = "The same rule, fact or index kept as an original in more than one place, so a correction applied to one copy silently leaves the others authoritative -- and where the duplicate is an INDEX rather than a body, the stale copy is a strict subset that reads as correct and never contradicts anything"
home = { kind = "global" }
created = "2026-09-19"
origin = "mined"
status = { kind = "active" }
incident = "2026-07-22, the elevation of a configuration directory to a governed repository. The same instruction layer was found to exist in three places at once: the live directory, a second checkout kept \"for reference\", and a backup tree. 1.85 GB was deleted and six memory files quarantined before one copy could be called the source. Nothing had announced the fork, because nothing could: each copy was individually well-formed, and the only evidence of duplication was that a correction applied to one of them did not show up in a session reading another. Recorded as \"P2, one home per rule\" in a framework document, cited by tag in six places since, and defined nowhere -- the rule governing the library was the one rule the library did not contain. Written down 2026-09-19 after the class recurred in a new shape that the original framing would not have caught: a transcript miner held a hand-written list of WHICH RULES EXIST beside the corpus that defines them, drifted to 23 rows against 84 with nothing failing, and five rules a deterministic hook already held were invisible to the quarterly review that decides what gets retired. The second home was not a copy of the rules; it was a copy of the INDEX, which is why reading either one looked correct."
+++

Every rule, fact or definition has exactly one home, and every other place that needs it holds a **reference or a generated copy**, never a second original.

**The test is not "are these files identical" but "if this changed, how many places would I have to edit".** More than one is a fork already, whether or not the copies have drifted yet. Drift is the symptom that makes a fork visible; it is never what makes it a fork.

**An index is a second home too, and it is the one that hides.** A list of what exists -- which rules, which files, which tests, which hooks -- read beside the thing that defines them is a duplicate of the most load-bearing fact in the system, and it fails silently in a way a duplicated body does not. Two copies of a rule's TEXT eventually contradict each other and someone notices. Two copies of the INDEX never contradict: the stale one is a strict subset, so every entry it holds is correct and the reader sees a coherent, smaller world. Ask of any list: *what would it look like if this were out of date?* If the answer is "exactly like this, only shorter", nothing can tell you which you are looking at.

**Prefer deriving to synchronising.** A generated copy is not a second home: it is an artefact with a stated source, regenerable, and safe to delete. A synchronised copy is a second home with a chore attached, and the chore is what stops being done. If a consumer needs the data, give it a way to ask rather than a copy to keep.

**Where a second original is genuinely unavoidable,** say so at both sites, name which one is authoritative, and give the copy a check that fails when it diverges. That is worse than one home and much better than two that both look right.

Failure-mode check, before adding any list, table, registry or inventory: *does something else already know this, and can I read it from there instead?*

`[R:doc-currency]` is the downstream half -- when the source moves, the descriptions of it move in the same change. This rule is upstream of that: it asks why a second description existed to go stale.
