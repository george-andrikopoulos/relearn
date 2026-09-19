---
name: project-design-architecture-tool
description: "Engineering discipline for the design-architecture-tool repository. Covers: role is an edge property; seeded data needs a migration; verify the glyph exists"
---

# project: design-architecture-tool rules

## A role belongs to an edge endpoint, never to the thing at the end of it [R:role-is-an-edge-property]

> Also enforced by test:the_repository_derives_no_queue_role_from_a_spin_mode (documents) + test:spin_mode_does_not_change_a_single_queue_end_letter (renderers).

> Has recurred 2 time(s) since it was written; most recently 2026-08-30.

Read a role from the DIRECTION of an edge, never from a property of a service.

A service that writes into a queue is that queue's producer; one that reads from it is a
consumer. Any service is routinely both at once -- a mid-chain hop consumes from its
inbound queue and produces into its outbound one -- so a role assigned to a whole service
is only ever correct for a pure source or a pure sink, and the diagram is wrong
everywhere else.

Spin mode says how a service WAITS, not what it does with a queue. What it legitimately
says is how often an endpoint pays a socket crossing: a cost, not an identity. That is
`SpinCost` and `HandoffSeverity` in `shared/src/model/handoff.rs`, and neither type can
express a role, so neither can drift back into asserting one. Making the wrong statement
unrepresentable is what ended this, after prose had failed three times.

The recurrences are the lesson, not the original error. A derivation removed from the
code lives on in the doc comments, and removed from the doc comments lives on in the
domain-knowledge section -- each fix landing where the last reader complained rather than
everywhere the claim is made. When a correction is to a *concept*, grep the whole
repository for the claim before calling it fixed, and pin it with a test that reads the
concept rather than the surface: `spin_mode_does_not_change_a_single_queue_end_letter`
would have failed on day one.

## Seed data reaches only the installations that did not exist yet [R:seeded-data-needs-a-migration]

> Also enforced by test:the_catalogue_has_not_grown_without_a_migration_to_carry_it.

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

## A character the UI draws must have a glyph in the faces the app actually loads [R:verify-the-glyph-exists]

> Also enforced by test:client/src/app/font_tests.rs + gate:verify.sh emoji_ban.

Ask the font stack, not the document. A character that renders in an editor, a terminal
or a markdown preview proves nothing about the four faces egui loads at runtime, and the
failure is silent -- an empty box, not an error.

Prose cannot hold this, and this repository has the proof: the rule existed, was read,
and prescribed a codepoint that produced the exact defect it forbade. When the
instruction itself is the defect, re-reading it does not help, which is the argument for
pushing a rule down a layer rather than restating it more firmly.

Two controls now hold it, and they cover different halves:

* `client/src/app/font_tests.rs` asks the REAL `FontDefinitions::default()` -- the very
  stack the app builds -- whether each non-ASCII character the UI draws has a glyph, in
  the proportional and monospace families both. This is the positive half: what the table
  in CLAUDE.md claims is now asserted against the thing it claims about.
* `scripts/verify.sh` `emoji_ban()` blocks two whole codepoint blocks outright in
  non-comment lines under `client/src` -- astral emoji (U+1F000-U+1FAFF) and fullwidth
  forms (U+FF00-U+FFEF). The BLOCK is banned rather than the individual codepoint,
  because every member of it is equally uncovered and banning one at a time is how the
  second one gets in.

Failure-mode check, before any non-ASCII character reaches a widget: **which of the four
loaded faces has this glyph, and what asserted that?** If the answer is that it looked
fine where it was typed, nothing has been verified.

<!-- relearn:generated v0.1.0 sha256=a9995b0696dca541c4989462668451fd4c71757894b65b227db6d9b513bebf08 rules=R:role-is-an-edge-property,R:seeded-data-needs-a-migration,R:verify-the-glyph-exists -- DO NOT EDIT; regenerate with `relearn build` -->
