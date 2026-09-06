+++
tag = "R:verify-through-production-path"
title = "Verify through the production path"
error_class = "Verifying a feature through a stand-in wiring (dev override, mock transport, alternate config channel) instead of the exact channel production uses"
home = { kind = "global" }
created = "2026-07-16"
origin = "mined"
status = { kind = "active" }
incident = "ha-mcp v0.7.0 (2026-07-16): the IP allowlist passed all smoke gates via a dev OPTIONS_FILE override, while the real run.sh -> ALLOWED_CIDRS handoff produced non-JSON (bashio prints lists newline-separated) -- the headline security feature would have shipped silently disabled (fail-open). Caught in pre-deploy review, not smoke."

[[recurrence]]
date = "2026-09-04"
incident = "Design-Architecture-Tool M34: every ON DELETE CASCADE in the schema did nothing in production for the life of the project, while every test saw them work. `PRAGMA foreign_keys` is a PER-CONNECTION setting that defaults to OFF and is not stored in the file, and it was set inside `migrate_v0_to_v1` -- so it was on for a database the process CREATED and off for one it merely OPENED. `open_db` opens an already-migrated database, where no version function runs. The tests never saw it because `test_db()` migrates from v0 and is therefore always the create path. Found by writing a deletion test against a database opened the way production opens one, not by reading the code. Fixed by moving the pragma into `migrate()` itself: OFF before the ledger, ON after it."
+++

Before declaring anything verified, run at least one check through the exact channel production uses: same env var, same startup script, same config file, same transport. A test that exercises a stand-in is evidence the stand-in works, not that the feature does. Ask: which line of production wiring did my test NOT execute? That line is where it breaks.
