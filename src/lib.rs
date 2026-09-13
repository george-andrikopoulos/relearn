//! `relearn` — compile correction-derived rules from one neutral source form
//! out to every AI assistant's native instruction layer (Claude skills, Cursor
//! `.mdc` rules, GitHub Copilot instructions, `AGENTS.md`, plain `CLAUDE.md`).
//!
//! Reference implementation of the error loop in *Tuning the Stochastic
//! Machine* and of the Stochos framework's P1 (persist or perish) and P2 (one
//! home per rule). The rule is the artifact; every emitted format is a build
//! target and is never hand-edited.
//!
//! Pipeline (see `ARCHITECTURE.md`):
//!
//! ```text
//! rules/*.md --parse--> Library<Unvalidated> --validate--> Library<Validated> --emit--> target files
//! ```
//!
//! A typestate boundary between stages makes it impossible for a later stage to
//! receive earlier-stage data: `emit` accepts only `Library<Validated>`, so
//! emitting unvalidated rules does not compile. Each module below restates its
//! own responsibility and its **must not** constraint, so the boundary lives in
//! the code and not only in the architecture doc.
//!
//! Nothing here is implemented yet — this is the scaffold. Implementation
//! begins with the types (TODO.md Phase A), never with an emitter.

pub mod cli;
pub mod contribute;
pub mod emit;
pub mod fsio;
pub mod library;
pub mod lint;
pub mod rule;
pub mod scrub;
