+++
tag = "R:make-illegal-states-unrepresentable"
title = "Make illegal states unrepresentable"
error_class = "Designing types that permit contradictory or invalid states -- a bool beside an Option that can disagree, two fields that can contradict -- so the logic must defensively guard what the type should have forbidden"
home = { kind = "global" }
created = "2026-07-16"
origin = "mined"
status = { kind = "active" }
incident = "The foundation of the type-driven discipline; the recurring class is a type that admits states the logic must remember to guard. Its corollary, R:no-sentinel-values, was realized concretely by the mesh-watchdog incident (2026-07-16), where a stopped service mapped to a zero sentinel suppressed the recovery the tool existed for."
published_incident = "The recurring class is a type that admits states the surrounding logic must remember to guard, so every guard is a place the next reader can forget. Its sharpest instance was a service's stopped state encoded as a zero value of an existing numeric type: a downstream gate read that zero as an ordinary measurement, compared it against a minimum, and suppressed forever the recovery the system existed to perform."
+++

Before writing logic, design the types so invalid states cannot be constructed: sum types over boolean flags, one field that cannot contradict another. If two fields can disagree, redesign until they cannot. R:no-sentinel-values is the corollary -- an absent or stopped state is an enum variant, not a magic value the surrounding logic must remember to special-case.
