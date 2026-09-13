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
    Local {
        /// Which revision of this tag the document is, if its author has
        /// numbered one.
        ///
        /// **The same field a cache carries, answering the same question**:
        /// *which revision of this tag is this document?* A cache answers it
        /// about a copy; a home answers it about the original. It lives inside
        /// the authority rather than as a second field beside it because a
        /// cached file would otherwise carry the number twice — once as the
        /// revision it is, once as the revision it holds — and two spellings of
        /// one fact is the drift this repository refuses everywhere else.
        ///
        /// `None` is a real state rather than a zero: a rule nobody has
        /// published has no revision, which is every rule in this corpus.
        /// Absent, no cache of it can ever be told it is stale, and the poke
        /// says nothing rather than guessing at one.
        version: Option<Version>,
    },
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
    /// This install's own rule, with no revision recorded — what every rule
    /// file that says nothing about authority is, and what every rule in this
    /// corpus is.
    #[must_use]
    pub fn local() -> Self {
        Authority::Local { version: None }
    }

    /// This install's own rule at a stated revision: what a published rule in
    /// an upstream corpus looks like, and the only thing a cache of it can be
    /// compared against.
    #[must_use]
    pub fn local_at(version: Version) -> Self {
        Authority::Local {
            version: Some(version),
        }
    }

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
            Authority::Local { .. } | Authority::Adopted { .. } => true,
            Authority::Cached { .. } => false,
        }
    }

    /// Which revision of the tag this document is, if one is recorded.
    ///
    /// One question for all three variants — *which revision is this?* — so
    /// code comparing two documents of one tag never has to know what kind of
    /// authority each carries. [`Authority::is_behind`] asks the **different**
    /// question, and it is the one that must treat a home differently from a
    /// copy.
    #[must_use]
    pub fn version(&self) -> Option<Version> {
        match self {
            Authority::Local { version } => *version,
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
        // An exhaustive match rather than a read through `version()`, because
        // the two questions genuinely diverge for a home: `version()` says
        // which revision this document *is*, and a home at revision 3 facing a
        // document claiming 4 is not behind — it is the source, and the other
        // document is the stale one. Reading the revision through the other
        // accessor would make a home trail its own caches.
        match self {
            Authority::Local { .. } => false,
            Authority::Adopted { version, .. } | Authority::Cached { version, .. } => {
                *version < upstream
            }
        }
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
            Authority::Local { .. } => Err(AdoptError::AlreadyLocal),
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

/// Why a rule cannot be pulled into this install as a cache.
///
/// Every variant is a refusal rather than a transformation, for the same reason
/// [`NotContributable`](crate::contribute::NotContributable)'s are: a pull that
/// "did its best" would leave a file that looks like a cache of something and
/// is not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotPullable {
    /// The upstream rule declares no revision, so no cache of it could ever be
    /// told it is stale.
    #[error(
        "this upstream rule declares no revision — a cache of it could never be told it is \
         stale, so it is refused at the door rather than months later. Ask upstream to \
         publish it with `authority = {{ kind = \"local\", version = N }}`"
    )]
    NoUpstreamVersion,
    /// The rule's home never leaves a machine, so it never arrives on one.
    #[error(
        "this rule's home never leaves a machine, so it cannot arrive on one either: a \
         project home names somebody else's filesystem path, and an org layer is an \
         organisation's own"
    )]
    HomeIsWithheld,
    /// A rule with this tag is already here, and it is this install's own.
    #[error(
        "a rule with this tag is already here and it is yours — pulling would replace \
         hand-authored source with a stranger's copy. Re-home or rename one of them"
    )]
    WouldClobberLocal,
    /// A rule with this tag is already here, and it is a deliberate fork.
    #[error(
        "a rule with this tag is already here and it is a fork you took deliberately — \
         pulling would discard both your change and the provenance of what it was forked \
         from. Drop the fork first if you want upstream's copy back"
    )]
    WouldClobberFork,
}

/// A rule that may be written into this install **as a cache**.
///
/// **The second witness, and the reason it is a second one.** `EditableRule`
/// gates the path that writes a rule this install owns, and its constructor
/// refuses a cache — which is precisely what a pull must write. Giving
/// `write_rule` a flag to skip that check would make "edit a cache in place"
/// reachable by passing `true`, and that state is the one B1 spent a phase
/// making unconstructible. So there are two witnesses and two write paths:
/// [`EditableRule`] for source you maintain, `PulledRule` for a copy you do
/// not, and neither can be minted for the other's subject.
///
/// It owns its rule rather than borrowing one, because the cached rule **does
/// not exist yet** when the witness is minted: this constructor is the single
/// place a cache is ever constructed, so there is no second site to keep in
/// step with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PulledRule(Rule);

impl PulledRule {
    /// Mint the witness: take `upstream`'s rule, re-home nothing, and record
    /// what it is a copy of, at which revision, and when.
    ///
    /// `existing` is whatever this install already holds under that tag, which
    /// is what the two clobber refusals read. `None` means the tag is new here.
    ///
    /// The four refusals are asked **here, once**, at the only place a cache
    /// can come into existence — so a caller holding one of these has already
    /// passed all of them and `fsio` re-checks nothing.
    pub fn of(
        upstream: &Rule,
        existing: Option<&Rule>,
        from: SourceId,
        pulled: Date,
    ) -> Result<Self, NotPullable> {
        match upstream.home().federation() {
            super::Federation::Publishable => {}
            super::Federation::Withheld => return Err(NotPullable::HomeIsWithheld),
        }
        let version = upstream
            .authority()
            .version()
            .ok_or(NotPullable::NoUpstreamVersion)?;
        if let Some(existing) = existing {
            match existing.authority() {
                Authority::Cached { .. } => {}
                Authority::Local { .. } => return Err(NotPullable::WouldClobberLocal),
                Authority::Adopted { .. } => return Err(NotPullable::WouldClobberFork),
            }
        }

        Ok(PulledRule(Rule::new(
            upstream.tag().clone(), // allow:clone: the cache owns every field, and the upstream document it was read from is left intact for the caller to print
            upstream.title().clone(), // allow:clone: same
            upstream.error_class().clone(), // allow:clone: same
            upstream.home().clone(), // allow:clone: same — the cache keeps the home it arrived with, which is what lets it emit like a local rule
            upstream.created(),
            upstream.origin().clone(),   // allow:clone: same
            upstream.status().clone(),   // allow:clone: same
            upstream.incident().clone(), // allow:clone: same — upstream's `incident` is already a published account; the raw one never left their machine
            upstream.body().clone(),     // allow:clone: same
            // **No recurrences.** Upstream's history of its own rule is theirs;
            // this install has not seen this rule fire even once, and copying
            // their count would fabricate local evidence.
            Vec::new(),
            upstream.applies_to().to_vec(), // allow:clone: same
            Authority::cached(from, version, pulled),
            // Not carried: a published incident is what upstream wrote *to
            // publish with*, and this install has nothing to publish.
            None,
        )))
    }

    /// The cache this pull would write.
    #[must_use]
    pub fn rule(&self) -> &Rule {
        &self.0
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
        assert!(Authority::local().is_editable());
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
        assert_eq!(Authority::local().version(), None);
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
            Authority::local().adopt(date("2026-09-14")),
            Err(AdoptError::AlreadyLocal)
        );
    }
}
