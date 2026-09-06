+++
tag = "R:report-the-hit-not-the-match"
title = "Report the hit, never the match"
error_class = "A search for something whose whole problem is that it exists -- a secret, a credential, a banned name, someone's personal data -- printing the thing it found, in any field of its output: the matched text, a context line, an error message, or a path whose last component IS the name"
home = { kind = "global" }
created = "2026-09-05"
origin = "mined"
status = { kind = "graduated", to = "hook:banned-name-in-output", date = "2026-09-05" }
incident = "Design-Architecture-Tool M33-A5, 2026-09-05: auditing a repository for an employer-owned product name, a redaction printed `${p%/*}` -- the parent path of each match -- which for a DIRECTORY match is the name itself; the next command printed the paths outright. Two leaks in two commands, during the audit whose entire subject was removing that name, in a repository whose own gate does this correctly by reporting a path and a length and never the match. The correct implementation was open in the same session and did not transfer, because it was read as a property of that artefact rather than of the act."
+++

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
