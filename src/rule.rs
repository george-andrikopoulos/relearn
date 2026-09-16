//! `rule` — the neutral rule type and its newtypes: `RuleTag`, `ErrorClass`,
//! `Incident`, `Home`, `Status`, and the `Rule` aggregate. Parsing of the
//! neutral format (TOML front-matter delimited by `+++`, imperative markdown
//! body) lives here, and validation mints witness newtypes at the perimeter —
//! parse, don't validate, so nothing downstream re-checks.
//!
//! Type-driven decisions this module owns (ARCHITECTURE.md):
//! - `RuleTag` is constructible only via `RuleTag::parse` (shape
//!   `R:[a-z0-9][a-z0-9-]*`); possessing one *is* the proof.
//! - `Home` is `Global | Domain { name } | Project { path }` — one home per
//!   rule is a type-level guarantee, not a lint.
//! - `ScopeTag` is the *audience* a rule serves, and is deliberately a separate
//!   field from `Home`: ownership has exactly one answer, audience has several.
//!   A scope never affects where a rule is emitted, only whether it is.
//! - `Status` carries its payload, so "graduated" without a destination or
//!   "attic" without a reason cannot be constructed.
//! - Date fields parse wide, then range-check (`[R:parse-wide-then-range-check]`).
//! - `Rule` aggregates witness newtypes; because each field is a distinct type,
//!   its constructor is swap-proof.
//!
//! **Must NOT:** know anything about output formats. An emitter's concerns
//! never leak into the rule type.

mod authority;
mod control;
mod date;
mod def;
mod home;
mod origin;
mod parse;
mod scope;
mod serialize;
mod status;
mod tag;
mod text;

pub use authority::{
    AdoptError, Authority, CachedIsNotEditable, DroppableCache, EditableRule, NotDroppable,
    NotPullable, Provenance, PulledRule, SourceId, Unwanted, Version,
};
pub use control::{Control, ControlError, ControlKind, Controls};
pub use date::{Date, DateError};
pub use def::{Recurrence, Rule};
pub use home::{DomainName, Federation, Home, OrgName, ProjectPath};
pub use origin::{Approval, Approver, ControlRef, Origin, OriginError, RecurrenceRole};
pub use parse::{ParseError, parse_document};
pub use scope::{ScopeTag, ScopeTagError};
pub use serialize::to_document;
pub use status::{Emittability, ProseCoverage, Reason, Status, StatusError, Uncovered};
pub use tag::{RuleTag, RuleTagError};
pub use text::{Body, EmptyText, ErrorClass, Incident, PublishedIncident, Title};
