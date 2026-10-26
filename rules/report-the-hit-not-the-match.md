+++
tag = "R:report-the-hit-not-the-match"
title = "Report the hit, never the match"
error_class = "A search for something whose whole problem is that it exists -- a secret, a credential, a banned name, someone's personal data -- printing the thing it found, in any field of its output: the matched text, a context line, an error message, or a path whose last component IS the name"
home = { kind = "domain", name = "disclosure" }
created = "2026-09-05"
origin = "mined"
status = { kind = "partial", by = "hook:banned-name-in-output", uncovered = "a secret that is neither on a banned-terms digest list nor in a file named on the machine's secret-files list -- a generated value the operator never listed, or one that appears JSON-escaped; output of a tool outside the hook's matcher, such as Read; and, for everything the hook does cover, the first printing, because it runs after the tool and can only stop the leak being repeated", date = "2026-10-26" }
incident = "Design-Architecture-Tool M33-A5, 2026-09-05: auditing a repository for an employer-owned product name, a redaction printed `${p%/*}` -- the parent path of each match -- which for a DIRECTORY match is the name itself; the next command printed the paths outright. Two leaks in two commands, during the audit whose entire subject was removing that name, in a repository whose own gate does this correctly by reporting a path and a length and never the match. The correct implementation was open in the same session and did not transfer, because it was read as a property of that artefact rather than of the act."
published_incident = "While auditing a repository to remove a protected name, a redaction printed the parent path of each match instead of the match itself — which for a directory whose own name is the protected one discloses exactly what was being hidden. The next command printed the paths outright. Two disclosures in two commands, during the audit whose entire subject was removing that name, in a repository whose committed detector does this correctly by reporting a location and a length and never the text. The correct implementation was open in the same session and did not transfer, because it had been read as a property of that one artefact rather than of the act of searching for a secret."

[[recurrence]]
date = "2026-10-26"
incident = "Ferridis Windows service survey. Before redeploying a service that serves MCP over loopback HTTP behind a bearer token, the install directory was read to learn how the service was configured, with the stated intent of printing nothing secret -- and the token file was carefully left unprinted. The service wrapper's XML config was then printed whole, and it carried the same token inline as an environment entry. The token reached the transcript and had to be rotated. The redaction covered the file KNOWN to hold the secret and skipped the check the rule names -- for every field about to be printed, can the hidden thing be inside it? -- for a file not known to hold it. Configuration files are where services keep their secrets, so a whole-file print of one is the case the check exists for. The rule was marked graduated to a hook that hashes tool output against a banned-terms list; a generated token is on no list, so the hook could not have fired and the graduation claimed a class its control covers only part of. Status moved to partial with the uncovered part named. The repair afterwards is the rule's own: field names only, and token occurrences reported as counts against the file rather than printed."
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
