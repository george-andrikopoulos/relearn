//! `Origin` — where a rule came from: a real failure, or existing practice
//! written down.
//!
//! This is the **counter-metric** to recurrence, and it exists because a metric
//! without one is a number waiting to be gamed. Recurrence counts only what
//! someone was willing to record about their own rule failing, so it falls
//! through under-reporting exactly as easily as through prevention. The counter
//! is the corpus's *inert* fraction — rules that have never recurred **and**
//! were never mined from a real failure — because the null result says those are
//! the ones that change nothing: you cannot author your way to a delta. A
//! library that looks healthy because it is full of them is the failure the
//! recurrence count would otherwise hide.

use super::Date;
use super::text::{EmptyText, nonempty};

/// Who signed off a mandated rule — a person, a board, a working group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approver(String);

impl Approver {
    /// Parse a non-empty approver.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("approval.by", s)?))
    }

    /// The approver text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The artefact a **codified** rule was written down from: a page, a paper, a
/// specification clause, a section of a standing instruction file.
///
/// Free text rather than a URL type, and deliberately. The artefact that
/// defines a practice is as often `~/.claude/CLAUDE.md, Reasoning &
/// Methodology` or a chapter of a book as it is something with a scheme and a
/// host, so a `Url` newtype would make the commonest source unrepresentable
/// while proving nothing about the uncommon one — a well-formed URL is not a
/// reachable one, and this field's job is to tell a reader where to look, not
/// to promise the look will succeed. What the type holds is the invariant
/// worth holding: a source that is present is not blank.
///
/// Named `SourceArtefact` rather than `Source` because [`SourceId`] already
/// means something else in this crate — which install a cached rule came
/// *from*. Two meanings of "source" one module apart is exactly the collision
/// `[R:newtype-liberally]` exists to prevent.
///
/// [`SourceId`]: super::SourceId
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceArtefact(String);

impl SourceArtefact {
    /// Parse a non-empty source artefact.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("source", s)?))
    }

    /// The artefact text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The control-framework requirement a mandated rule implements, in whatever
/// vocabulary the organisation's framework uses — `AC-6(9)`, `A.9.2.3`, a
/// policy clause number.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlRef(String);

impl ControlRef {
    /// Parse a non-empty control reference.
    pub fn parse(s: impl Into<String>) -> Result<Self, EmptyText> {
        Ok(Self(nonempty("approval.control", s)?))
    }

    /// The control reference text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The provenance of a **mandated** rule: who approved it, when, and against
/// which control.
///
/// For a mined rule, provenance is the incident — what went wrong. A mandate has
/// no incident, because nothing went wrong: it is a requirement somebody
/// accepted on a date. Recording the approver is what lets any compliance claim
/// follow from a record rather than from the tool asserting one
/// (`[R:guarantee-needs-a-reader]` — a tool claiming regulatory alignment with
/// no signer is an unenforced guarantee, and in a regulated environment a
/// liability rather than a feature).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approval {
    by: Approver,
    date: Date,
    control: ControlRef,
}

impl Approval {
    /// Record an approval: who, when, against which control. All three are
    /// required — an approval missing any of them is not an approval.
    #[must_use]
    pub fn new(by: Approver, date: Date, control: ControlRef) -> Self {
        Approval { by, date, control }
    }

    /// Who approved it.
    #[must_use]
    pub fn by(&self) -> &Approver {
        &self.by
    }

    /// When it was approved.
    #[must_use]
    pub fn date(&self) -> Date {
        self.date
    }

    /// The control it implements.
    #[must_use]
    pub fn control(&self) -> &ControlRef {
        &self.control
    }
}

/// Where a rule came from.
///
/// **`Codified` carries an *optional* source, and the optionality is the whole
/// design.** Until 2026-09-18 it carried none, on the stated ground that "a
/// practice written down from standing doctrine is meaningful without naming a
/// document". That reasoning was sound for the corpus it was written against —
/// every codified rule had been ported out of the author's own always-loaded
/// instruction file, where the `incident` prose already said so. It stopped
/// being sound the first time a practice was codified from **someone else's**
/// published artefact, because there the document is not a footnote to the
/// provenance, it *is* the provenance, and a reader who cannot reach it cannot
/// check the practice against the thing that defines it.
///
/// So "codified without a source" is still not an illegal state — it is the
/// common one, and twenty-two rule files depend on it staying free. What the
/// payload buys is the other half: a source that is *present* is non-blank by
/// construction, and a source on an origin that has no use for one is a parse
/// error rather than a field the parser quietly drops.
///
/// **`Mandated` does carry its approval, and that is the point.** An
/// organisation's control-framework requirements are not corrections: they have
/// no incident and were never mined. Their provenance is the sign-off, so the
/// approval is the variant's *payload* rather than a field beside it — which
/// makes both halves unrepresentable at once. A mandate with no approval cannot
/// be constructed, and an approval attached to a mined or codified rule cannot
/// be written down at all. The alternative, an `Option<Approval>` field on
/// `Rule`, makes both illegal states representable and turns the invariant into
/// a check somebody has to remember at every construction site.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// Written because something actually went wrong: there is a specific,
    /// dated failure behind it. These are the rules the error loop produced.
    Mined,
    /// Written down from existing practice or doctrine rather than from a
    /// single incident — the `incident` field records when and from where it
    /// was codified, not a failure it retired.
    ///
    /// **The source is optional, and that is not the same decision as the
    /// mandate's.** `Mandated` carries its approval because a mandate with no
    /// signer is an unenforced guarantee. A codified rule with no named
    /// artefact is not the analogous defect: a practice ported out of the
    /// author's own standing instruction file is fully accounted for by its
    /// `incident`, and twenty-two such rules predate this field. What the
    /// payload adds is the case that convention could not hold — a practice
    /// codified from *someone else's* published artefact, where the document
    /// is the provenance rather than a footnote to it.
    /// `[R:source-practice-from-its-artefact]`
    Codified(Option<SourceArtefact>),
    /// Required by an organisation's control framework and signed off, rather
    /// than learned from anything. Carries the approval that is its provenance.
    Mandated(Approval),
}

/// Why an `origin` field (with or without its `approval` table) did not describe
/// a valid origin.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OriginError {
    /// The `origin` field held a value that is not a known origin.
    #[error("`origin` has unknown value `{0}` (expected mined | codified | mandated)")]
    Unknown(String),
    /// `origin = "mandated"` with no `approval` table. A mandate with no signer
    /// is the unenforced guarantee this field exists to prevent, so it is a
    /// parse error rather than a defaulted blank.
    #[error("`origin = \"mandated\"` requires an `approval` table (by, date, control)")]
    MandateWithoutApproval,
    /// An `approval` table on a rule that is not mandated. Refused rather than
    /// ignored: a silently dropped approval reads, to the next person, as a
    /// rule that was signed off when it was not.
    #[error("`approval` is only valid with `origin = \"mandated\"` (got `{0}`)")]
    ApprovalWithoutMandate(String),
    /// A `source` field on a rule that is not codified. Refused rather than
    /// ignored, for the same reason as the approval above: a source the parser
    /// drops reads back, to the next person, as an artefact the rule cited. A
    /// mined rule's provenance is its incident and a mandate's is its approval,
    /// so a `source` beside either is a mistake or a mislabelled origin — both
    /// worth stopping the build for.
    #[error("`source` is only valid with `origin = \"codified\"` (got `{0}`)")]
    SourceWithoutCodification(String),
}

/// A rule's standing in the recurrence statistics.
///
/// The counter-metric turns on a three-way distinction that was previously
/// computed as `!is_mandated() && !is_mined()` -- two `matches!` composed into
/// an implicit "everything else", so a fourth `Origin` would have joined the
/// *inert* bucket without anybody deciding. Naming the three roles in one
/// exhaustive match makes that a compile error, which matters more here than
/// elsewhere: this is the number whose entire purpose is to be honest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecurrenceRole {
    /// Mined from a real failure: it is recurrence evidence, and never inert.
    Evidence,
    /// Authored from standing practice: counted *inert* if it has never fired.
    Authored,
    /// Outside the statistics in both directions -- a mandate was never mined,
    /// so it cannot be evidence, and "inert" is a judgement about something
    /// that was meant to be evidence.
    Excluded,
}

impl Origin {
    /// The origin as the one word the neutral format writes, and the same word
    /// `parse` accepts back.
    ///
    /// Exhaustive with no catch-all, so a fourth origin cannot be reported as
    /// one of these three by default. The payloads are deliberately dropped:
    /// this answers *what kind of provenance does this rule have*, which is the
    /// question the recurrence statistics turn on — a mandate is not evidence
    /// and `Codified` never fired. What the artefact or the signer *says* is a
    /// different question, and one whose answer is free prose.
    #[must_use]
    pub fn kind_word(&self) -> &'static str {
        match self {
            Origin::Mined => "mined",
            Origin::Codified(_) => "codified",
            Origin::Mandated(_) => "mandated",
        }
    }

    /// Parse the neutral format's `origin` value **together with** its optional
    /// `approval` table and its optional `source` — one perimeter that sees all
    /// three, so each correspondence is settled where the evidence is rather
    /// than re-checked later by something holding only one of them.
    ///
    /// Both payloads are refused on the origins they have no meaning for, and
    /// the match has no catch-all over the pair, so a fourth origin cannot
    /// acquire either payload's policy by default.
    pub fn parse(
        s: &str,
        approval: Option<Approval>,
        source: Option<SourceArtefact>,
    ) -> Result<Self, OriginError> {
        let kind = s.trim();
        match (kind, approval, source) {
            ("mined", None, None) => Ok(Origin::Mined),
            ("codified", None, source) => Ok(Origin::Codified(source)),
            ("mandated", Some(approval), None) => Ok(Origin::Mandated(approval)),
            ("mandated", None, _) => Err(OriginError::MandateWithoutApproval),
            ("mined" | "codified", Some(_), _) => {
                Err(OriginError::ApprovalWithoutMandate(kind.to_owned()))
            }
            ("mined" | "mandated", _, Some(_)) => {
                Err(OriginError::SourceWithoutCodification(kind.to_owned()))
            }
            (other, _, _) => Err(OriginError::Unknown(other.to_owned())),
        }
    }

    /// The neutral format's spelling. The inverse of [`Origin::parse`] for the
    /// payload-free variants; `Mandated` also needs its `approval` table, which
    /// the serializer writes beside this value.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Origin::Mined => "mined",
            Origin::Codified(_) => "codified",
            Origin::Mandated(_) => "mandated",
        }
    }

    /// The artefact this practice was written down from, for a codified rule
    /// that names one; `None` for every other origin — which is not a sentinel
    /// but the honest reading, exactly as for [`Origin::approval`]. A mined
    /// rule has an incident instead, and a mandate has a signer.
    #[must_use]
    pub fn source(&self) -> Option<&SourceArtefact> {
        match self {
            Origin::Mined | Origin::Mandated(_) => None,
            Origin::Codified(source) => source.as_ref(),
        }
    }

    /// The approval, for a mandated rule; `None` for every other origin — which
    /// is not a sentinel but the honest reading: only a mandate has a signer.
    #[must_use]
    pub fn approval(&self) -> Option<&Approval> {
        match self {
            Origin::Mined | Origin::Codified(_) => None,
            Origin::Mandated(approval) => Some(approval),
        }
    }

    /// Whether this rule was produced by the error loop rather than authored.
    ///
    /// The distinction the counter-metric turns on: a `Codified` rule that has
    /// never recurred is evidence of nothing, whereas a `Mined` rule that has
    /// never recurred is a rule that may well be working.
    #[must_use]
    pub fn is_mined(&self) -> bool {
        self.recurrence_role() == RecurrenceRole::Evidence
    }

    /// This origin's standing in the recurrence statistics -- the single
    /// exhaustive authority, so a new origin cannot join a bucket by default.
    #[must_use]
    pub fn recurrence_role(&self) -> RecurrenceRole {
        match self {
            Origin::Mined => RecurrenceRole::Evidence,
            Origin::Codified(_) => RecurrenceRole::Authored,
            Origin::Mandated(_) => RecurrenceRole::Excluded,
        }
    }

    /// Whether this rule was mandated rather than learned.
    ///
    /// A mandate is **outside** the recurrence statistics entirely, in both
    /// directions. It cannot be recurrence evidence, because nothing was mined;
    /// and it must not count as *inert* either, because inert means "authored
    /// and never fired" — a judgement about a rule that was supposed to be
    /// evidence. Counting mandates in either number swamps the only figure that
    /// says whether prose is holding, with rules that were never about that.
    #[must_use]
    pub fn is_mandated(&self) -> bool {
        self.recurrence_role() == RecurrenceRole::Excluded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approval() -> Approval {
        Approval::new(
            Approver::parse("the change board").expect("non-empty approver"),
            Date::parse("2026-07-11").expect("valid date"),
            ControlRef::parse("AC-6(9)").expect("non-empty control"),
        )
    }

    #[test]
    fn parses_both_spellings() {
        assert_eq!(Origin::parse("mined", None, None), Ok(Origin::Mined));
        assert_eq!(
            Origin::parse("codified", None, None),
            Ok(Origin::Codified(None))
        );
    }

    #[test]
    fn trims_before_matching() {
        assert_eq!(
            Origin::parse("  codified  ", None, None),
            Ok(Origin::Codified(None))
        );
    }

    #[test]
    fn a_mandate_parses_with_its_approval() {
        assert_eq!(
            Origin::parse("mandated", Some(approval()), None),
            Ok(Origin::Mandated(approval()))
        );
    }

    // The two halves of "required when and only when". Neither is a warning and
    // neither is silently dropped: a mandate with no signer is the unenforced
    // guarantee the field exists to prevent, and a dropped approval reads to the
    // next person as a rule that was signed off when it was not.
    #[test]
    fn a_mandate_without_an_approval_is_refused() {
        assert_eq!(
            Origin::parse("mandated", None, None),
            Err(OriginError::MandateWithoutApproval)
        );
    }

    #[test]
    fn an_approval_without_a_mandate_is_refused_and_names_the_origin() {
        assert_eq!(
            Origin::parse("mined", Some(approval()), None),
            Err(OriginError::ApprovalWithoutMandate("mined".to_owned()))
        );
        assert_eq!(
            Origin::parse("codified", Some(approval()), None),
            Err(OriginError::ApprovalWithoutMandate("codified".to_owned()))
        );
    }

    #[test]
    fn a_mandate_is_neither_mined_nor_inert_material() {
        let mandated = Origin::Mandated(approval());
        assert!(!mandated.is_mined());
        assert!(mandated.is_mandated());
        assert!(!Origin::Mined.is_mandated());
        assert!(!Origin::Codified(None).is_mandated());
    }

    #[test]
    fn only_a_mandate_has_an_approval() {
        assert!(Origin::Mandated(approval()).approval().is_some());
        assert!(Origin::Mined.approval().is_none());
        assert!(Origin::Codified(None).approval().is_none());
    }

    // The error names the offending value and the alternatives, so a typo in a
    // rule file reports what to write rather than only that it was wrong.
    #[test]
    fn an_unknown_value_is_named() {
        let err = Origin::parse("invented", None, None).expect_err("not an origin");
        let text = err.to_string();
        assert!(text.contains("invented"), "{text}");
        assert!(text.contains("mined | codified | mandated"), "{text}");
    }

    #[test]
    fn as_str_round_trips_through_parse() {
        for origin in [
            Origin::Mined,
            Origin::Codified(None),
            Origin::Codified(Some(
                SourceArtefact::parse("a page").expect("non-empty source"),
            )),
            Origin::Mandated(approval()),
        ] {
            // Both payloads travel beside the spelling, which is exactly how the
            // serializer writes them: `origin = "..."` then the table or the
            // field. The codified-with-source case is in the list because
            // `as_str` is lossy for it — two distinct origins share the spelling
            // `codified`, and only the payload tells them apart.
            let approval = origin.approval().cloned(); // allow:clone: the round trip needs an owned approval to hand back to `parse`, and the original must stay for the comparison
            let source = origin.source().cloned(); // allow:clone: same — `parse` takes the source by value while the original is still needed for the comparison
            assert_eq!(Origin::parse(origin.as_str(), approval, source), Ok(origin));
        }
    }

    #[test]
    fn only_mined_is_mined() {
        assert!(Origin::Mined.is_mined());
        assert!(!Origin::Codified(None).is_mined());
    }

    #[test]
    fn every_origin_declares_its_role_in_the_statistics() {
        // This was `!is_mandated() && !is_mined()` -- an implicit "everything
        // else" that a fourth origin would have joined without anybody
        // deciding, in the one number whose whole purpose is honesty.
        assert_eq!(Origin::Mined.recurrence_role(), RecurrenceRole::Evidence);
        assert_eq!(
            Origin::Codified(None).recurrence_role(),
            RecurrenceRole::Authored
        );
        assert_eq!(
            Origin::Mandated(approval()).recurrence_role(),
            RecurrenceRole::Excluded
        );
    }
}
