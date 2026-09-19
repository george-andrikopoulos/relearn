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
    HomeSlug, OutputFile, RelativePath, audience_note, emittable, enforcement_note, home_label,
    recurrence_note, source_note, yaml_double_quote,
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
    /// **Subjects, not titles, and certainly not error classes.** Each entry is
    /// the rule's tag body with its hyphens turned into spaces — `no coordinated
    /// omission`, `a view is not a copy`. The three candidates differ by an
    /// order of magnitude in what they cost per rule and barely at all in what
    /// a matcher can do with them:
    ///
    /// * the **error class** states the failure in a clause written for a
    ///   reviewer, and is what made this field seven times too long in the
    ///   first place;
    /// * the **title** states the practice as a sentence, at sixty-odd
    ///   characters — over thirty rules that is 1824 characters against a 1024
    ///   budget, so more than half of `global` was replaced by `+N more`;
    /// * the **tag body** states the same subject in half the space (749 for
    ///   those same thirty), because it was authored as a keyword phrase and
    ///   carries no articles, no verbs of being, and no qualifying clause.
    ///
    /// The hyphens become spaces so the matcher sees separate word tokens rather
    /// than one identifier, and so a human reading the skill list sees a phrase.
    /// This is not a heuristic compression of the title: the tag is a field the
    /// author wrote, it is unique, and it is the rule's identity, so nothing is
    /// being inferred or thrown away.
    ///
    /// Truncation remains possible and is no longer expected. It is ordered by
    /// [`description_order`] rather than by the alphabet, and
    /// `tests/description_reaches_every_rule.rs` fails the build if any home
    /// ever reaches it again.
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
/// The order titles are offered to [`SkillDescription::build`], which keeps as
/// many as fit and drops the rest.
///
/// **This exists because the truncation had no stated intent.** `build` walks
/// its input and stops at the cap, so whatever order it is handed decides which
/// rules stay discoverable — and it was handed the caller's tag order, which is
/// the alphabet. Measured on a sixteen-rule home, the three titles dropped were
/// the three whose tags sort last, and nothing about `p`, `t` or `v` says "least
/// worth loading". `[R:order-by-explicit-rank]`
///
/// Three keys, most significant first:
///
/// 1. Recurrence count, **descending** — a rule that has fired again is one this
///    reader has already needed, and the count is the corpus's only *evidence*
///    of that.
/// 2. [`Status::instruction_reliance`] — a rule held by prose alone loses
///    everything if its layer does not load; one with a named control holding
///    the whole class still has that control.
/// 3. Tag, ascending — the tiebreak, so emission stays a deterministic function
///    of the library and re-emitting unchanged rules reproduces the same bytes.
///
/// **Recurrence leads, and that order was reversed on 2026-09-19.** The first
/// version put reliance first, reasoning that the rank answers "what does it
/// cost for this title to be absent", which is a question about what *else*
/// holds the rule. That is defensible in isolation and inconsistent with
/// everything around it: `lint` fails CI on an unheld recurrence, the
/// session-start hook leads with recurrence, and the framework treats it as the
/// number that says whether a rule is working. Ranking a rule that has never
/// fired above one that has inverted that in exactly one place, and the live
/// casualty was `[R:guarantee-needs-a-reader]` — `partial`, one recurrence, and
/// dropped from `global`'s description while never-fired rules were kept.
///
/// The **body** of the skill is deliberately not reordered: it stays in tag
/// order, where a reader can find a rule by name and a diff stays readable. This
/// orders which titles survive a truncation, and nothing else.
fn description_order<'a>(rules: &[&'a Rule]) -> Vec<&'a Rule> {
    let mut ordered: Vec<&Rule> = rules.to_vec();
    ordered.sort_by(|a, b| {
        b.recurrences()
            .len()
            .cmp(&a.recurrences().len())
            .then(
                a.status()
                    .instruction_reliance()
                    .cmp(&b.status().instruction_reliance()),
            )
            .then(a.tag().as_str().cmp(b.tag().as_str()))
    });
    ordered
}

fn render_skill(slug: &HomeSlug, home: &Home, rules: &[&Rule]) -> String {
    let label = home_label(home);
    let owned: Vec<String> = description_order(rules)
        .iter()
        .map(|r| r.tag().body().replace('-', " "))
        .collect();
    let subjects: Vec<&str> = owned.iter().map(String::as_str).collect();
    let description = SkillDescription::build(&description_lead(home), &subjects)
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
        if let Some(note) = audience_note(r) {
            out.push_str(&note);
        }
        if let Some(note) = enforcement_note(r) {
            out.push_str(&note);
        }
        if let Some(note) = recurrence_note(r) {
            out.push_str(&note);
        }
        if let Some(note) = source_note(r) {
            out.push_str(&note);
        }
        out.push_str(r.body().as_str());
        out.push('\n');
    }
    out
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
    use crate::rule::{
        Authority, Body, Date, ErrorClass, Home, Incident, Origin, Recurrence, RuleTag, Status,
        Title,
    };

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
            Authority::local(),
            None,
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
        // The rule's *subject* — its tag body, hyphens turned to spaces — not
        // its title. Half the characters for the same matchable words, which is
        // what lets a thirty-rule home fit inside the cap at all.
        assert!(description.contains("r1"), "{description}");
        assert!(description.contains("r2"), "{description}");
        // And the title is now absent, so the saving is real rather than
        // additive.
        assert!(!description.contains("Rust one"), "{description}");
        assert!(!description.contains("Rust two"), "{description}");
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

    /// The description value is double-quoted, so a colon inside it stays valid
    /// YAML rather than reading as a second key.
    ///
    /// **This test's channel changed and the test changed with it.** It used to
    /// put the colon in a rule *title*, which was how a title reached the
    /// description. Titles no longer do — the field lists tag bodies, and a tag
    /// is `[a-z0-9][a-z0-9-]*`, so it cannot carry a colon at all. Left as it
    /// was, the test would have gone on passing while asserting nothing: the
    /// colon would simply never have arrived.
    ///
    /// The colon can still arrive, by the one route left open — a **project
    /// path**, which `description_lead` interpolates and which on Windows begins
    /// `C:`. So the quoting is still load-bearing, and this now exercises the
    /// channel that actually exists. `[R:verify-through-production-path]`
    #[test]
    fn description_is_quoted_so_a_colon_in_a_home_label_stays_valid_yaml() {
        let home = Home::project("C:/repo").expect("non-empty project path");
        let lib = validated(vec![rule("R:x", home, "A title", "err", "b")]);
        let files = emit(&lib);
        let c = files[0].contents();
        assert!(
            c.contains(r#"description: "Engineering discipline for the C:/repo repository."#),
            "the description value is double-quoted, so an inner colon is safe: {c}"
        );
        // The colon is inside the quoted scalar, not ending the key.
        let line = c
            .lines()
            .find(|l| l.starts_with("description:"))
            .expect("a description line");
        assert!(line.ends_with('"'), "the value is closed: {line}");
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
            Authority::local(),
            None,
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

    // ── which titles survive a truncation ───────────────────────────────────

    /// A rule with the given tag, status and recurrence count, for ordering
    /// tests. Titles are made long on purpose so a handful of rules overflows
    /// the cap and the truncation is reached.
    fn ranked(tag: &str, status: Status, recurrences: usize) -> Rule {
        let padded = format!(
            "A title for {tag} long enough that a few of these overflow the description cap"
        );
        let recs: Vec<Recurrence> = (0..recurrences)
            .map(|_| {
                Recurrence::new(
                    Date::parse("2026-09-01").expect("valid date"),
                    Incident::parse("it fired again").expect("non-empty incident"),
                )
            })
            .collect();
        Rule::new(
            RuleTag::parse(tag).expect("valid tag"),
            Title::parse(padded).expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::global(),
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            status,
            Incident::parse("an incident").expect("non-empty incident"),
            Body::parse("Body.").expect("non-empty body"),
            recs,
            Vec::new(),
            Authority::local(),
            None,
        )
    }

    fn graduated() -> Status {
        Status::graduated("hook:something", Date::parse("2026-08-01").expect("date"))
            .expect("a valid destination")
    }

    /// **The defect this rank exists for.** A rule held by prose alone outranks
    /// a graduated one whatever the alphabet says — so a truncation drops the
    /// rule that still has a control firing for it, not the one that has
    /// nothing else.
    ///
    /// `R:aaa` is graduated and sorts first by tag; `R:zzz` is active and sorts
    /// last. Under the old tag ordering `R:aaa` was kept and `R:zzz` dropped,
    /// which is the exact inversion measured on the real corpus.
    #[test]
    fn a_rule_prose_alone_holds_outranks_a_graduated_one_whatever_the_tag_says() {
        let rules = [
            ranked("R:aaa", graduated(), 0),
            ranked("R:zzz", Status::active(), 0),
        ];
        let refs: Vec<&Rule> = rules.iter().collect();
        let ordered = description_order(&refs);
        assert_eq!(
            ordered.iter().map(|r| r.tag().as_str()).collect::<Vec<_>>(),
            ["R:zzz", "R:aaa"],
            "the actively-held rule must come first"
        );
    }

    /// **The case the two keys disagree on, and the reason their order was
    /// swapped on 2026-09-19.**
    ///
    /// A graduated rule that has fired again against an active rule that never
    /// has. Reliance-first keeps the active one, because the graduated rule has
    /// a control behind it; recurrence-first keeps the graduated one, because it
    /// is the one the corpus has evidence a reader needed. Everything else in
    /// this system weights recurrence that way — `lint` fails CI on an unheld
    /// one, the session-start hook leads with it — so ranking a never-fired rule
    /// above a proven one inverted the framework in exactly one place.
    ///
    /// The two tests either side of this one pass under **both** orders; this is
    /// the only one that pins the choice.
    #[test]
    fn a_recurred_rule_outranks_a_never_fired_one_even_when_it_is_held_elsewhere() {
        let rules = [
            ranked("R:aaa", Status::active(), 0),
            ranked("R:zzz", graduated(), 3),
        ];
        let refs: Vec<&Rule> = rules.iter().collect();
        let ordered = description_order(&refs);
        assert_eq!(
            ordered.iter().map(|r| r.tag().as_str()).collect::<Vec<_>>(),
            ["R:zzz", "R:aaa"],
            "evidence a reader needed the rule outranks the absence of a second control"
        );
    }

    /// Within one reliance tier, a rule that has fired again outranks one that
    /// never has — the corpus's only evidence that a reader has needed it.
    #[test]
    fn a_recurred_rule_outranks_one_that_has_never_fired() {
        let rules = [
            ranked("R:aaa", Status::active(), 0),
            ranked("R:zzz", Status::active(), 2),
        ];
        let refs: Vec<&Rule> = rules.iter().collect();
        let ordered = description_order(&refs);
        assert_eq!(
            ordered.iter().map(|r| r.tag().as_str()).collect::<Vec<_>>(),
            ["R:zzz", "R:aaa"],
            "the rule that has bitten must come first"
        );
    }

    /// The tag is the tiebreak and nothing more, so emission stays a
    /// deterministic function of the library.
    #[test]
    fn equal_rank_falls_back_to_the_tag_so_emission_stays_deterministic() {
        let rules = [
            ranked("R:zzz", Status::active(), 1),
            ranked("R:aaa", Status::active(), 1),
        ];
        let refs: Vec<&Rule> = rules.iter().collect();
        let ordered = description_order(&refs);
        assert_eq!(
            ordered.iter().map(|r| r.tag().as_str()).collect::<Vec<_>>(),
            ["R:aaa", "R:zzz"]
        );
    }

    /// The rank orders the **description** and leaves the body alone: a reader
    /// finds a rule by name in tag order, and a diff stays readable.
    #[test]
    fn the_rank_reorders_the_description_and_never_the_body() {
        let lib = validated(vec![
            ranked("R:aaa", graduated(), 0),
            ranked("R:zzz", Status::active(), 0),
        ]);
        let files = emit(&lib);
        let c = files[0].contents();

        let desc_end = c.find("\n---\n\n").expect("front-matter is closed");
        let (front, body) = c.split_at(desc_end);
        // The description names subjects (tag bodies), so the tags appear there
        // without their `R:` prefix; the body's headings keep the full tag.
        assert!(
            front.find("zzz").expect("zzz is described") < front.find("aaa").expect("aaa too"),
            "the description is ranked: {front}"
        );
        assert!(
            body.find("[R:aaa]").expect("aaa is in the body")
                < body.find("[R:zzz]").expect("zzz is in the body"),
            "the body stays in tag order: {body}"
        );
    }
}
