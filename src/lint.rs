//! `lint` — advisory static analysis over a validated library. Findings are
//! *reported*, never acted on: a flagged rule is input to a human cut-list
//! decision, and the tool must never auto-delete a correction (that would be the
//! sediment-in-reverse this project exists to prevent).
//!
//! The checks here are the deterministic ones — overlapping scope, home-slug
//! collision, and dangling in-library references. Two Phase-B checks are
//! deliberately absent because faking them would be dishonest:
//! - **contradiction** between rules is a semantic judgment (delegate to Claude,
//!   never a keyword heuristic dressed up as certainty);
//! - **cold-surface** (a rule nothing exercises) needs runtime invocation data,
//!   which lives in the stochos-lab ledger, not in the rule text.
//!
//! **Must NOT:** modify or delete rules, or perform I/O. It reads a library and
//! returns findings; the caller decides what to do with them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::emit::HomeSlug;
use crate::library::{Library, Validated};
use crate::rule::{Home, Rule, RuleTag};

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
    findings.extend(dangling_references(library));
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

/// Rules that cite an `R:...` tag (in body or incident) absent from the library.
fn dangling_references(library: &Library<Validated>) -> Vec<Finding> {
    let existing: BTreeSet<&str> = library.rules().iter().map(|r| r.tag().as_str()).collect();
    let mut findings = Vec::new();
    for rule in library.rules() {
        for cited in cited_tags(rule) {
            if cited.as_str() != rule.tag().as_str() && !existing.contains(cited.as_str()) {
                findings.push(Finding::DanglingReference {
                    from: rule.tag().clone(), // allow:clone: the finding owns its tags, outliving the &Library borrow
                    to: cited,
                });
            }
        }
    }
    findings
}

/// Every well-formed `R:...` tag mentioned in a rule's body or incident.
fn cited_tags(rule: &Rule) -> Vec<RuleTag> {
    let mut out = Vec::new();
    collect_tag_tokens(rule.body().as_str(), &mut out);
    collect_tag_tokens(rule.incident().as_str(), &mut out);
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
