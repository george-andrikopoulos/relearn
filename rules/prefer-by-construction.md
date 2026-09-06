+++
tag = "R:prefer-by-construction"
title = "Prefer by-construction impossibility over after-the-fact controls"
error_class = "Reaching for a runtime control (a check, a guard, a review step) to catch a mistake after it occurs when the design could have made that mistake impossible to express in the first place"
home = { kind = "global" }
created = "2026-08-13"
origin = "mined"
status = { kind = "active" }
incident = "A standing principle in the operating model, tagged [R:prefer-by-construction] there. Its concrete realizations recur: R:make-illegal-states-unrepresentable is the type-level form, and the repo-in-place design (editing a file IS editing the live system, so there is no deploy step to forget) removes a class of staleness by construction rather than guarding against it. The weaker the layer a guarantee lives at, the more it must be remembered; by-construction is the layer that cannot be forgotten."
+++

When a class of mistake can be designed out, design it out, rather than adding a control that catches it after the fact. A control that catches a mistake still admits the mistake; a design that cannot express the mistake retires the whole class. Rank the options by how little must be remembered for them to hold: a type the compiler enforces beats a test that samples beats a review step that relies on attention. R:make-illegal-states-unrepresentable is this rule in the type system.
