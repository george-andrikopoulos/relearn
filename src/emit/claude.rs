//! `emit::claude` — one Claude skill **per home layer** (decision 2026-08-13),
//! not one per rule. Each home produces `skills/<home-slug>/SKILL.md` with YAML
//! front-matter (`name`, `description`); the description is a *trigger* — a lead
//! sentence saying when the skill applies, then as many of the home's rule
//! **titles** as fit [`DESCRIPTION_MAX_CHARS`], with the remainder counted. It
//! is bounded by construction ([`SkillDescription`]) because `description` is
//! the only string Claude reads when deciding whether to load the skill, and an
//! over-long one is rejected at install time. Per-home keeps the
//! always-resident skill metadata bounded (P6), rather than growing it
//! `O(rules)`.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.

use std::collections::BTreeMap;

use super::{
    HomeSlug, OutputFile, RelativePath, emittable, graduation_note, recurrence_note,
    yaml_double_quote,
};
use crate::library::{Library, Validated};
use crate::rule::{Home, Rule};

/// Emit one skill per home layer. Homes are ordered by slug and rules within a
/// home by tag, so the output is a deterministic function of the library — the
/// property `emit(lib) == emit(lib)` holds, which the overwrite guard relies on.
///
/// Accepts only `&Library<Validated>`: an unvalidated library does not have
/// this type, so emitting unchecked rules is a compile error, not a runtime
/// guard.
#[must_use]
pub fn emit(library: &Library<Validated>) -> Vec<OutputFile> {
    // Group by home slug, keeping a reference to the home for its human label.
    // `&Rule`/`&Home` borrow the library for this call; nothing is cloned.
    // Only emittable rules (active + graduated); atticked guidance is suppressed,
    // so a home whose rules are all atticked produces no skill.
    let mut by_home: BTreeMap<HomeSlug, (&Home, Vec<&Rule>)> = BTreeMap::new();
    for rule in emittable(library) {
        let slug = HomeSlug::of(rule.home());
        by_home
            .entry(slug)
            .or_insert_with(|| (rule.home(), Vec::new()))
            .1
            .push(rule);
    }

    let mut files = Vec::with_capacity(by_home.len());
    for (slug, (home, mut rules)) in by_home {
        rules.sort_by(|a, b| a.tag().as_str().cmp(b.tag().as_str()));
        let contents = render_skill(&slug, home, &rules);
        let path = RelativePath::from_segments(&["skills", slug.as_str(), "SKILL.md"]);
        // The OutputFile outlives the `&Library` borrow, so it owns its
        // provenance tags. In tag order (rules are sorted above), so the
        // header's `rules=` list is deterministic.
        let sources = rules
            .iter()
            .map(|r| r.tag().clone()) // allow:clone: the returned OutputFile owns its provenance, outliving the library borrow
            .collect();
        files.push(OutputFile::new(path, contents, sources));
    }
    files
}

/// The most characters Claude's skill front-matter accepts in `description`.
///
/// The cap is the target's, so it lives with the target's emitter. It is a hard
/// limit, not a style preference: an over-long description is rejected when the
/// skill is installed, and `description` is the only string the model reads when
/// deciding whether to load the skill, so it is the trigger and nothing else.
pub const DESCRIPTION_MAX_CHARS: usize = 1024;

/// A Claude skill `description`, **within [`DESCRIPTION_MAX_CHARS`] by
/// construction**.
///
/// The private field and the fallible-free constructor are the point: the only
/// way to obtain one is [`SkillDescription::build`], which truncates, so an
/// over-long description is not a bug to catch but a value that cannot be made.
///
/// **Why this type exists** (2026-09-06). The description was assembled inline
/// as `"Rules for {label}. Covers: {title} ({error_class}); ..."` over every
/// rule in the home. At 46 rules that produced **6924** characters for `global`
/// and **4322** for `domain-rust` — nearly seven times the cap. Both skills were
/// therefore uninstallable, and had they installed, the field that decides
/// whether the skill loads was an unreadable wall of error-class prose. Nothing
/// caught it because nothing measured it: the emitter's own tests build
/// two-rule libraries, where the inline format is comfortably short and stays
/// short forever. `[R:prefer-by-construction]`
#[derive(Debug, Clone, PartialEq, Eq)]
struct SkillDescription(String);

impl SkillDescription {
    /// Build a description from a lead sentence and the home's rule titles,
    /// keeping as many titles as fit and naming the count of those dropped.
    ///
    /// **Titles, not error classes.** The error class states the *failure* in a
    /// subordinate clause written for a reviewer ("Encoding a distinct state as
    /// a magic value of an existing type ... that downstream logic must
    /// remember to special-case"); the title states the *practice* in a few
    /// words ("No sentinel values: absent states are enum variants"). A matcher
    /// reads the description looking for the subject at hand, so the title is
    /// the useful token and the error class is ballast — it is also what made
    /// the field seven times too long.
    ///
    /// Truncation is deterministic (titles arrive in the caller's tag order), so
    /// re-emitting an unchanged library reproduces the same bytes.
    fn build(lead: &str, titles: &[&str]) -> Self {
        const JOIN: &str = "; ";
        let mut out = format!("{lead} Covers: ");
        let mut kept = 0usize;
        for title in titles {
            // Reserve room for the "+N more" tail before committing a title, so
            // the tail can always be appended afterwards without overflowing.
            let separator = if kept == 0 { "" } else { JOIN };
            let tail = Self::more_tail(titles.len() - kept - 1);
            let projected =
                out.chars().count() + separator.chars().count() + title.chars().count() + tail;
            if projected > DESCRIPTION_MAX_CHARS {
                break;
            }
            out.push_str(separator);
            out.push_str(title);
            kept += 1;
        }
        let dropped = titles.len() - kept;
        if dropped > 0 {
            out.push_str(&format!("{JOIN}+{dropped} more"));
        }
        debug_assert!(out.chars().count() <= DESCRIPTION_MAX_CHARS);
        SkillDescription(out)
    }

    /// The character cost of the `"; +N more"` tail for `n` dropped titles.
    fn more_tail(n: usize) -> usize {
        if n == 0 {
            0
        } else {
            format!("; +{n} more").chars().count()
        }
    }

    /// The description text.
    fn as_str(&self) -> &str {
        &self.0
    }
}

/// The lead sentence for a home — *when* the skill applies, which is what a
/// matcher needs first. Derived from [`Home`], so the emitter still reads
/// nothing the rule does not carry (the same derivation `LoadSemantics` makes
/// for Cursor's globs).
fn description_lead(home: &Home) -> String {
    match home {
        Home::Global => {
            "Engineering discipline that applies to every project and language.".to_owned()
        }
        Home::Org { name } => format!(
            "Engineering discipline that applies across {}.",
            name.as_str()
        ),
        Home::Domain { name } => {
            format!("Engineering discipline for working in {}.", name.as_str())
        }
        Home::Project { path } => {
            format!(
                "Engineering discipline for the {} repository.",
                path.as_str()
            )
        }
    }
}

/// Render one skill file: YAML front-matter then a markdown section per rule.
fn render_skill(slug: &HomeSlug, home: &Home, rules: &[&Rule]) -> String {
    let label = home_label(home);
    let titles: Vec<&str> = rules.iter().map(|r| r.title().as_str()).collect();
    let description = SkillDescription::build(&description_lead(home), &titles)
        .as_str()
        .to_owned();

    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("name: {}\n", slug.as_str()));
    out.push_str(&format!(
        "description: {}\n",
        yaml_double_quote(&description)
    ));
    out.push_str("---\n\n");
    out.push_str(&format!("# {label} rules\n"));
    for r in rules {
        out.push_str(&format!(
            "\n## {} [{}]\n\n",
            r.title().as_str(),
            r.tag().as_str()
        ));
        if let Some(note) = graduation_note(r) {
            out.push_str(&note);
        }
        if let Some(note) = recurrence_note(r) {
            out.push_str(&note);
        }
        out.push_str(r.body().as_str());
        out.push('\n');
    }
    out
}

/// The human-readable label for a home, used in headings and the description.
fn home_label(home: &Home) -> String {
    match home {
        Home::Global => "global".to_owned(),
        Home::Org { name } => format!("org: {}", name.as_str()),
        Home::Domain { name } => format!("domain: {}", name.as_str()),
        Home::Project { path } => format!("project: {}", path.as_str()),
    }
}

#[cfg(test)]
mod tests {

    // The bound is the whole point of SkillDescription, so it is pinned at the
    // three places it can break: comfortably under, exactly at, and far over.
    #[test]
    fn a_short_description_keeps_every_title_and_names_no_remainder() {
        let d = SkillDescription::build("Lead.", &["First title", "Second title"]);
        assert_eq!(d.as_str(), "Lead. Covers: First title; Second title");
        assert!(!d.as_str().contains("more"));
    }

    #[test]
    fn an_over_long_list_is_truncated_within_the_cap_and_counts_the_remainder() {
        // 200 titles of ~30 chars each is ~6000 characters of raw material,
        // the scale the real corpus reached.
        let owned: Vec<String> = (0..200)
            .map(|i| format!("A rule title number {i:03}"))
            .collect();
        let titles: Vec<&str> = owned.iter().map(String::as_str).collect();
        let d = SkillDescription::build("Lead.", &titles);
        let chars = d.as_str().chars().count();
        assert!(
            chars <= DESCRIPTION_MAX_CHARS,
            "{chars} chars exceeds the cap"
        );
        assert!(
            d.as_str().contains(" more"),
            "the dropped count must be named: {}",
            d.as_str()
        );
        assert!(
            d.as_str()
                .starts_with("Lead. Covers: A rule title number 000")
        );
    }

    // A title so long that not even the first one fits must still produce a
    // description within the cap rather than overflowing on the first push.
    #[test]
    fn a_single_oversized_title_does_not_overflow_the_cap() {
        let huge = "x".repeat(DESCRIPTION_MAX_CHARS * 2);
        let d = SkillDescription::build("Lead.", &[huge.as_str(), "second"]);
        assert!(d.as_str().chars().count() <= DESCRIPTION_MAX_CHARS);
        assert!(d.as_str().contains("+2 more"), "{}", d.as_str());
    }

    #[test]
    fn the_lead_sentence_says_when_the_skill_applies() {
        assert!(description_lead(&Home::global()).contains("every project"));
        let rust = Home::domain("rust").expect("non-empty domain");
        assert!(description_lead(&rust).contains("rust"));
    }
    use super::*;
    use crate::library::Library;
    use crate::rule::{Body, Date, ErrorClass, Home, Incident, Origin, RuleTag, Status, Title};

    fn rule(tag: &str, home: Home, title: &str, error_class: &str, body: &str) -> Rule {
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse(title).expect("non-empty title"),
            ErrorClass::parse(error_class).expect("non-empty error class"),
            home,
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            Status::active(),
            Incident::parse("an incident").expect("non-empty incident"),
            Body::parse(body).expect("non-empty body"),
            Vec::new(),
            Vec::new(),
        )
    }

    fn validated(rules: Vec<Rule>) -> Library<Validated> {
        Library::from_rules(rules)
            .validate()
            .expect("distinct tags validate")
    }

    fn mixed_library() -> Library<Validated> {
        validated(vec![
            rule("R:g", Home::global(), "Global rule", "gc", "Global body."),
            rule(
                "R:r2",
                Home::domain("rust").expect("non-empty domain"),
                "Rust two",
                "rc2",
                "Rust two body.",
            ),
            rule(
                "R:r1",
                Home::domain("rust").expect("non-empty domain"),
                "Rust one",
                "rc1",
                "Rust one body.",
            ),
            rule(
                "R:p",
                Home::project("C:/repo").expect("non-empty project"),
                "Project rule",
                "pc",
                "Project body.",
            ),
        ])
    }

    #[test]
    fn one_skill_per_home_layer() {
        let files = emit(&mixed_library());
        assert_eq!(files.len(), 3, "global, domain-rust, project-c-repo");
        let paths: Vec<&str> = files.iter().map(|f| f.path().as_str()).collect();
        assert!(paths.contains(&"skills/global/SKILL.md"));
        assert!(paths.contains(&"skills/domain-rust/SKILL.md"));
        assert!(paths.contains(&"skills/project-c-repo/SKILL.md"));
    }

    #[test]
    fn home_skill_aggregates_its_rules_sorted_by_tag() {
        let files = emit(&mixed_library());
        let rust = files
            .iter()
            .find(|f| f.path().as_str() == "skills/domain-rust/SKILL.md")
            .expect("the rust skill is present");
        let c = rust.contents();
        let i1 = c.find("[R:r1]").expect("R:r1 heading present");
        let i2 = c.find("[R:r2]").expect("R:r2 heading present");
        assert!(i1 < i2, "rules are ordered by tag within a home");
        assert!(c.contains("Rust one body."));
        assert!(c.contains("Rust two body."));
    }

    #[test]
    fn front_matter_names_the_home_slug() {
        let files = emit(&mixed_library());
        let rust = files
            .iter()
            .find(|f| f.path().as_str() == "skills/domain-rust/SKILL.md")
            .expect("the rust skill is present");
        assert!(
            rust.contents()
                .starts_with("---\nname: domain-rust\ndescription: ")
        );
    }

    #[test]
    fn description_covers_each_rule_in_the_home() {
        let files = emit(&mixed_library());
        let rust = files
            .iter()
            .find(|f| f.path().as_str() == "skills/domain-rust/SKILL.md")
            .expect("the rust skill is present");
        let description = description_of(rust.contents());
        assert!(description.contains("Rust one"), "{description}");
        assert!(description.contains("Rust two"), "{description}");
        // Error classes are deliberately absent: they are reviewer-facing prose
        // about the *failure*, they say nothing a matcher can trigger on, and
        // including them is what pushed the real corpus past the cap.
        assert!(!description.contains("rc1"), "{description}");
        assert!(!description.contains("rc2"), "{description}");
    }

    /// The description's first sentence says *when* the skill applies, before
    /// any rule title — a matcher reads the opening words first.
    #[test]
    fn description_leads_with_when_the_skill_applies() {
        let files = emit(&mixed_library());
        let rust = files
            .iter()
            .find(|f| f.path().as_str() == "skills/domain-rust/SKILL.md")
            .expect("the rust skill is present");
        assert!(
            description_of(rust.contents()).starts_with("Engineering discipline for working in "),
            "{}",
            description_of(rust.contents())
        );
    }

    /// The `description:` value from a rendered skill, unquoted.
    fn description_of(contents: &str) -> String {
        contents
            .lines()
            .find_map(|l| l.strip_prefix("description: "))
            .expect("a rendered skill has a description")
            .trim_matches('"')
            .to_owned()
    }

    #[test]
    fn description_is_quoted_so_a_colon_in_a_title_stays_valid_yaml() {
        let lib = validated(vec![rule(
            "R:x",
            Home::global(),
            "Title: with colon",
            "err",
            "b",
        )]);
        let files = emit(&lib);
        let c = files[0].contents();
        assert!(
            c.contains(
                "description: \"Engineering discipline that applies to every project and language. Covers: Title: with colon\""
            ),
            "the description value is double-quoted, so an inner colon is safe: {c}"
        );
    }

    #[test]
    fn emission_is_deterministic() {
        let lib = mixed_library();
        assert_eq!(emit(&lib), emit(&lib));
    }

    fn rule_with_status(tag: &str, home: Home, status: Status, body: &str) -> Rule {
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse("Title").expect("non-empty title"),
            ErrorClass::parse("ec").expect("non-empty error class"),
            home,
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            status,
            Incident::parse("an incident").expect("non-empty incident"),
            Body::parse(body).expect("non-empty body"),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn a_home_with_only_atticked_rules_produces_no_skill() {
        let date = Date::parse("2026-09-01").expect("valid date");
        let lib = validated(vec![
            rule_with_status("R:g", Home::global(), Status::active(), "Global body."),
            rule_with_status(
                "R:r",
                Home::domain("rust").expect("non-empty domain"),
                Status::attic("retired", date).expect("non-empty reason"),
                "Rust body.",
            ),
        ]);
        let files = emit(&lib);
        let paths: Vec<&str> = files.iter().map(|f| f.path().as_str()).collect();
        assert!(paths.contains(&"skills/global/SKILL.md"));
        assert!(
            !paths.contains(&"skills/domain-rust/SKILL.md"),
            "the rust home's only rule is atticked, so no skill is written for it"
        );
    }

    #[test]
    fn a_graduated_rule_is_annotated_in_its_skill() {
        let lib = validated(vec![rule_with_status(
            "R:grad",
            Home::domain("rust").expect("non-empty domain"),
            Status::graduated(
                "hook:no-unwrap-in-src",
                Date::parse("2026-07-21").expect("valid date"),
            )
            .expect("non-empty destination"),
            "Never unwrap in production.",
        )]);
        let files = emit(&lib);
        let rust = files
            .iter()
            .find(|f| f.path().as_str() == "skills/domain-rust/SKILL.md")
            .expect("the rust skill is present");
        let c = rust.contents();
        assert!(c.contains("[R:grad]"));
        assert!(c.contains("> Also enforced by hook:no-unwrap-in-src."));
    }
}
