//! `Authority` — whether this install is a rule's home, or holds a copy of one
//! whose home is elsewhere.
//!
//! **A cache is an emission.** The principle this repository already lives by —
//! *no emitted artifact is a source; anything under an emitter's output path may
//! be regenerated and must never be hand-edited* — applied one level up. A
//! cached rule compiles into the instruction layer exactly like a local one,
//! which is the point of caching it; what it may not do is be edited in place,
//! because an edited cache is a **silent fork**: P2 gone, with no error to read
//! and nothing recording that the copy and its source have diverged.
//!
//! Forking is allowed. Forking by accident is not — which is what
//! [`Authority::adopt`] and [`EditableRule`] are between them for.

use super::text::{EmptyText, nonempty};
use super::{Date, Rule};

/// Which upstream a cached rule came from.
///
/// Shape follows [`ScopeTag`](super::ScopeTag): trimmed, non-empty, and an
/// identifier a person types rather than a URL. A source is named, not located —
/// where a source lives is a property of the clone on disk, and putting a path
/// here would bake one machine's layout into a rule file that travels.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SourceId(String);

impl SourceId {
    /// Parse a non-empty source identifier.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("authority.from", s)?))
    }

    /// The source identifier text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Which revision of an upstream rule a cache holds.
///
/// **A monotonic revision number, and the total order is the whole point.** Two
/// caches of one rule must be comparable — always, for every pair — or "is this
/// cache stale?" has no answer in some cases, and staleness is the only reason
/// the version is recorded at all. `Ord` over a `u32` gives that by
/// construction: trichotomy holds for every pair, with no configuration, no
/// parsing rules and nothing to get wrong.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(u32);

impl Version {
    /// A revision number. Any `u32` is a valid version — the ordering carries
    /// the meaning, and no individual value is illegal.
    #[must_use]
    pub fn new(revision: u32) -> Self {
        Version(revision)
    }

    /// The revision number.
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Whether this install is a rule's home, holds a cache of someone else's, or
/// holds a fork it took deliberately.
///
/// Three variants because there are three states and the third is the one a
/// two-state model loses. A cache that "became local" on adoption would be
/// indistinguishable from a rule authored here — the silent fork wearing a
/// different hat, arriving through the very command that exists to make forking
/// deliberate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Authority {
    /// This install is the rule's home. The source of truth, and editable. The
    /// default, and what every rule written before this field existed is.
    Local,
    /// A fork taken deliberately: editable, and it remembers what it came from.
    Adopted {
        /// The upstream it was forked from.
        from: SourceId,
        /// The upstream revision it was forked at.
        version: Version,
        /// When the cache it was forked from was pulled.
        pulled: Date,
        /// When the fork was taken.
        adopted: Date,
    },
    /// Home is elsewhere. A cache: regenerable, never hand-edited.
    Cached {
        /// The upstream it came from.
        from: SourceId,
        /// The upstream revision this copy holds.
        version: Version,
        /// When it was pulled.
        pulled: Date,
    },
}

/// Adopting a rule that is not a cache.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AdoptError {
    /// The rule is this install's own, so there is nothing to fork from.
    #[error("this rule is already local — there is nothing to adopt it from")]
    AlreadyLocal,
    /// The rule is already a fork. Adopting it again would overwrite the
    /// provenance of the first fork with a second one that never happened.
    #[error("this rule was already adopted from `{from}` at version {version}")]
    AlreadyAdopted {
        /// The source the first adoption named.
        from: String,
        /// The version the first adoption named.
        version: Version,
    },
}

impl Authority {
    /// A cache of an upstream rule.
    #[must_use]
    pub fn cached(from: SourceId, version: Version, pulled: Date) -> Self {
        Authority::Cached {
            from,
            version,
            pulled,
        }
    }

    /// A deliberate fork of an upstream rule.
    #[must_use]
    pub fn adopted(from: SourceId, version: Version, pulled: Date, adopted: Date) -> Self {
        Authority::Adopted {
            from,
            version,
            pulled,
            adopted,
        }
    }

    /// Whether a rule under this authority may be edited in place.
    ///
    /// One exhaustive match, and the only definition of the word: no second
    /// predicate anywhere can drift from it, and a future variant is a compile
    /// error here rather than a state that quietly becomes editable.
    #[must_use]
    pub fn is_editable(&self) -> bool {
        match self {
            Authority::Local | Authority::Adopted { .. } => true,
            Authority::Cached { .. } => false,
        }
    }

    /// The upstream revision this authority refers to, if any.
    #[must_use]
    pub fn version(&self) -> Option<Version> {
        match self {
            Authority::Local => None,
            Authority::Adopted { version, .. } | Authority::Cached { version, .. } => {
                Some(*version)
            }
        }
    }

    /// Whether this authority is behind `upstream`.
    ///
    /// Always answerable, which is what the total order on [`Version`] buys: a
    /// content hash would tell you two copies differ and never which is newer,
    /// and "differs" is not the question a reader is asking.
    ///
    /// A `Local` rule is behind nothing — there is no upstream for it to trail.
    #[must_use]
    pub fn is_behind(&self, upstream: Version) -> bool {
        self.version().is_some_and(|held| held < upstream)
    }

    /// Take a deliberate fork of a cached rule: the result is editable and
    /// records what it was forked from and when.
    ///
    /// Refused for anything that is not a cache. Adopting a local rule would
    /// write a fork provenance naming a source it never came from; adopting an
    /// already-adopted one would overwrite the first fork's provenance with a
    /// second fork that did not happen.
    pub fn adopt(&self, on: Date) -> Result<Authority, AdoptError> {
        match self {
            Authority::Local => Err(AdoptError::AlreadyLocal),
            Authority::Adopted { from, version, .. } => Err(AdoptError::AlreadyAdopted {
                from: from.as_str().to_owned(),
                version: *version,
            }),
            Authority::Cached {
                from,
                version,
                pulled,
            } => Ok(Authority::Adopted {
                from: from.clone(), // allow:clone: the adopted authority owns its provenance, and the cache it was taken from is left intact for the caller to compare against
                version: *version,
                pulled: *pulled,
                adopted: on,
            }),
        }
    }
}

/// A rule that may be written back to its file.
///
/// **The witness that makes the refusal structural.** Any path that writes a
/// rule file takes one of these, and the only constructor refuses `Cached` — so
/// a write path added later cannot reach the filesystem without minting the
/// witness, and minting it is where the question is asked. A check inside the
/// write function would be satisfied exactly once: the second write path would
/// not have it, would compile, and would fork a cache in silence.
///
/// Same shape as `Library<Validated>` gating emission, and the same argument as
/// `[R:generate-guards-unversioned]`: a marker the code recognises, never
/// discipline at each call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EditableRule<'a>(&'a Rule);

/// A cached rule cannot be edited in place — that is a silent fork. `adopt` it
/// first, or open a contribution upstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "this rule is a cache of an upstream rule and must not be edited in place — \
     `adopt` it to take a deliberate fork, or contribute the change upstream"
)]
pub struct CachedIsNotEditable;

impl<'a> EditableRule<'a> {
    /// Mint the witness, or refuse because the rule is a cache.
    pub fn of(rule: &'a Rule) -> Result<Self, CachedIsNotEditable> {
        if rule.is_editable() {
            Ok(EditableRule(rule))
        } else {
            Err(CachedIsNotEditable)
        }
    }

    /// The rule, which the witness proves is editable.
    #[must_use]
    pub fn rule(&self) -> &'a Rule {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source() -> SourceId {
        SourceId::parse("relearn-upstream").expect("valid source")
    }

    fn date(s: &str) -> Date {
        Date::parse(s).expect("valid date")
    }

    #[test]
    fn a_source_id_is_trimmed_and_non_empty() {
        assert_eq!(
            SourceId::parse("  relearn-upstream ")
                .expect("valid")
                .as_str(),
            "relearn-upstream"
        );
        assert!(SourceId::parse("   ").is_err());
    }

    #[test]
    fn only_a_cache_is_uneditable() {
        assert!(Authority::Local.is_editable());
        assert!(
            Authority::adopted(
                source(),
                Version::new(1),
                date("2026-09-13"),
                date("2026-09-14")
            )
            .is_editable()
        );
        assert!(!Authority::cached(source(), Version::new(1), date("2026-09-13")).is_editable());
    }

    #[test]
    fn a_local_authority_holds_no_version() {
        assert_eq!(Authority::Local.version(), None);
        assert_eq!(
            Authority::cached(source(), Version::new(9), date("2026-09-13")).version(),
            Some(Version::new(9))
        );
    }

    // Adoption keeps the version and the pull date: the fork's provenance is
    // where it came from, not when somebody happened to run the command.
    #[test]
    fn adoption_carries_the_pull_date_forward() {
        let adopted = Authority::cached(source(), Version::new(5), date("2026-09-01"))
            .adopt(date("2026-09-14"))
            .expect("a cache adopts");
        match adopted {
            Authority::Adopted {
                version, pulled, ..
            } => {
                assert_eq!(version, Version::new(5));
                assert_eq!(pulled.to_string(), "2026-09-01");
            }
            other => panic!("expected Adopted, got {other:?}"),
        }
    }

    #[test]
    fn adopting_a_local_rule_is_refused() {
        assert_eq!(
            Authority::Local.adopt(date("2026-09-14")),
            Err(AdoptError::AlreadyLocal)
        );
    }
}
