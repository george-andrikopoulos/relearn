//! `library` — collection semantics over rules: tag uniqueness across the
//! library, home partitioning, and the `Library<Unvalidated>` /
//! `Library<Validated>` typestate that gates emission. A duplicate tag is a
//! library-level error; a validated library is the witness that every rule
//! parsed and every cross-rule invariant held.
//!
//! **Must NOT:** perform I/O. The library is a pure in-memory value; reading
//! rules from disk and writing emitted files belong to `fsio`.
