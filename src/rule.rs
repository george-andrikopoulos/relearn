//! `rule` — the neutral rule type and its newtypes: `RuleTag`, `ErrorClass`,
//! `Incident`, `Home`, `Status`. Parsing of the neutral format (TOML
//! front-matter delimited by `+++`, imperative markdown body) lives here, and
//! validation mints witness newtypes at the perimeter — parse, don't validate,
//! so nothing downstream re-checks.
//!
//! Type-driven decisions this module owns (ARCHITECTURE.md):
//! - `RuleTag` is constructible only via `RuleTag::parse` (shape
//!   `R:[a-z0-9][a-z0-9-]*`); possessing one *is* the proof.
//! - `Home` is `Global | Domain { name } | Project { path }` — one home per
//!   rule is a type-level guarantee, not a lint.
//! - `Status` carries its payload, so "graduated" without a destination or
//!   "attic" without a reason cannot be constructed.
//! - Date fields parse wide, then range-check (`[R:parse-wide-then-range-check]`).
//!
//! **Must NOT:** know anything about output formats. An emitter's concerns
//! never leak into the rule type.

mod date;
mod home;
mod status;
mod tag;
mod text;

pub use date::{Date, DateError};
pub use home::{DomainName, Home, ProjectPath};
pub use status::{Destination, Reason, Status};
pub use tag::{RuleTag, RuleTagError};
pub use text::{EmptyText, ErrorClass, Incident};
