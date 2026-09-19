//! `emit` — one submodule per target assistant. Each emitter is a pure
//! function `&Library<Validated> -> Vec<OutputFile>`: it is handed already
//! validated rules and returns values for the caller (`fsio`, via `cli`) to
//! write. Emitters never perform the write themselves, which is what makes
//! them trivially unit-testable without a filesystem.
//!
//! Each [`OutputFile`] carries the rules that produced it ([`OutputFile::sources`]);
//! the generated-by header and content hash are rendered and appended by `fsio`
//! at write time (decision 2026-08-13), so the marker the overwrite guard reads
//! back lives in exactly one place. Emitters stay pure body-producers.
//!
//! **Must NOT:** read the filesystem, compute the hash / render the header, or
//! read anything not carried by the rule. If a target needs data, the data
//! belongs in the rule, not in the emitter.
//!
//! This module root holds the types every emitter shares: [`OutputFile`] (what
//! an emitter produces), [`RelativePath`] (where, relative to the output root),
//! and [`HomeSlug`] (the filesystem-safe identity of a home layer, which is
//! also a valid Claude skill `name`).

pub mod agents;
pub mod claude;
pub mod claude_rules;
pub mod copilot;
pub mod copilot_paths;
pub mod cursor;

use crate::library::{Library, Validated};
use crate::rule::{Emittability, Home, Rule, RuleTag, ScopeTag, Status};

/// A path relative to the output root, in portable forward-slash form. Built
/// only from known-safe segments inside this crate (`pub(crate)` constructor),
/// so it never carries untrusted, absolute, or `..`-bearing input; `fsio` joins
/// it onto the chosen output directory at write time.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RelativePath(String);

impl RelativePath {
    /// Join ordered path segments with `/`. Internal to `emit` — callers pass
    /// literal, already-safe segments (`"skills"`, a [`HomeSlug`], `"SKILL.md"`).
    #[must_use]
    pub(crate) fn from_segments(segments: &[&str]) -> Self {
        RelativePath(segments.join("/"))
    }

    /// A relative path already in portable forward-slash form. Used by `fsio`
    /// when it discovers a generated file on disk (orphan detection) and needs to
    /// compare its path against the emitters' output paths, which are also
    /// forward-slash. Internal to the crate; the caller normalizes separators.
    #[must_use]
    pub(crate) fn from_forward_slash(s: impl Into<String>) -> Self {
        RelativePath(s.into())
    }

    /// The path text, forward-slash separated.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A file an emitter wants written: where (relative to the output root), the
/// body content, and which rules produced it. An emitter returns these; it does
/// not write them — the write, the generated-by header, and the content hash
/// are all `fsio`'s single audited responsibility. `sources` is the provenance
/// `fsio` serializes into that header (it is emit's knowledge, not fsio's).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFile {
    path: RelativePath,
    contents: String,
    sources: Vec<RuleTag>,
}

impl OutputFile {
    /// Assemble an output file. Internal to `emit`. `sources` are the tags of
    /// the rules that produced `contents`, in a deterministic order.
    #[must_use]
    pub(crate) fn new(path: RelativePath, contents: String, sources: Vec<RuleTag>) -> Self {
        OutputFile {
            path,
            contents,
            sources,
        }
    }

    /// Where to write, relative to the output root.
    #[must_use]
    pub fn path(&self) -> &RelativePath {
        &self.path
    }

    /// The body content, before `fsio` appends the generated-by header.
    #[must_use]
    pub fn contents(&self) -> &str {
        &self.contents
    }

    /// The tags of the rules that produced this file — the provenance the
    /// overwrite guard's header records.
    #[must_use]
    pub fn sources(&self) -> &[RuleTag] {
        &self.sources
    }
}

/// A filesystem-safe identity for a home layer: `[a-z0-9-]+`, derived from a
/// [`Home`]. It names the skill directory (`skills/<slug>/`) and doubles as the
/// Claude skill `name` (which has the same shape), so one type serves both.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HomeSlug(String);

impl HomeSlug {
    /// Derive the slug for a home. `Global` → `global`; a domain or project
    /// carries its name/path slugified (lowercased, non-alphanumerics collapsed
    /// to single `-`), so `project = "C:/repo"` becomes `project-c-repo`.
    #[must_use]
    pub fn of(home: &Home) -> Self {
        let slug = match home {
            Home::Global => "global".to_owned(),
            Home::Org { name } => format!("org-{}", slugify(name.as_str())),
            Home::Domain { name } => format!("domain-{}", slugify(name.as_str())),
            Home::Project { path } => format!("project-{}", slugify(path.as_str())),
        };
        HomeSlug(slug)
    }

    /// The slug text, shape `[a-z0-9-]+`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Lowercase `s` and collapse every run of non-alphanumeric characters to a
/// single `-`, with no leading or trailing `-`. Turns free-form domain names
/// and project paths into a single filesystem- and skill-name-safe token.
fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut pending_dash = false;
    for ch in s.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_dash = true;
        }
    }
    out
}

/// Quote `s` as a YAML double-quoted scalar, escaping the characters that would
/// otherwise break it. Shared by every emitter whose target parses its
/// front-matter as **strict** YAML (Claude skills) and so needs a rule title
/// or error class containing `:` or `"` kept safe. Targets with a lenient
/// line-based front-matter (Cursor `.mdc`) deliberately do not use this.
#[must_use]
pub(crate) fn yaml_double_quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// The glob set a [`LoadSemantics::WhenReading`] scope attaches to — **non-empty
/// by construction**.
///
/// The private field and fallible constructor are the point. An empty glob list
/// is not a narrower scope, it is the *absence* of one, and the two readings of
/// it diverge silently across targets: Cursor treats a blank `globs` line as "no
/// auto-attach", while a Claude rule file with `paths: []` loads at
/// `session_start` — measured 2026-08-22, i.e. resident in **every** session, the
/// exact opposite. A `(globs, always_apply)` pair let that state be built; this
/// type does not.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Globs(Vec<String>);

impl Globs {
    /// Build a glob set, rejecting the empty one. `None` means "this home has no
    /// file globs", and the caller must choose a different [`LoadSemantics`]
    /// variant rather than carry an empty list.
    #[must_use]
    pub fn new(globs: Vec<String>) -> Option<Self> {
        if globs.is_empty() {
            None
        } else {
            Some(Globs(globs))
        }
    }

    /// The globs — always at least one.
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }
}

/// How a rule's [`Home`] translates into a target's **load model**. The neutral
/// concept is `Home`; turning it into Cursor's `globs`/`alwaysApply`, or a Claude
/// rule file's `paths:`, is the emitter's job, and that translation lives here —
/// in one place — rather than being re-derived per emitter or, worse, carried as
/// vendor-specific fields on the rule (decision 2026-08-13).
///
/// **Why an enum, not a `(globs, always_apply)` pair** (decision 2026-08-22). The
/// pair carried three real states in two fields, so a fourth combination — no
/// globs and not always-on — was representable and meant something different
/// again. Every target had to re-derive which of the three it was holding, and a
/// target that could not express one of them had no way to say so: it rendered
/// something plausible instead. That is how a rule meant to be reachable only by
/// description would have become a rule resident in every session on the machine.
/// As variants, a target that cannot represent a case must match it and decide,
/// and a fourth load model added later is a compile error at every emitter rather
/// than a silent default. `[R:prefer-by-construction]`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadSemantics {
    /// Resident in every session, whatever files are touched: `Global` (applies
    /// everywhere) and `Project` (applies throughout its own tree).
    Always,
    /// Attaches when the assistant reads a file matching one of these globs — a
    /// known language domain, `rust` → `**/*.rs`.
    WhenReading(Globs),
    /// Reachable only when the assistant asks for it by description; never
    /// blanket-applied. An unknown language domain, for which no glob is known.
    OnRequest,
}

impl LoadSemantics {
    /// Derive the load model for a home:
    /// - `Global` / `Project` → [`Always`](LoadSemantics::Always).
    /// - `Domain` with a known language → [`WhenReading`](LoadSemantics::WhenReading)
    ///   that language's globs.
    /// - `Domain` with an unknown language → [`OnRequest`](LoadSemantics::OnRequest),
    ///   never silently everywhere.
    #[must_use]
    pub fn for_home(home: &Home) -> Self {
        match home {
            // An organisation's own principles bind every language and every
            // tree inside it, so the load model is `Global`'s rather than a
            // domain's: there is no file extension that means "this belongs to
            // the company", and `OnRequest` would make the layer a corporation
            // fills the one layer nothing loads.
            Home::Global | Home::Org { .. } | Home::Project { .. } => LoadSemantics::Always,
            Home::Domain { name } => match Globs::new(domain_globs(name.as_str())) {
                Some(globs) => LoadSemantics::WhenReading(globs),
                None => LoadSemantics::OnRequest,
            },
        }
    }
}

/// The file globs for a language domain. An unknown domain returns no globs —
/// the rule then reaches the assistant by description, never blanket-applied.
/// Language knowledge lives here so every target that scopes by language reads
/// one table.
fn domain_globs(name: &str) -> Vec<String> {
    let patterns: &[&str] = match name.to_ascii_lowercase().as_str() {
        "rust" => &["**/*.rs"],
        "python" | "py" => &["**/*.py"],
        "typescript" | "ts" => &["**/*.ts", "**/*.tsx"],
        "javascript" | "js" => &["**/*.js", "**/*.jsx"],
        "go" | "golang" => &["**/*.go"],
        "java" => &["**/*.java"],
        "c" => &["**/*.c", "**/*.h"],
        "cpp" | "c++" => &["**/*.cpp", "**/*.hpp", "**/*.cc", "**/*.hh"],
        "ruby" | "rb" => &["**/*.rb"],
        _ => &[],
    };
    patterns.iter().map(|p| (*p).to_owned()).collect()
}

/// A stable ordering rank for a home, general → specific: `Global` (0) before
/// `Domain` (1) before `Project` (2). Emitters that concatenate every home into
/// one file (Copilot, `AGENTS.md`) order by this rank first — so the broadest
/// rules lead — then by home slug (same-home rules stay grouped), then by tag.
#[must_use]
pub(crate) fn home_rank(home: &Home) -> u8 {
    match home {
        Home::Global => 0,
        // Between global and domain: an organisation's principles are narrower
        // than everyone's and broader than one language's, and a reader should
        // meet them before the language-specific rules that sit inside them.
        Home::Org { .. } => 1,
        Home::Domain { .. } => 2,
        Home::Project { .. } => 3,
    }
}

/// The rules a library permits into the instruction layer, in library order:
/// every rule whose [`Status::emittability`] is `Emit` (active + graduated),
/// with atticked rules filtered out. The single place the emit-status policy is
/// applied — every emitter starts here rather than from `library.rules()`, so
/// "withdrawn guidance never leaks" is enforced once, not re-derived five times.
/// The full library (including atticked rules) stays available to `lint`, which
/// needs retired rules present to flag references to them.
#[must_use]
pub(crate) fn emittable(library: &Library<Validated>) -> Vec<&Rule> {
    library
        .rules()
        .iter()
        .filter(|r| r.status().emittability() == Emittability::Emit)
        .collect()
}

/// The one-line annotation a graduated rule carries in every emitted format: a
/// markdown blockquote naming the stronger control that *also* holds the
/// guarantee, so a reader of the instruction layer knows the prose is
/// belt-and-suspenders rather than the sole enforcement (decision 2026-08-13).
/// `None` for any non-graduated rule — active rules need no note, and atticked
/// rules never reach an emitter. Returns the note plus its trailing blank line,
/// so a caller splices it between a rule's heading and body with one `push_str`.
#[must_use]
pub(crate) fn enforcement_note(rule: &Rule) -> Option<String> {
    match rule.status() {
        Status::Graduated { to, .. } => Some(format!("> Also enforced by {to}.\n\n")),
        Status::Partial { by, uncovered, .. } => Some(format!(
            "> Partly enforced by {}; {} is held by this instruction alone.\n\n",
            by,
            uncovered.as_str()
        )),
        Status::Active | Status::Attic { .. } => None,
    }
}

/// The one-line annotation a rule that has **recurred** carries in every emitted
/// format: a markdown blockquote counting the later occurrences and dating the
/// most recent one. `None` for a rule with no recurrences, so an unrecurred rule
/// emits exactly what it emitted before the field existed.
///
/// Same category, same place, same reason as [`enforcement_note`]. An assistant
/// reading the corpus should weight a rule that has bitten three times above one
/// written once and never seen again; without this the two are indistinguishable
/// in the instruction layer, which is where the weighting actually happens.
/// Returns the note plus its trailing blank line, so a caller splices it in with
/// one `push_str`.
#[must_use]
pub(crate) fn recurrence_note(rule: &Rule) -> Option<String> {
    let latest = rule.latest_recurrence()?;
    Some(format!(
        "> Has recurred {} time(s) since it was written; most recently {latest}.\n\n",
        rule.recurrences().len()
    ))
}

/// The one-line annotation a **scoped** rule carries in every emitted format: a
/// markdown blockquote naming the audiences the rule was written for. `None` for
/// a rule that declares no `applies_to`, so an unscoped rule emits exactly what
/// it emitted before this existed — and, more importantly, says nothing it has
/// not been told. A rule with no audience is emitted under *every* audience;
/// annotating it "applies to all" would be a claim the rule does not make.
///
/// Third of the same family as [`enforcement_note`] and [`recurrence_note`], and
/// the last of the three by rank on purpose: the reader's first question about a
/// rule is whether it is theirs, the second is how firmly it is held, the third
/// is whether it has bitten. Returns the note plus its trailing blank line, so a
/// caller splices it in with one `push_str`.
///
/// Decided 2026-09-14. Before it, a scoped rule and an unscoped one emitted
/// byte-identically, which made the old
/// `scope_never_reaches_an_emitted_path_or_body` property easy to state and left
/// a reader of `skills/domain-low-latency/SKILL.md` with no way to tell that the
/// rule in front of them was written for Rust and Java engineers in particular.
/// The path half of that property is untouched and still holds: `Home` alone
/// decides where a rule lands.
#[must_use]
pub(crate) fn audience_note(rule: &Rule) -> Option<String> {
    let names: Vec<&str> = rule.applies_to().iter().map(ScopeTag::as_str).collect();
    let list = match names.as_slice() {
        [] => return None,
        [only] => (*only).to_owned(),
        [rest @ .., last] => format!("{} and {last}", rest.join(", ")),
    };
    let noun = if names.len() == 1 {
        "audience"
    } else {
        "audiences"
    };
    Some(format!("> Written for the {list} {noun}.\n\n"))
}

/// The one-line annotation a **codified** rule that names its artefact carries
/// in every emitted format: a markdown blockquote naming the document the
/// practice was written down from. `None` for every other origin, and for a
/// codified rule that names none — so a rule that cites nothing emits exactly
/// what it emitted before this existed.
///
/// Fourth of the same family as [`enforcement_note`], [`recurrence_note`] and
/// [`audience_note`], and **last by rank on purpose**. The first three answer
/// operational questions in the order a reader asks them — is this rule mine,
/// how firmly is it held, has it bitten before. Provenance answers none of
/// those: it is what a reader follows when they want to check the rule against
/// the thing that defines it, which is the question asked last and least often.
/// `[R:order-by-explicit-rank]` — the position is a decision, not where it
/// happened to land.
///
/// **This is the only channel from a source into emitted text**, which
/// `a_source_reaches_a_body_only_through_the_source_note` holds: provenance
/// annotates, it never rewrites a body, reorders around one, or decides whether
/// a rule is emitted at all. Returns the note plus its trailing blank line, so a
/// caller splices it in with one `push_str`.
#[must_use]
pub(crate) fn source_note(rule: &Rule) -> Option<String> {
    rule.origin()
        .source()
        .map(|s| format!("> Written down from {}.\n\n", s.as_str()))
}

/// The human-readable label for a home, used in headings and skill
/// descriptions — `global`, `domain: rust`, `project: relearn`.
///
/// Shared rather than duplicated: it was private to `emit::claude` until
/// `emit::copilot_paths` needed the same phrasing for the same purpose, and two
/// copies of a label become two labels the day one of them is edited.
/// Exhaustive over `Home` with no catch-all, so a new home kind has to be given
/// a name here rather than acquiring one by default.
#[must_use]
pub(crate) fn home_label(home: &Home) -> String {
    match home {
        Home::Global => "global".to_owned(),
        Home::Org { name } => format!("org: {}", name.as_str()),
        Home::Domain { name } => format!("domain: {}", name.as_str()),
        Home::Project { path } => format!("project: {}", path.as_str()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Library;
    use crate::rule::{
        Authority, Body, Date, ErrorClass, Home, Incident, Origin, Recurrence, RuleTag, ScopeTag,
        Status, Title,
    };

    fn rule_with_recurrences(tag: &str, status: Status, recurrences: Vec<Recurrence>) -> Rule {
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::global(),
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            status,
            Incident::parse("an incident").expect("non-empty incident"),
            Body::parse("Do the thing.").expect("non-empty body"),
            recurrences,
            Vec::new(),
            Authority::local(),
            None,
        )
    }

    fn recurrence(date: &str) -> Recurrence {
        Recurrence::new(
            Date::parse(date).expect("valid date"),
            Incident::parse("it happened again").expect("non-empty incident"),
        )
    }

    fn rule_with_status(tag: &str, status: Status) -> Rule {
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::global(),
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            status,
            Incident::parse("an incident").expect("non-empty incident"),
            Body::parse("Do the thing.").expect("non-empty body"),
            Vec::new(),
            Vec::new(),
            Authority::local(),
            None,
        )
    }

    #[test]
    fn emittable_keeps_active_and_graduated_and_drops_attic() {
        let date = Date::parse("2026-09-01").expect("valid date");
        let lib = Library::from_rules(vec![
            rule_with_status("R:active", Status::active()),
            rule_with_status(
                "R:grad",
                Status::graduated("hook:x", Date::parse("2026-07-21").expect("valid date"))
                    .expect("non-empty destination"),
            ),
            rule_with_status(
                "R:attic",
                Status::attic("retired", date).expect("non-empty reason"),
            ),
        ])
        .validate()
        .expect("distinct tags validate");

        let kept: Vec<&str> = emittable(&lib).iter().map(|r| r.tag().as_str()).collect();
        assert_eq!(kept, vec!["R:active", "R:grad"]);
    }

    #[test]
    fn a_partial_rule_is_annotated_with_both_halves() {
        // The reader of an instruction layer must be able to tell "a control
        // holds this" from "a control holds HALF of this", because the second
        // means they are still the control for the other half. One sentence
        // carries both, and the uncovered part is in it by construction.
        let note = enforcement_note(&rule_with_status(
            "R:p",
            Status::partial(
                "test:ledger.rs::every_enforced_by_row",
                "any guarantee outside FEATURES.md",
                Date::parse("2026-09-16").expect("valid date"),
            )
            .expect("non-empty control and uncovered part"),
        ))
        .expect("a partial rule carries an enforcement note");

        assert_eq!(
            note,
            "> Partly enforced by test:ledger.rs::every_enforced_by_row; any guarantee outside FEATURES.md is held by this instruction alone.\n\n"
        );
        assert!(
            !note.contains("Also enforced by"),
            "a partial graduation must never render as a full one"
        );
    }

    #[test]
    fn enforcement_note_only_annotates_graduated_rules() {
        let date = Date::parse("2026-09-01").expect("valid date");
        assert_eq!(
            enforcement_note(&rule_with_status("R:a", Status::active())),
            None
        );
        assert_eq!(
            enforcement_note(&rule_with_status(
                "R:x",
                Status::attic("retired", date).expect("non-empty reason"),
            )),
            None
        );
        assert_eq!(
            enforcement_note(&rule_with_status(
                "R:g",
                Status::graduated(
                    "hook:no-unwrap-in-src",
                    Date::parse("2026-07-21").expect("valid date")
                )
                .expect("non-empty destination"),
            )),
            Some("> Also enforced by hook:no-unwrap-in-src.\n\n".to_owned())
        );
    }

    #[test]
    fn recurrence_note_only_annotates_rules_that_have_recurred() {
        assert_eq!(
            recurrence_note(&rule_with_status("R:quiet", Status::active())),
            None
        );
        assert_eq!(
            recurrence_note(&rule_with_recurrences(
                "R:bitten",
                Status::active(),
                vec![recurrence("2026-08-24")],
            )),
            Some(
                "> Has recurred 1 time(s) since it was written; most recently 2026-08-24.\n\n"
                    .to_owned()
            )
        );
    }

    // The note dates the *latest* recurrence, whichever order the file lists
    // them in — a rule read out of the corpus must not claim it last bit in
    // August because August happened to be written last.
    #[test]
    fn recurrence_note_counts_all_and_dates_the_most_recent() {
        assert_eq!(
            recurrence_note(&rule_with_recurrences(
                "R:thrice",
                Status::active(),
                vec![
                    recurrence("2026-08-30"),
                    recurrence("2026-08-16"),
                    recurrence("2026-08-24"),
                ],
            )),
            Some(
                "> Has recurred 3 time(s) since it was written; most recently 2026-08-30.\n\n"
                    .to_owned()
            )
        );
    }

    // A graduated rule that has *also* recurred carries both notes: the
    // stronger control that now holds it, and the record that it bit before.
    // They are independent facts and neither suppresses the other.
    #[test]
    fn graduation_and_recurrence_notes_are_independent() {
        let r = rule_with_recurrences(
            "R:both",
            Status::graduated("hook:x", Date::parse("2026-07-21").expect("valid date"))
                .expect("non-empty destination"),
            vec![recurrence("2026-08-24")],
        );
        assert!(enforcement_note(&r).is_some());
        assert!(recurrence_note(&r).is_some());
    }

    fn rule_scoped(tag: &str, audiences: &[&str]) -> Rule {
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::global(),
            Date::parse("2026-09-14").expect("valid date"),
            Origin::Mined,
            Status::active(),
            Incident::parse("an incident").expect("non-empty incident"),
            Body::parse("Do the thing.").expect("non-empty body"),
            Vec::new(),
            audiences
                .iter()
                .map(|a| ScopeTag::parse(*a).expect("valid scope"))
                .collect(),
            Authority::local(),
            None,
        )
    }

    #[test]
    fn an_unscoped_rule_has_no_audience_note() {
        assert_eq!(audience_note(&rule_scoped("R:none", &[])), None);
    }

    #[test]
    fn one_audience_is_named_in_the_singular() {
        assert_eq!(
            audience_note(&rule_scoped("R:one", &["rust"])),
            Some("> Written for the rust audience.\n\n".to_owned())
        );
    }

    #[test]
    fn two_audiences_are_joined_with_and() {
        assert_eq!(
            audience_note(&rule_scoped("R:two", &["rust", "java"])),
            Some("> Written for the rust and java audiences.\n\n".to_owned())
        );
    }

    #[test]
    fn three_or_more_audiences_are_comma_separated_before_the_last() {
        assert_eq!(
            audience_note(&rule_scoped("R:three", &["rust", "java", "embedded"])),
            Some("> Written for the rust, java and embedded audiences.\n\n".to_owned())
        );
    }

    /// The declared order is the emitted order — `applies_to` is a list the
    /// author wrote, not a set the tool reorders behind them.
    #[test]
    fn the_declared_order_is_the_announced_order() {
        assert_eq!(
            audience_note(&rule_scoped("R:order", &["java", "rust"])),
            Some("> Written for the java and rust audiences.\n\n".to_owned())
        );
    }

    /// All three notes are independent: a rule can be graduated, recurred and
    /// scoped at once, and each annotation is decided by its own field.
    #[test]
    fn the_audience_note_is_independent_of_the_other_two() {
        let scoped = rule_scoped("R:s", &["rust"]);
        assert!(audience_note(&scoped).is_some());
        assert!(enforcement_note(&scoped).is_none());
        assert!(recurrence_note(&scoped).is_none());

        let recurred =
            rule_with_recurrences("R:r", Status::active(), vec![recurrence("2026-08-24")]);
        assert!(audience_note(&recurred).is_none());
        assert!(recurrence_note(&recurred).is_some());
    }

    #[test]
    fn global_slug_is_global() {
        assert_eq!(HomeSlug::of(&Home::global()).as_str(), "global");
    }

    #[test]
    fn domain_slug_is_prefixed() {
        assert_eq!(
            HomeSlug::of(&Home::domain("rust").expect("domain")).as_str(),
            "domain-rust"
        );
    }

    #[test]
    fn project_path_is_made_filesystem_safe() {
        let home = Home::project("C:/repo/sub").expect("project");
        assert_eq!(HomeSlug::of(&home).as_str(), "project-c-repo-sub");
    }

    #[test]
    fn slugify_collapses_and_trims_separators() {
        assert_eq!(slugify("  Foo__Bar!! "), "foo-bar");
        assert_eq!(slugify("a"), "a");
    }

    #[test]
    fn relative_path_joins_with_forward_slash() {
        let p = RelativePath::from_segments(&["skills", "global", "SKILL.md"]);
        assert_eq!(p.as_str(), "skills/global/SKILL.md");
    }

    #[test]
    fn global_scope_is_always() {
        assert_eq!(
            LoadSemantics::for_home(&Home::global()),
            LoadSemantics::Always
        );
    }

    #[test]
    fn project_scope_is_always() {
        assert_eq!(
            LoadSemantics::for_home(&Home::project("C:/repo").expect("non-empty project")),
            LoadSemantics::Always
        );
    }

    #[test]
    fn known_domain_scopes_to_language_globs() {
        let scope = LoadSemantics::for_home(&Home::domain("rust").expect("non-empty domain"));
        match scope {
            LoadSemantics::WhenReading(globs) => {
                assert_eq!(globs.as_slice(), &["**/*.rs".to_owned()]);
            }
            other => panic!("expected WhenReading, got {other:?}"),
        }
    }

    #[test]
    fn unknown_domain_is_on_request() {
        // Reachable by description — never blanket-applied, and never an empty
        // glob list, which some targets read as "everything".
        assert_eq!(
            LoadSemantics::for_home(&Home::domain("cobol").expect("non-empty domain")),
            LoadSemantics::OnRequest
        );
    }

    #[test]
    fn an_empty_glob_list_cannot_be_built() {
        // The illegal state, refused at the constructor rather than rendered.
        assert!(Globs::new(Vec::new()).is_none());
        assert!(Globs::new(vec!["**/*.rs".to_owned()]).is_some());
    }

    #[test]
    fn home_rank_orders_general_before_specific() {
        assert!(home_rank(&Home::global()) < home_rank(&Home::domain("rust").expect("domain")));
        assert!(
            home_rank(&Home::domain("rust").expect("domain"))
                < home_rank(&Home::project("C:/repo").expect("project"))
        );
    }

    #[test]
    fn domain_matching_is_case_insensitive_and_aliased() {
        assert_eq!(
            match LoadSemantics::for_home(&Home::domain("TypeScript").expect("non-empty domain")) {
                LoadSemantics::WhenReading(g) => g.as_slice().to_vec(),
                other => panic!("expected WhenReading, got {other:?}"),
            },
            &["**/*.ts".to_owned(), "**/*.tsx".to_owned()]
        );
        assert_eq!(
            match LoadSemantics::for_home(&Home::domain("ts").expect("non-empty domain")) {
                LoadSemantics::WhenReading(g) => g.as_slice().to_vec(),
                other => panic!("expected WhenReading, got {other:?}"),
            },
            &["**/*.ts".to_owned(), "**/*.tsx".to_owned()]
        );
    }
}
