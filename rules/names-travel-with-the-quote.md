+++
tag = "R:names-travel-with-the-quote"
title = "Quoting an incident carries its names past the gate that was holding them"
error_class = "A private identifier -- a product name, a customer, a hostname, a path -- copied out of the repository that protects it and into an artefact with a wider audience: a public rule corpus, a paper, an issue, a talk. The source repository's detector is still correct and still green, because a gate belongs to a repository and the quotation travelled without it"
home = { kind = "global" }
created = "2026-09-06"
origin = "mined"
status = { kind = "active" }
incident = "relearn, 2026-09-06. An employer-owned product name was found in this public repository: once in `rules/repair-the-lying-artefact.md`, in the `incident` field, and twice in `TODO.md`. It was present in all 52 commit trees, in TODO.md since the first commit. Design-Architecture-Tool -- the repository the incident was quoted FROM -- has had a working detector for exactly this name since its M33-A4, salted digests read by `the_repository_contains_no_banned_name` on every push. That gate was green throughout and could not have helped: it scans its own tree. The name did not escape a control, it was carried past one, by the act of writing an incident down somewhere else. Found by a PostToolUse output hook firing on an unrelated read of the rule file, not by any check either repository owned."
+++

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
