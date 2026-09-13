//! `Home` — where a rule lives. Exactly one home per rule, guaranteed by the
//! sum type: a rule holds a single `Home`, so it cannot carry two, and each
//! variant carries its own data, so a home cannot be half-specified.

use super::text::{EmptyText, nonempty};

/// A domain name, e.g. `rust` — non-empty. Drives target-specific scoping (such
/// as Cursor globs) in the emitters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainName(String);

impl DomainName {
    /// Parse a non-empty domain name.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("domain name", s)?))
    }

    /// The domain name text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// An organisation name, e.g. `acme` — non-empty. Names the layer that carries
/// an organisation's own engineering principles, which is the one home whose
/// rules may never leave the machine (see [`Home::federation`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrgName(String);

impl OrgName {
    /// Parse a non-empty organisation name.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("org name", s)?))
    }

    /// The organisation name text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A project path — non-empty; the root a `Project` rule is scoped to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPath(String);

impl ProjectPath {
    /// Parse a non-empty project path.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("project path", s)?))
    }

    /// The project path text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Where a rule applies. One home per rule is a type-level guarantee.
///
/// Variants are ordered general to specific, which is also
/// [`crate::emit::home_rank`]'s order: everywhere, then one organisation, then
/// one language, then one tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Home {
    /// Applies everywhere.
    Global,
    /// Applies across one organisation — its own engineering principles, which
    /// are nobody else's business. The one home that never federates; see
    /// [`Home::federation`].
    Org {
        /// The organisation, e.g. `acme`.
        name: OrgName,
    },
    /// Applies to a language or technology domain.
    Domain {
        /// The domain, e.g. `rust`.
        name: DomainName,
    },
    /// Applies within a single project tree.
    Project {
        /// The project root the rule is scoped to.
        path: ProjectPath,
    },
}

/// Whether a home lets a rule leave the machine.
///
/// A deliberately closed, two-valued answer rather than a `bool`: a boolean at a
/// call site says nothing about which way round it runs, and this is the one
/// answer in the codebase where getting it backwards publishes something
/// private.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Federation {
    /// May be contributed upstream, and may appear in a recurrence report.
    Publishable,
    /// Never leaves this machine, in either flow.
    Withheld,
}

impl Home {
    /// Whether a rule in this home may leave the machine.
    ///
    /// **An exhaustive match, and that is the enforcement.** Adding a home
    /// variant without deciding its federation behaviour is a compile error
    /// here, rather than a rule quietly inheriting whichever default a
    /// catch-all arm happened to name. There is no `_` arm and there must never
    /// be one — `tests/federation_exclusion.rs` reads this function's source to
    /// say so, because a catch-all is the one regression the compiler cannot
    /// see. `[R:prefer-by-construction]`
    ///
    /// Two homes are withheld, for two different reasons that happen to share an
    /// answer:
    ///
    /// * **`Org`** — an organisation's own engineering principles. The public
    ///   tool supports the layer; every organisation fills it privately; nothing
    ///   of theirs can cross the boundary even by accident.
    /// * **`Project`** — the home carries a filesystem path, which is a private
    ///   identifier, and a stranger's project home would be meaningless
    ///   upstream anyway. Contributing such a rule means re-homing it first,
    ///   which is an authored act rather than a transfer.
    #[must_use]
    pub fn federation(&self) -> Federation {
        match self {
            Home::Global => Federation::Publishable,
            Home::Org { .. } => Federation::Withheld,
            Home::Domain { .. } => Federation::Publishable,
            Home::Project { .. } => Federation::Withheld,
        }
    }

    /// The global home.
    #[must_use]
    pub fn global() -> Self {
        Home::Global
    }

    /// An organisation home; the name must be non-empty.
    pub fn org(name: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Home::Org {
            name: OrgName::parse(name)?,
        })
    }

    /// A domain home; the name must be non-empty.
    pub fn domain(name: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Home::Domain {
            name: DomainName::parse(name)?,
        })
    }

    /// A project home; the path must be non-empty.
    pub fn project(path: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Home::Project {
            path: ProjectPath::parse(path)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_needs_nothing() {
        assert_eq!(Home::global(), Home::Global);
    }

    #[test]
    fn domain_requires_nonempty_name() {
        assert!(Home::domain("rust").is_ok());
        assert!(Home::domain("   ").is_err());
    }

    #[test]
    fn project_requires_nonempty_path() {
        assert!(Home::project("C:/repo").is_ok());
        assert!(Home::project("").is_err());
    }
}
