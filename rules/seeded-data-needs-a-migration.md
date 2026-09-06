+++
tag = "R:seeded-data-needs-a-migration"
title = "Seed data reaches only the installations that did not exist yet"
error_class = "New rows, defaults or catalogue entries shipped through a one-time seeder or an install-time path, so the change reaches every fresh installation and no existing one. The code is correct, the tests pass because a test builds its database from empty, and the thing in front of the user does not change"
home = { kind = "project", path = "design-architecture-tool" }
created = "2026-08-27"
origin = "mined"
status = { kind = "graduated", to = "test:the_catalogue_has_not_grown_without_a_migration_to_carry_it", date = "2026-08-27" }
incident = "M21, 2026-08-27. Schema v5 created `ipc_library` and seeded it from `builtin_library()` -- once. M21 then added fifteen rows to the built-in catalogue. Every fresh database got them and every existing database did not: the code was right, the suite was green, and the library in the window did not change. It was found by running the real binary against a v5 database, never by a test, because `test_db()` migrates from v0 and therefore only ever exercises the fresh-install path."
+++

Adding a row to the built-in catalogue under `shared/src/model/ipc/catalog/` is **not**
a code change on its own. It needs a new version function in `server/src/db/migrations.rs`
that calls `seed_builtin_library`, and a bump of `SCHEMA_VERSION`, in the same change.
Do not merely update the count -- add the version first.

The reason it is easy to miss is that every test agrees with you. `test_db()` builds its
schema by running `migrate()` from v0, so a test database is always a *fresh install* and
the upgrade path has no test unless one is written deliberately. Pin the upgrade, not
just the outcome: `v9_carries_the_chronicle_event_loop_row_into_an_existing_database` is
the shape -- open a database at the previous version, migrate it, and assert the row
arrived.

Seed on a version bump, never on every open. Seeding on open would resurrect every row a
user deleted, on every restart, and deletion would stop meaning anything -- which is why
`ipc_library_deleted` exists and why the seeder's insert carries `WHERE NOT EXISTS`
against it. A migration must respect a deletion; `restore-builtins` is the route that
exists to undo one, and it clears the tombstones first.

Failure-mode check: **which databases does this change actually reach?** If the answer is
"the ones created after it ships", the migration is missing and the suite will not tell
you.

`SEEDED_CATALOGUE_ROWS` fails the moment the row count moves without a version to carry
it, so the omission cannot ship quietly. That check exists because the prose version of
this rule did not hold.
