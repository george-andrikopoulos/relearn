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
use crate::rule::{Home, Rule, RuleTag, Status};

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
}

impl Finding {
    /// The severity of this finding.
    #[must_use]
    pub fn severity(&self) -> Severity {
        match self {
            Finding::HomeSlugCollision { .. } => Severity::Error,
            Finding::OverlappingScope { .. } | Finding::DanglingReference { .. } => {
                Severity::Warning
            }
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
    findings.extend(home_slug_collisions(library));
    findings.extend(reference_checks(library));
    // Most-severe first; a stable sort keeps each check's own deterministic
    // order within a severity.
    findings.sort_by_key(|finding| std::cmp::Reverse(finding.severity()));
    findings
}

/// Rules that declare the same error class (case-insensitive), grouped.
fn overlapping_scope(library: &Library<Validated>) -> Vec<Finding> {
    let mut by_class: BTreeMap<String, (String, Vec<RuleTag>)> = BTreeMap::new();
    for rule in library.rules() {
        let key = rule.error_class().as_str().to_lowercase();
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
                    if !matches!(target.status(), Status::Active) {
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

/// A stable label for a status kind, for diagnostics.
fn status_kind(status: &Status) -> &'static str {
    match status {
        Status::Active => "active",
        Status::Graduated { .. } => "graduated",
        Status::Attic { .. } => "atticked",
    }
}

/// Every well-formed `R:...` tag mentioned in a rule's body or incident, each
/// once. Deduplicated (and sorted, for determinism) so a rule that cites the
/// same tag in both its body and its incident yields a single finding, not two.
fn cited_tags(rule: &Rule) -> Vec<RuleTag> {
    let mut out = Vec::new();
    collect_tag_tokens(rule.body().as_str(), &mut out);
    collect_tag_tokens(rule.incident().as_str(), &mut out);
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
