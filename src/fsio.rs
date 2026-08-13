//! `fsio` — all filesystem reads (loading `rules/*.md`) and writes (emitting
//! target files), including the generated-file guard: it refuses to overwrite
//! a target that lacks the generated-by header and matching hash, atticking or
//! aborting rather than deleting unversioned content
//! (`[R:generate-guards-unversioned]`).
//!
//! **Must NOT:** contain business logic. It moves bytes and enforces the
//! overwrite guard; every decision about *what* to write is made upstream in
//! `emit`/`library` and handed here as values.
