//! `contribute` — what a rule looks like when it leaves the machine.
//!
//! **[`Contribution`] is a projection, not a filter.** It borrows the fields
//! that may be published and **never borrows `incident` or the recurrences at
//! all**, so no renderer can emit them, forget to strip them, or strip them
//! incorrectly. The difference matters: a filter that removes the raw incident
//! is code somebody can edit, reorder or duplicate; a type that never held it
//! cannot be made to reveal it by any amount of editing downstream.
//!
//! What travels is the **published incident** — a separate authored field, a
//! rewritten account with no quotation, no names, no paths, no repository
//! identifiers. Nothing here derives it. §12.2 of `docs/federated-relearn.md`
//! puts that scrub on the contributor, because judgement about text is the one
//! thing no gate performs: a published incident can identify a customer with
//! none of the words a matcher knows.
//!
//! The code's job is therefore narrow and worth doing anyway: make the raw
//! field unreachable, refuse what it *can* recognise, and put the exact text in
//! front of a human before anything is written.

use std::fmt::Write as _;

use crate::rule::{
    Authority, Body, Date, ErrorClass, Federation, Home, Origin, PublishedIncident, Rule, RuleTag,
    ScopeTag, Status, Version, to_document,
};

/// Why a rule may not be contributed.
///
/// Every variant is a refusal rather than a transformation. Publishing a
/// stripped-down version of a rule that cannot be published would produce
/// something that *looks* like the rule and is not it, which is worse than
/// publishing nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotContributable {
    /// The rule's home never leaves the machine: an organisation's own
    /// principles, or a project home that names a filesystem path. This is
    /// [`Home::federation`] consumed — the exclusion established before
    /// anything could publish, now doing its job.
    #[error(
        "this rule's home never leaves the machine (an org layer, or a project home that names \
         a path) — re-home it first if it is genuinely general"
    )]
    HomeIsWithheld,
    /// No published incident has been authored. The raw one cannot stand in for
    /// it, which is the whole design.
    #[error(
        "this rule has no `published_incident` — write one: the error class, what went wrong \
         and what it cost, with no quotation, no names, no paths, no repository identifiers"
    )]
    NoPublishedIncident,
    /// The rule is a cache of an upstream rule: it is already published, and it
    /// is not this install's to contribute. A **fork** may be contributed —
    /// that is a change you made.
    #[error("this rule is a cache of an upstream rule — adopt it first if you have changed it")]
    IsACache,
    /// The rule is mandated, so its provenance is an approver: a person or a
    /// board, by name. Refused rather than stripped — a mandate without its
    /// approval would claim a sign-off it no longer records.
    #[error(
        "a mandated rule carries an approver's name as its provenance, and compliance content \
         is an organisation's own — it is not shared-corpus material"
    )]
    IsMandated,
}

/// A proposed revision does not supersede what is already published.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "revision {proposed} does not supersede the {published} already published: a republication \
     must go forwards, or every cache of this rule reads as newer than upstream and nothing \
     notices, because staleness is decided by comparing these two numbers"
)]
pub struct NotSuperseding {
    proposed: Version,
    published: Version,
}

impl NotSuperseding {
    /// The revision that was asked for.
    #[must_use]
    pub fn proposed(self) -> Version {
        self.proposed
    }

    /// The revision already on the destination.
    #[must_use]
    pub fn published(self) -> Version {
        self.published
    }
}

/// A revision that supersedes whatever is already published under its tag.
///
/// **The witness that makes a backwards republication unrenderable**, rather
/// than a check somebody performs before calling the renderer. `to_document`
/// takes one of these and nothing else, so "published a revision that goes
/// backwards" is not a mistake a call site can make — it is a document that
/// cannot be built.
///
/// The failure it closes is silent in both directions at once. A cache records
/// the revision it holds and `cache-behind` decides staleness by comparing
/// numbers, so republishing at or below the published revision makes every
/// existing cache read as *newer than upstream* — and nothing anywhere notices,
/// because the comparison is exactly what has been corrupted.
///
/// A first publication supersedes nothing, so any revision is valid for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupersedingVersion(Version);

impl SupersedingVersion {
    /// Mint the witness, or refuse because `proposed` does not go forwards.
    ///
    /// `published` is whatever revision the destination already holds under
    /// this tag — `None` when nothing is published there yet.
    pub fn of(proposed: Version, published: Option<Version>) -> Result<Self, NotSuperseding> {
        match published {
            Some(published) if proposed <= published => Err(NotSuperseding {
                proposed,
                published,
            }),
            _ => Ok(SupersedingVersion(proposed)),
        }
    }

    /// The revision, which the witness proves goes forwards.
    #[must_use]
    pub fn get(self) -> Version {
        self.0
    }
}

/// A rule as it would leave the machine.
///
/// Holds **only** publishable fields, by reference. There is no `incident` here
/// and no recurrences: not removed, never borrowed.
#[derive(Debug, Clone, Copy)]
pub struct Contribution<'a> {
    tag: &'a RuleTag,
    title: &'a str,
    error_class: &'a ErrorClass,
    home: &'a Home,
    created: Date,
    origin: &'a Origin,
    status: &'a Status,
    published_incident: &'a PublishedIncident,
    body: &'a Body,
    applies_to: &'a [ScopeTag],
}

impl<'a> Contribution<'a> {
    /// Project a rule into what may be published, or refuse and say why.
    ///
    /// The four refusals are checked here, once, at the only place a
    /// `Contribution` can come into existence — so a caller holding one has
    /// already passed all of them and no renderer re-checks.
    pub fn of(rule: &'a Rule) -> Result<Self, NotContributable> {
        match rule.home().federation() {
            Federation::Publishable => {}
            Federation::Withheld => return Err(NotContributable::HomeIsWithheld),
        }
        if let Authority::Cached { .. } = rule.authority() {
            return Err(NotContributable::IsACache);
        }
        if rule.origin().is_mandated() {
            return Err(NotContributable::IsMandated);
        }
        let published_incident = rule
            .published_incident()
            .ok_or(NotContributable::NoPublishedIncident)?;

        Ok(Contribution {
            tag: rule.tag(),
            title: rule.title().as_str(),
            error_class: rule.error_class(),
            home: rule.home(),
            created: rule.created(),
            origin: rule.origin(),
            status: rule.status(),
            published_incident,
            body: rule.body(),
            applies_to: rule.applies_to(),
        })
    }

    /// The rule's identity, which travels.
    #[must_use]
    pub fn tag(self) -> &'a RuleTag {
        self.tag
    }

    /// The account that travels in place of the incident.
    #[must_use]
    pub fn published_incident(self) -> &'a PublishedIncident {
        self.published_incident
    }

    /// Render the contribution as a rule document.
    ///
    /// It **is** a rule — upstream parses it, lints it and emits it with the
    /// same code — so this builds one and serializes it rather than hand-rolling
    /// a second document format that would drift from the first.
    ///
    /// Three deliberate substitutions, each a consequence of what the projection
    /// holds: the published incident stands where `incident` goes, there are no
    /// recurrences (this install's history of the rule is its own business, and
    /// anonymous aggregate counts are the report flow's job), and the authority
    /// is `Local` **at the stated revision** — upstream is the home of what it
    /// publishes, and a local fork's provenance is not upstream's concern.
    ///
    /// **The revision is an argument because publication is what assigns it.**
    /// A rule's own authority cannot supply it: a fork's records the revision
    /// it was *forked at*, which is a different number from the one it is being
    /// published as, and publishing a changed rule under a revision that
    /// already exists upstream would make every cache of it wrong in the one
    /// direction nobody could detect. Without a revision here, no cache of this
    /// rule could ever be told it is stale — which is why `pull` refuses an
    /// unnumbered upstream rule outright rather than accepting a copy whose
    /// staleness signal is dead.
    #[must_use]
    pub fn to_document(self, version: SupersedingVersion) -> String {
        // The published incident is a *witness of a different type*, so putting
        // it where the incident goes is an explicit re-parse rather than an
        // accident of shape. It cannot fail: non-empty is non-empty.
        let incident = crate::rule::Incident::parse(self.published_incident.as_str())
            .expect("a published incident is non-empty, so it is a valid incident");

        let rule = Rule::new(
            self.tag.clone(), // allow:clone: the rendered rule owns its fields, and the source rule outlives this borrow unchanged — nothing here may mutate what it was given
            crate::rule::Title::parse(self.title).expect("a title is non-empty"),
            self.error_class.clone(), // allow:clone: same — an owned Rule is what the serializer takes
            self.home.clone(),        // allow:clone: same
            self.created,
            self.origin.clone(), // allow:clone: same
            self.status.clone(), // allow:clone: same
            incident,
            self.body.clone(), // allow:clone: same
            Vec::new(),
            self.applies_to.to_vec(),
            Authority::local_at(version.get()),
            None,
        );
        to_document(&rule)
    }

    /// A human-readable account of exactly what would leave, for the
    /// confirmation step.
    ///
    /// The whole document, verbatim, because a summary is precisely what a
    /// reader would skim. The one thing this adds is the reminder that the
    /// matcher cannot read for judgement.
    #[must_use]
    pub fn what_would_leave(self, version: SupersedingVersion) -> String {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "This is exactly what would leave this machine — read it before confirming.\n\
             The matcher checks for names you have written down. It cannot tell you whether \
             this text identifies a person, a customer or a repository without naming one.\n"
        );
        out.push_str(&self.to_document(version));
        out
    }
}
