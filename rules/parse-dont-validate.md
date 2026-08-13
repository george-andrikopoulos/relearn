+++
tag = "R:parse-dont-validate"
title = "Parse, don't validate"
error_class = "Re-validating already-checked data at interior call sites instead of carrying a witness newtype minted once at the boundary"
home = { kind = "domain", name = "rust" }
created = "2026-07-22"
status = { kind = "active" }
incident = "Codified from the rust-typedd discipline (2026-07-22): validation scattered across call sites drifts, and an interior re-check that disagrees with the boundary check is a latent bug. R:parse-wide-then-range-check is the sharp edge of this same rule."
+++

Transform raw input into a rich domain type at the outermost boundary, producing a witness newtype whose existence proves the check happened. Interior code takes the witness and never re-checks -- one perimeter, one check, enforced everywhere after by the compiler. R:parse-wide-then-range-check sharpens this: the boundary must be able to see the illegal value in order to name it.
