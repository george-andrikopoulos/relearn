+++
tag = "R:detector-excludes-own-definitions"
title = "A detector excludes its own definitions from its scan"
error_class = "A check whose subject matter is text it must itself contain matching on its own source, comments, or documentation, so it is always red -- and an always-red check is muted, leaving the system looking guarded by something that no longer reports"
home = { kind = "global" }
created = "2026-07-22"
origin = "mined"
status = { kind = "active" }
incident = "2026-07-22, twice within one hour. A model-reach detector flagged all six of the files it protected, because each contained the config-directory path the detector matched on. A deploy verifier flagged its own docstring, which described the pattern it was looking for. Neither found a real defect; both produced a red run that had to be reasoned past, which is the first step toward a run nobody reads. Ported into this library 2026-08-25."
+++

Any check whose subject matter is text it must itself contain -- a linter, a secret scanner, a policy grep, a rules sweep -- matches itself by construction. Exclude the detector's own source, comments, and docstrings from its scan, strip comments before matching, and normalise paths to placeholders so documentation *about* a pattern is never read as an instance of it.

The reason is not tidiness. An always-red check gets muted, and a muted check is worse than no check, because the system still looks guarded. Alarm fatigue is negative value, not neutral: it spends the attention that a real finding will need, and it trains the reader to skip exactly the output that will one day matter.

This extends past the detector's own file to anything that quotes the pattern for a legitimate reason. A correction note explaining that a bad string was removed, written with the bad string in it, becomes a permanent false positive in the very sweep it was written to satisfy -- so describe the string, do not reproduce it, and where reproduction is genuinely necessary put it somewhere the scan excludes by rule rather than by luck.

Ask before shipping any scan: *does this check appear in its own corpus, and if so what does its output look like on a clean tree?* If the clean-tree output is not empty, the check does not yet work, however correct its logic.
