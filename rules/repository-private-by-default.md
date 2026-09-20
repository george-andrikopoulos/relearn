+++
tag = "R:repository-private-by-default"
title = "A repository is created private; going public is an explicit decision"
error_class = "A repository created with public visibility by default, so publication happens by omission rather than by decision -- and unlike privacy, publication has no undo, because clones and mirrors do not take a later visibility change with them"
home = { kind = "domain", name = "disclosure" }
created = "2026-09-20"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written down 2026-09-20 from a standing practice that had lived in a conditionally-loaded skill alongside two disclosure rules that were demonstrably not in force, and was rehomed with them. The practice predates the corpus: every repository is created private and going public is an explicit decision, with one sanctioned public exception maintained deliberately. It is codified rather than mined because no incident is needed to see the asymmetry -- publishing is a one-way door and privacy is not -- and because the two rules it was rehomed beside supply the evidence for what a conditionally-loaded policy is worth."
+++

A new repository is created **private**. Going public is a decision somebody takes, states and can point at.

**The asymmetry is the whole argument.** Making a private repository public is reversible in the only sense that matters least — the button flips back, and the clones, forks, mirrors and archived crawls do not. Making a public repository private does not unpublish it. So the two defaults are not symmetric choices with different odds; one of them has an undo and the other has not.

**Default-public is publication by omission**, and by the time it is noticed the decision has already been taken by whoever typed the create command fastest. Pass the flag explicitly at creation rather than auditing visibility afterwards.

**An exception is named, not inferred.** Where a repository is genuinely meant to be public, say which one and why, so that "this one is public" is a fact a reader can check rather than a state somebody assumes was intended.

**Before any visibility change, the history is the artefact.** A repository about to go public publishes every commit tree, message, branch name and tag, not the tip. `[R:employer-identity-not-in-public-artefacts]` is what to check for, and a detector that has no list must fail rather than pass.

Failure-mode check, at creation: *did I choose this visibility, or accept it?*
