//! `lint` — advisory static analysis over a validated library. Findings are
//! *reported*, never acted on: a flagged rule is input to a human cut-list
//! decision, and the tool must never auto-delete a correction (that would be the
//! sediment-in-reverse this project exists to prevent).
//!
//! The checks here are the deterministic ones — overlapping scope, home-slug
//! collision, and reference integrity (dangling references, and citations of
//! *retired* rules). Two Phase-B checks are deliberately **not** implemented as
//! code here, because faking them as heuristics would be dishonest:
//! - **contradiction** between rules is a semantic judgment — it is handled by a
//!   periodic *Claude review pass* over the corpus (a governance control,
//!   delegated through the subscription; see ARCHITECTURE), never a keyword
//!   heuristic dressed up as certainty;
//! - **cold-surface** (a rule nothing exercises) needs runtime invocation data,
//!   which lives in the stochos-lab ledger (phase C), not in the rule text.
//!
//! **Must NOT:** modify or delete rules, or perform I/O. It reads a library and
//! returns findings; the caller decides what to do with them.

use std::collections::BTreeMap;
use std::fmt;

use crate::emit::HomeSlug;
use crate::library::{Library, Validated};
use crate::rule::{Date, Home, ProseCoverage, Rule, RuleTag, ScopeTag, Status};

/// How much a finding matters. Ordered so `Error > Warning > Info`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// For information; most likely fine as-is.
    Info,
    /// Worth a look — possibly redundant, in tension, or a typo.
    Warning,
    /// A real correctness risk: two rules would collide on one output file.
    Error,
}

impl Severity {
    /// A lowercase label for diagnostics.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Severity::Info => "info",
            Severity::Warning => "warning",
            Severity::Error => "error",
        }
    }
}

/// Something the linter noticed. Advisory only — never a reason to delete a rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Finding {
    /// Two or more rules declare the same error class — possibly redundant or in
    /// tension. Carries the class and the rules that share it.
    OverlappingScope {
        /// The shared error-class text (as first written).
        error_class: String,
        /// The tags of the rules that share it.
        tags: Vec<RuleTag>,
    },
    /// A scope used by exactly one rule sits a slip away from one used by
    /// many — almost certainly a typo, and a silent one: the shape rules pass,
    /// nothing fails to parse, and the rule simply serves an audience nobody
    /// asks for.
    ///
    /// **This is what replaces a vocabulary curator** (§12.5). The set of
    /// scopes in use is whatever appears in the corpus — one identity space,
    /// not a controlled list somebody owns — and drift is a near-miss problem,
    /// which is detectable without an owner.
    ScopeNearDuplicate {
        /// The scope used by exactly one rule.
        rare: ScopeTag,
        /// The established scope it is a slip away from.
        common: ScopeTag,
        /// The rules declaring the rare spelling — where the fix goes.
        tags: Vec<RuleTag>,
    },
    /// Two or more rules with *different* homes slugify to the same token, so
    /// they would silently share one emitted skill file — a lost rule.
    HomeSlugCollision {
        /// The colliding home slug.
        slug: String,
        /// The tags of the colliding rules.
        tags: Vec<RuleTag>,
    },
    /// A rule cites another rule's tag (in its body or incident) that is not
    /// present in the library — a dangling reference.
    DanglingReference {
        /// The rule that carries the citation.
        from: RuleTag,
        /// The cited tag that does not resolve.
        to: RuleTag,
    },
    /// A rule cites another rule that *exists* but has been retired (atticked or
    /// graduated) — likely benign (a historical pointer), so `Info`, but worth a
    /// glance in case the citation is building on withdrawn guidance.
    RetiredReference {
        /// The rule that carries the citation.
        from: RuleTag,
        /// The cited, retired rule.
        to: RuleTag,
        /// The retired rule's status kind (`graduated` or `atticked`).
        status: &'static str,
    },
    /// A **graduated** rule recurred *after* it graduated. The named stronger
    /// control was claimed to hold this and demonstrably did not.
    ///
    /// The sharpest finding the library can produce, and the only recurrence
    /// finding that is an `Error`: the emitted instruction layer is telling
    /// every reader *"Also enforced by {to}"*, which the recurrence proves
    /// false. That is a lying artefact in a live artefact
    /// (`[R:repair-the-lying-artefact]`), not a rule merely wanting promotion.
    RecurrenceAfterGraduation {
        /// The rule whose graduation did not hold.
        tag: RuleTag,
        /// The control it was said to have graduated to.
        to: String,
        /// When it graduated.
        graduated: Date,
        /// The most recent recurrence *after* that date.
        recurred: Date,
    },
    /// An `Active` rule — one held by prose alone — has recurred since it was
    /// written. The prose is demonstrably not holding it, so it wants a
    /// stronger control and a `Status::Graduated` recording which.
    UnheldRecurrence {
        /// The rule that keeps firing.
        tag: RuleTag,
        /// Which status it is in -- `active` (prose holds all of it) or
        /// `partial` (prose holds the part its controls do not). Carried rather
        /// than assumed: this message said "is `active`" unconditionally, which
        /// `Status::Partial` made false on arrival.
        status: &'static str,
        /// How many recurrences it records.
        times: usize,
        /// The date of the most recent one.
        latest: Date,
    },
}

impl Finding {
    /// The severity of this finding.
    #[must_use]
    pub fn severity(&self) -> Severity {
        match self {
            Finding::HomeSlugCollision { .. }
            // An `Error` because the *emitted tree* is wrong: it carries a
            // "> Also enforced by ..." line that this finding proves false.
            | Finding::RecurrenceAfterGraduation { .. } => Severity::Error,
            // `Warning`, not `Error`. `Error` here means the *emitted tree*
            // would be wrong — two rules colliding on one output file. An
            // unheld recurrence is a fault in the **library**: the emission is
            // correct, and a build that emits correctly should not fail.
            Finding::OverlappingScope { .. }
            | Finding::ScopeNearDuplicate { .. }
            | Finding::DanglingReference { .. }
            | Finding::UnheldRecurrence { .. } => Severity::Warning,
            Finding::RetiredReference { .. } => Severity::Info,
        }
    }
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Finding::OverlappingScope { error_class, tags } => write!(
                f,
                "overlapping scope: {} rules share error class {error_class:?}: {}",
                tags.len(),
                join_tags(tags)
            ),
            // States the suspicion and where to act on it, not the arithmetic:
            // a reader does not need the edit distance, they need to know which
            // spelling is the odd one out and which file to open.
            Finding::ScopeNearDuplicate { rare, common, tags } => write!(
                f,
                "scope near-duplicate: {:?} is declared by one rule and is a slip away from {:?}, which is established — check {} for a typo, because a misspelled audience does not fail, it just serves nobody",
                rare.as_str(),
                common.as_str(),
                join_tags(tags)
            ),
            Finding::HomeSlugCollision { slug, tags } => write!(
                f,
                "home-slug collision: rules with different homes share slug {slug:?} and would collide on one file: {}",
                join_tags(tags)
            ),
            Finding::DanglingReference { from, to } => write!(
                f,
                "dangling reference: {} cites {}, which is not a rule in this library",
                from.as_str(),
                to.as_str()
            ),
            Finding::RetiredReference { from, to, status } => write!(
                f,
                "retired reference: {} cites {}, which is {status} — verify the citation is intentional",
                from.as_str(),
                to.as_str()
            ),
            Finding::RecurrenceAfterGraduation {
                tag,
                to,
                graduated,
                recurred,
            } => write!(
                f,
                "recurrence after graduation: {} graduated to {to} on {graduated}, and the error class recurred on {recurred} — after the stronger control was in place. Every emitted layer is telling readers it is \"Also enforced by {to}\", which this disproves: repair the control or withdraw the claim",
                tag.as_str()
            ),
            // States what the finding *means*, not what it found. The count is
            // evidence; the actionable fact is that prose is the only thing
            // holding this rule and prose has already been shown to fail.
            Finding::UnheldRecurrence {
                tag,
                status,
                times,
                latest,
            } => {
                let holding = if *status == "partial" {
                    "so prose alone holds the part its controls do not"
                } else {
                    "so prose is the only thing holding it"
                };
                let remedy = if *status == "partial" {
                    "Extend its controls over the uncovered part, or narrow what `uncovered` claims is still exposed"
                } else {
                    "Promote it to a control that can hold it (a type, a property test, a gate check) and record that with `status = { kind = \"graduated\", to = \"...\" }`"
                };
                write!(
                    f,
                    "unheld recurrence: {} is `{status}`, {holding} — and it has fired {times} more time(s) since it was written, most recently {latest}. {remedy}",
                    tag.as_str()
                )
            }
        }
    }
}

fn join_tags(tags: &[RuleTag]) -> String {
    tags.iter()
        .map(RuleTag::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}

/// Run every deterministic check over the library, returning findings sorted
/// most-severe first (stable within a severity).
#[must_use]
pub fn lint(library: &Library<Validated>) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(overlapping_scope(library));
    findings.extend(scope_near_duplicates(library));
    findings.extend(home_slug_collisions(library));
    findings.extend(reference_checks(library));
    findings.extend(unheld_recurrences(library));
    findings.extend(recurrence_after_graduation(library));
    // Most-severe first; a stable sort keeps each check's own deterministic
    // order within a severity.
    findings.sort_by_key(|finding| std::cmp::Reverse(finding.severity()));
    findings
}

/// The furthest apart two spellings may be and still be read as a slip.
///
/// §12.5 says edit distance 2, and that alone is wrong in a way the vocabulary
/// it governs makes immediate: `rust` and `ruby` are two edits apart and are two
/// languages, and **any** two two-letter scopes — `go` and `js` — are within two
/// of each other by arithmetic rather than by error. A check firing on those is
/// the false positive that gets the whole thing muted.
///
/// So distance is read **relative to length**: a slip must be small compared to
/// the word it damages. Two edits therefore require a scope of at least five
/// characters, one edit at least three, which admits `low-latency` against
/// `low-latencv` and refuses `rust` against `ruby`. The design's number is kept
/// as the ceiling; what is added is the floor it needed.
const NEAR_ENOUGH: usize = 2;

/// Scopes that look like a slip of an established one (§12.5).
///
/// **This is what stands in for a vocabulary curator.** The set of audiences in
/// use is whatever the corpus declares — one identity space rather than a
/// controlled list somebody owns — so drift cannot be prevented at the
/// perimeter, only detected. `ScopeTag` already kills the malformed cases at
/// parse time; what survives is the well-formed near-miss, which is silent:
/// nothing fails, and the rule serves an audience nobody asks for.
///
/// Reported only when one spelling is used **once** and the other by **more**.
/// Two scopes each used once are two scopes: there is no established spelling to
/// have drifted from, and deciding which of the two was intended is the
/// judgement this check exists not to make.
fn scope_near_duplicates(library: &Library<Validated>) -> Vec<Finding> {
    let mut users: BTreeMap<&ScopeTag, Vec<RuleTag>> = BTreeMap::new();
    for rule in library.rules() {
        for scope in rule.applies_to() {
            users.entry(scope).or_default().push(rule.tag().clone()); // allow:clone: the finding owns its tags, outliving the &Library borrow
        }
    }

    let mut findings = Vec::new();
    for (rare, tags) in &users {
        if tags.len() != 1 {
            continue;
        }
        for (common, common_tags) in &users {
            if common_tags.len() <= tags.len() || !is_a_slip(rare.as_str(), common.as_str()) {
                continue;
            }
            findings.push(Finding::ScopeNearDuplicate {
                rare: (*rare).clone(), // allow:clone: the finding owns the scopes it names, outliving the borrow of the library they were read from
                common: (*common).clone(), // allow:clone: as above
                tags: tags.clone(),    // allow:clone: as above
            });
        }
    }
    findings
}

/// Whether `a` reads as a slip of `b`: close in absolute terms, and close
/// *relative to* the shorter of the two. See [`NEAR_ENOUGH`] for why the second
/// condition is not optional.
fn is_a_slip(a: &str, b: &str) -> bool {
    let distance = edit_distance(a, b);
    let shortest = a.chars().count().min(b.chars().count());
    distance > 0 && distance <= NEAR_ENOUGH && distance * 2 < shortest
}

/// Levenshtein distance, by the usual two-row table.
///
/// Written here rather than taken as a dependency: twenty lines against a crate
/// that would need pricing in the decisions log (`[R:price-every-dependency]`),
/// for a function whose behaviour is fully specified by four test cases.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    let mut current = vec![0; b.len() + 1];

    for (i, from) in a.iter().enumerate() {
        current[0] = i + 1;
        for (j, to) in b.iter().enumerate() {
            let substitution = previous[j] + usize::from(from != to);
            current[j + 1] = substitution.min(previous[j + 1] + 1).min(current[j] + 1);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

/// Rules that declare the same error class (case-insensitive), grouped.
fn overlapping_scope(library: &Library<Validated>) -> Vec<Finding> {
    let mut by_class: BTreeMap<String, (String, Vec<RuleTag>)> = BTreeMap::new();
    for rule in library.rules() {
        let key = rule.error_class().match_key();
        let entry = by_class
            .entry(key)
            .or_insert_with(|| (rule.error_class().as_str().to_owned(), Vec::new()));
        entry.1.push(rule.tag().clone()); // allow:clone: the finding owns its tags, outliving the &Library borrow
    }
    by_class
        .into_values()
        .filter(|(_, tags)| tags.len() >= 2)
        .map(|(error_class, tags)| Finding::OverlappingScope { error_class, tags })
        .collect()
}

/// Distinct homes whose slugs collide (would emit to the same skill path).
fn home_slug_collisions(library: &Library<Validated>) -> Vec<Finding> {
    let mut by_slug: BTreeMap<String, Vec<(RuleTag, &Home)>> = BTreeMap::new();
    for rule in library.rules() {
        let slug = HomeSlug::of(rule.home()).as_str().to_owned();
        by_slug
            .entry(slug)
            .or_default()
            .push((rule.tag().clone(), rule.home())); // allow:clone: the finding owns its tags, outliving the &Library borrow
    }
    by_slug
        .into_iter()
        .filter_map(|(slug, entries)| {
            let first_home = entries.first().map(|(_, home)| *home)?;
            let collides = entries.iter().any(|(_, home)| *home != first_home);
            collides.then(|| Finding::HomeSlugCollision {
                slug,
                tags: entries.into_iter().map(|(tag, _)| tag).collect(),
            })
        })
        .collect()
}

/// Check every `R:...` citation (in a rule's body or incident): a tag absent
/// from the library is a `DanglingReference`; a tag that resolves to a *retired*
/// (atticked/graduated) rule is a `RetiredReference`; a citation of an active
/// rule is fine.
fn reference_checks(library: &Library<Validated>) -> Vec<Finding> {
    let by_tag: BTreeMap<&str, &Rule> = library
        .rules()
        .iter()
        .map(|r| (r.tag().as_str(), r))
        .collect();
    let mut findings = Vec::new();
    for rule in library.rules() {
        for cited in cited_tags(rule) {
            if cited.as_str() == rule.tag().as_str() {
                continue; // a self-citation is not a cross-reference
            }
            match by_tag.get(cited.as_str()) {
                None => findings.push(Finding::DanglingReference {
                    from: rule.tag().clone(), // allow:clone: the finding owns its tags, outliving the &Library borrow
                    to: cited,
                }),
                Some(target) => {
                    if target.status().prose_coverage() == ProseCoverage::None {
                        findings.push(Finding::RetiredReference {
                            from: rule.tag().clone(), // allow:clone: the finding owns its tags, outliving the &Library borrow
                            to: cited,
                            status: status_kind(target.status()),
                        });
                    }
                }
            }
        }
    }
    findings
}

/// Rules whose prose still holds them — wholly (`Active`), or the part no named
/// control covers (`Partial`) — and which have recurred since they were written.
///
/// **No count threshold.** The first recurrence already proves the prose failed;
/// two is not more actionable than one, and any cut-off would be a magic number
/// this codebase does not use. A *graduated* rule with recurrences is
/// deliberately not flagged here: that is the sharper finding — a named stronger
/// control that demonstrably did not hold — and it needs a graduation date this
/// `Status` does not carry, so it is a separate change with its own argument.
fn unheld_recurrences(library: &Library<Validated>) -> Vec<Finding> {
    library
        .rules()
        .iter()
        .filter(|rule| rule.status().prose_coverage() == ProseCoverage::Holds)
        .filter_map(|rule| {
            let latest = rule.latest_recurrence()?;
            Some(Finding::UnheldRecurrence {
                tag: rule.tag().clone(), // allow:clone: the finding owns its tag, outliving the &Library borrow
                status: status_kind(rule.status()),
                times: rule.recurrences().len(),
                latest,
            })
        })
        .collect()
}

/// Graduated rules whose error class recurred **after** the graduation date.
///
/// The date is what makes this answerable at all, and it is the whole reason
/// `Status::Graduated` carries one. A recurrence *before* the graduation is not
/// a finding — it is very often the incident that prompted the graduation, and
/// flagging it would punish exactly the response the library wants. Only a
/// recurrence strictly after the date says the stronger control did not hold.
///
/// A `Partial` rule is deliberately never reported here, which is why this reads
/// [`Status::whole_class_claim`] rather than matching `Graduated` directly. Its
/// controls never claimed the uncovered part, so a recurrence there is the prose
/// failing — reported by `unheld_recurrences` as a warning — and not a named
/// control lying, which is an `Error` that fails CI. Recording an honest partial
/// graduation must never turn a warning into a build failure.
fn recurrence_after_graduation(library: &Library<Validated>) -> Vec<Finding> {
    library
        .rules()
        .iter()
        .filter_map(|rule| {
            let (to, date) = rule.status().whole_class_claim()?;
            let recurred = rule
                .recurrences()
                .iter()
                .map(crate::rule::Recurrence::date)
                .filter(|d| d > date)
                .max()?;
            Some(Finding::RecurrenceAfterGraduation {
                tag: rule.tag().clone(), // allow:clone: the finding owns its tag, outliving the &Library borrow
                to: to.as_str().to_owned(),
                graduated: *date,
                recurred,
            })
        })
        .collect()
}

/// The two numbers that make the recurrence count honest, and the reason
/// [`crate::rule::Origin`] exists.
///
/// P5: every metric carries a counter-metric. *Recurred* is the metric — and it
/// is gameable through under-reporting, because only someone willing to write
/// down that their own rule failed ever increments it. *Inert* is the counter:
/// rules that were authored rather than mined from a real failure **and** have
/// never been seen to fire. The null result says those are the ones that change
/// nothing, so a library that looks healthy because it is full of them is
/// exactly what a bare recurrence count would hide. Read together or not at all.
///
/// **Mandated rules are outside both numbers**, and the count of them is
/// reported so the denominator is visible rather than assumed. A mandate was
/// never mined, so it cannot be recurrence evidence; and it is not *inert*
/// either, because inert means "authored and never fired" — a judgement about
/// something that was meant to be evidence. Counting control-framework
/// requirements in either figure would swamp the only number that says whether
/// prose is holding, with rules that were never about that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tally {
    total: usize,
    recurred: usize,
    inert: usize,
    mandated: usize,
}

impl Tally {
    /// Every rule in the library.
    #[must_use]
    pub fn total(self) -> usize {
        self.total
    }

    /// Rules with at least one recorded recurrence — the metric.
    #[must_use]
    pub fn recurred(self) -> usize {
        self.recurred
    }

    /// Rules codified rather than mined, that have never recurred — the counter.
    #[must_use]
    pub fn inert(self) -> usize {
        self.inert
    }

    /// Rules mandated rather than learned — **excluded** from both numbers
    /// above, and reported so that exclusion is visible.
    #[must_use]
    pub fn mandated(self) -> usize {
        self.mandated
    }

    /// The population the two figures above are actually about: every rule that
    /// was meant to be evidence, mandates removed. The honest denominator, and
    /// the one a reader needs in order to turn either count into a fraction.
    #[must_use]
    pub fn evidential(self) -> usize {
        self.total - self.mandated
    }
}

/// Count the library's recurred and inert fractions, and the mandated rules
/// held out of both.
#[must_use]
pub fn tally(library: &Library<Validated>) -> Tally {
    let rules = library.rules();
    Tally {
        total: rules.len(),
        // Both filters go through `counts_toward_recurrence_statistics` rather
        // than testing the origin here, so there is one definition of what
        // belongs in the statistics and no second copy to drift from it.
        recurred: rules
            .iter()
            .filter(|r| r.counts_toward_recurrence_statistics() && r.has_recurred())
            .count(),
        inert: rules.iter().filter(|r| r.is_inert()).count(),
        mandated: rules
            .iter()
            .filter(|r| !r.counts_toward_recurrence_statistics())
            .count(),
    }
}

/// A stable label for a status kind, for diagnostics.
fn status_kind(status: &Status) -> &'static str {
    match status {
        Status::Active => "active",
        Status::Partial { .. } => "partial",
        Status::Graduated { .. } => "graduated",
        Status::Attic { .. } => "atticked",
    }
}

/// Every well-formed `R:...` tag cited in a rule's **body**, each once, sorted
/// for determinism.
///
/// The `incident` field is deliberately NOT scanned. It is provenance, and
/// provenance legitimately names tags that are retired, renamed, or owned by
/// another library — "retagged from R:x", "supersedes R:y". Those are
/// historical mentions, not live citations. Scanning them made the linter flag
/// its own provenance: documentation *about* a tag read as a citation *of* it,
/// which is exactly `[R:detector-excludes-own-definitions]`. An always-warning
/// linter gets muted, and a muted check is worse than none.
///
/// Cost asymmetry settles the scope: a citation missed in provenance is
/// harmless, a false dangling-reference is permanent noise.
///
/// Incident 2026-08-16: splitting `R:revision-integrity` into `R:doc-currency`,
/// the new rule's incident recorded "retagged from <the old tag>" and lint
/// reported a dangling reference to a tag appearing nowhere but provenance —
/// so the wording had to be contorted to silence a false positive.
pub fn cited_tags(rule: &Rule) -> Vec<RuleTag> {
    let mut out = Vec::new();
    collect_tag_tokens(rule.body().as_str(), &mut out);
    out.sort();
    out.dedup();
    out
}

/// Scan `text` for `R:[a-z0-9][a-z0-9-]*` tokens and push each that parses.
fn collect_tag_tokens(text: &str, out: &mut Vec<RuleTag>) {
    let bytes = text.as_bytes();
    let mut search_from = 0;
    while let Some(rel) = text[search_from..].find("R:") {
        let start = search_from + rel;
        let body_start = start + 2;
        // Require a boundary before `R:` so "OR:"/"FOR:" in prose don't match.
        let preceded_by_alnum = start > 0 && bytes[start - 1].is_ascii_alphanumeric();
        let mut end = body_start;
        if !preceded_by_alnum {
            while end < bytes.len() {
                let c = bytes[end];
                if c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' {
                    end += 1;
                } else {
                    break;
                }
            }
            // `R:` and the body chars are all ASCII, so these byte indices are
            // valid char boundaries for slicing.
            if let Ok(tag) = RuleTag::parse(&text[start..end]) {
                out.push(tag);
            }
        }
        search_from = body_start.max(end);
    }
}
