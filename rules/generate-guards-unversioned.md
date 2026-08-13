+++
tag = "R:generate-guards-unversioned"
title = "A generator never overwrites content it did not generate"
error_class = "A code or artifact generator overwriting a target file it did not create, destroying hand-authored content that was never generated output"
home = { kind = "project", path = "relearn" }
created = "2026-08-13"
status = { kind = "active" }
incident = "relearn (2026-08-13): emitters write into a target directory that can also hold hand-authored files. Without a guard, a rebuild would clobber a human's file that happens to sit at a generated path. The write side marks every emitted file with a generated-by header and refuses, in a pre-flight pass, to overwrite any existing target lacking that marker -- aborting the whole run rather than leaving a half-generated tree. `relearn verify` is the read-only complement that detects drift after the fact."
+++

A generator that writes into a directory shared with hand-authored files must never overwrite a file it did not itself generate. Stamp every generated file with a marker the generator can recognise on the next run, refuse to overwrite any target lacking it, and make the check all-or-nothing: abort the whole write before touching disk if any target is unversioned, rather than leave a half-generated tree. A dropped rule is a lost correction; a clobbered human file is a lost correction the tool itself destroyed.
