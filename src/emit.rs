//! `emit` — one submodule per target assistant. Each emitter is a pure
//! function `&Library<Validated> -> Vec<OutputFile>`: it is handed already
//! validated rules and returns values for the caller (`fsio`, via `cli`) to
//! write. Emitters never perform the write themselves, which is what makes
//! them trivially unit-testable without a filesystem.
//!
//! Every emitted file carries a generated-by header naming its source rule(s)
//! and a content hash — the string the overwrite guard checks.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the
//! rule. If a target needs data, the data belongs in the rule, not in the
//! emitter.

pub mod agents;
pub mod claude;
pub mod claude_md;
pub mod copilot;
pub mod cursor;
