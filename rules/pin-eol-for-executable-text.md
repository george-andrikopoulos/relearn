+++
tag = "R:pin-eol-for-executable-text"
title = "Pin the line endings of text a machine executes or hashes"
error_class = "Leaving line endings to the checkout for a file whose bytes are load-bearing -- a script an interpreter runs, or generated content whose hash is compared -- so a clone on one platform silently produces a file that no longer works or no longer matches"
home = { kind = "global" }
created = "2026-07-22"
origin = "mined"
status = { kind = "active" }
incident = "stochos-lab (2026-07-22): with `core.autocrlf=true` and no `.gitattributes`, a fresh clone rewrote every hook script to CRLF. The shebang line then ended in a carriage return, the interpreter was not found, and the whole deterministic hook layer stopped firing -- with no error attributable to the cause. Recurred in relearn 2026-08-22 in its other form: committing the emitted tree made line endings load-bearing for `verify`, which rehashes each file's body against the hash in its own header, so a Windows checkout would have reported every generated file as hand-edited while Linux stayed green."
published_incident = "With automatic line-ending conversion enabled and no attributes file pinning them, a fresh clone rewrote every hook script to carriage-return line endings. The shebang line then ended in a carriage return, the interpreter was not found, and an entire deterministic hook layer stopped firing — with no error attributable to the cause. The same class returned in another form when an emitted tree was committed: verification rehashes each file's body against a hash in its own header, so a checkout on the other platform would have reported every generated file as hand-edited while the first platform stayed green."

[[detector]]
name = "executable-text-eol-pinned"
engine = "eol-attribute"
files = ["**"]
events = ["repository-gate"]
action = "block"
message = "A tracked file whose first line is a shebang has no pinned line ending, so a checkout may rewrite it to CRLF and the interpreter is then not found. Pin it at rest in .gitattributes (<glob> text eol=lf), rather than teaching anything downstream to forgive the bytes."

[[detector.fixture]]
records = ["scripts/deploy.sh\t#!/usr/bin/env bash\tunspecified\tunspecified"]
expect = "hit"

[[detector.fixture]]
records = ["tools/run\t#!/bin/sh\tset\tcrlf"]
expect = "hit"

[[detector.fixture]]
records = ["scripts/deploy.sh\t#!/usr/bin/env bash\tset\tlf", "README.md\t# A project\tunspecified\tunspecified"]
expect = "clean"

[[detector.fixture]]
records = ["bin/tool\t#!/bin/sh\tunset\tunspecified"]
expect = "clean"
+++

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
