+++
tag = "R:parse-wide-then-range-check"
title = "Parse wide, then range-check"
error_class = "Range-validating by parsing straight into the target narrow type, so the out-of-range case is unreachable for the very value it exists to name"
home = { kind = "domain", name = "rust" }
created = "2026-07-22"
origin = "mined"
status = { kind = "active" }
incident = "Grouping task 01 (2026-07-22): 5/5 raw samples and 4/5 with skills loaded parsed into u16, so 70000 returned NotANumber; the declared OutOfRange variant was reachable only for 0. One sample documented the bug in its own rustdoc."
published_incident = "A range check was written by parsing straight into the narrow target type, so an out-of-range value could not be represented long enough to be reported as out of range: the input returned not-a-number instead, and the declared out-of-range variant was reachable for almost nothing. Measured across repeated attempts at the same task, nearly every one made the same mistake, and one of them documented the resulting bug in its own generated documentation without noticing."
+++

Parse into a type wide enough to *represent* the out-of-range value, then range-check to mint the narrow newtype. The perimeter must be able to see the illegal value in order to name it illegal; parsing directly into the target type collapses "out of range" into "not a number" and makes the OutOfRange class a lie the compiler will not catch.
