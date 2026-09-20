+++
tag = "R:employer-identity-not-in-public-artefacts"
title = "An employer name, internal product name or role title never reaches a public artefact"
error_class = "An employer's name, an internal product name or a role title reaching an artefact with a public audience -- a repository and its history, a paper, a talk, an issue -- where it is a disclosure exposure rather than a detail, and where deleting it later does not reach the clones that already have it"
home = { kind = "domain", name = "disclosure" }
created = "2026-09-20"
origin = "mined"
status = { kind = "active" }
incident = "2026-09-05, and the reason this is a rule rather than a habit. An audit of a repository intended for publication found an employer-owned product name in the working tree and in 77 of its 106 commit trees. A sweep six weeks earlier had recorded an all-clear for exactly that question, and the all-clear was believed precisely because it was written down. The standing policy -- never an employer name, an internal product name or a role title in a public artefact -- existed the whole time, in a skill that loads only when a particular vault workflow loads, so it was absent from every session that did not touch that vault. An organisation-level disclosure policy homed behind a conditional load is a policy that applies on the days nobody needed it. The working tree was fixable in an afternoon; the 77 commit trees were not, because history is what publication actually distributes and a later deletion does not reach a clone."
+++

An employer's name, an internal product name and a role title never reach an artefact with a public audience.

**The artefact is wider than the file.** A repository publishes its history, not its working tree: branch names, commit messages, commit *trees*, tags, issue text and release notes all travel, and a name removed from `main` today is still in the clone somebody took last month. Fix the working tree if you like, but the honest question is what the history contains.

**The protected list is machine-local, and that is not an oversight.** A committed list of terms to avoid is a published list of the things being protected — and a short name normalises to a handful of characters, so publishing the salt publishes the name. Keep the list out of the repository and have the gate read it from outside. A gate with no list must **fail**, not pass: a disarmed detector reporting clean is worse than no detector, because it ends the inquiry `[R:guarantee-needs-a-reader]`.

**Report a location and a length, never the match.** The output of a search for a protected name is another copy of it — in the matched text, a context line, a path whose last component *is* the name, or an error message. `[R:report-the-hit-not-the-match]` is the authority; this rule is the reason it matters here.

**This policy does not belong in a conditionally-loaded home.** A disclosure rule that is only in force when some unrelated workflow happens to load is in force on the days you did not need it. It belongs in the layer that loads for every session in every repository, and a pointer is what the specialised home should keep `[R:one-home-per-rule]`.

**Quoting is the leak nobody plans.** An incident description, a paper, a talk, a bug report — each is a place where text is copied *out* of the repository that was protecting it and into one whose detector has never heard of the term. `[R:names-travel-with-the-quote]`.

Failure-mode check, before any artefact becomes public: *does the history contain it, and which repository's detector covers the destination?* If the answer to the second is the repository the text came from, nothing covers where it is going.
