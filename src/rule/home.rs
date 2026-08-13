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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Home {
    /// Applies everywhere.
    Global,
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

impl Home {
    /// The global home.
    #[must_use]
    pub fn global() -> Self {
        Home::Global
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
