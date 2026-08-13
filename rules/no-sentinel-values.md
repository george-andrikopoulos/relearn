+++
tag = "R:no-sentinel-values"
title = "No sentinel values: absent states are enum variants"
error_class = "Encoding a distinct state as a magic value of an existing type (0, -1, \"\", T::zero()) that downstream logic must remember to special-case"
home = { kind = "global" }
created = "2026-07-16"
status = { kind = "active" }
incident = "mesh-watchdog (2026-07-16): a stopped Windows service was mapped to ServiceAge::zero(); the anti-thrash gate read 0 < min_age as too-young and suppressed the restart forever -- the exact recovery the tool existed for. Every per-task review passed it; only the whole-branch review against FEATURES.md caught it. Fix: enum ServiceLiveness { Stopped, Running(ServiceAge) }."
+++

If "absent / stopped / unknown" is a real state, make it an enum variant, not a magic value of an existing type. Downstream code will forget to special-case a sentinel; it cannot forget a variant the compiler forces it to handle. If a range check reads a sentinel as a real quantity, it fails in the direction of the sentinel, not of safety.
