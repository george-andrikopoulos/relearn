+++
tag = "R:verify-through-production-path"
title = "Verify through the production path"
error_class = "Verifying a feature through a stand-in wiring (dev override, mock transport, alternate config channel) instead of the exact channel production uses"
home = { kind = "global" }
created = "2026-07-16"
origin = "mined"
status = { kind = "active" }
incident = "ha-mcp v0.7.0 (2026-07-16): the IP allowlist passed all smoke gates via a dev OPTIONS_FILE override, while the real run.sh -> ALLOWED_CIDRS handoff produced non-JSON (bashio prints lists newline-separated) -- the headline security feature would have shipped silently disabled (fail-open). Caught in pre-deploy review, not smoke."
+++

Before declaring anything verified, run at least one check through the exact channel production uses: same env var, same startup script, same config file, same transport. A test that exercises a stand-in is evidence the stand-in works, not that the feature does. Ask: which line of production wiring did my test NOT execute? That line is where it breaks.
