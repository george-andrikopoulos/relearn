//! Property tests over generated rule sets (rust-typedd tier 2). These exercise
//! the whole public pipeline — parse/serialize round-trip and emitter
//! determinism — with `proptest`, which types alone cannot express: "for all
//! rules, serialize-then-parse is the identity" and "for all libraries, every
//! emitter is a deterministic function".

use std::collections::BTreeSet;

use proptest::prelude::*;

use relearn::emit;
use relearn::library::{Library, Validated};
use relearn::rule::{
    Body, Date, ErrorClass, Home, Incident, Recurrence, Rule, RuleTag, Status, Title,
    parse_document, to_document,
};

/// Non-empty, edge-trimmed text (parsing trims, so generated values must have no
/// leading/trailing whitespace to round-trip). Includes `"` to exercise the
/// serializer's TOML escaping under proptest.
fn arb_text() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9 .,()\"-]{1,40}"
        .prop_map(|s| s.trim().to_owned())
        .prop_filter("non-empty after trim", |s| !s.is_empty())
}

/// A well-shaped tag body (`R:` is added by the caller).
fn arb_tag_body() -> impl Strategy<Value = String> {
    proptest::string::string_regex("[a-z0-9][a-z0-9-]{0,20}").expect("valid tag-body regex")
}

/// A valid calendar date, generated safely (day <= 28 so every month is valid).
fn arb_date() -> impl Strategy<Value = Date> {
    (2000i32..=2100, 1u32..=12, 1u32..=28).prop_map(|(y, m, d)| {
        Date::parse(&format!("{y:04}-{m:02}-{d:02}")).expect("generated date is valid")
    })
}

fn arb_home() -> impl Strategy<Value = Home> {
    prop_oneof![
        Just(Home::global()),
        arb_text().prop_map(|n| Home::domain(n).expect("non-empty domain")),
        arb_text().prop_map(|p| Home::project(p).expect("non-empty project")),
    ]
}

fn arb_status() -> impl Strategy<Value = Status> {
    prop_oneof![
        Just(Status::active()),
        arb_text().prop_map(|t| Status::graduated(t).expect("non-empty destination")),
        (arb_text(), arb_date()).prop_map(|(r, d)| Status::attic(r, d).expect("non-empty reason")),
    ]
}

/// Zero to three recurrences, dates and text both generated. Zero is included
/// deliberately: the unrecurred rule is the overwhelmingly common shape and the
/// one whose rendering must not change.
fn arb_recurrences() -> impl Strategy<Value = Vec<Recurrence>> {
    proptest::collection::vec(
        (arb_date(), arb_text()).prop_map(|(date, incident)| {
            Recurrence::new(date, Incident::parse(incident).expect("non-empty incident"))
        }),
        0..3,
    )
}

/// A fully-arbitrary rule with the given tag body.
fn arb_rule(tag_body: String) -> impl Strategy<Value = Rule> {
    (
        arb_text(),
        arb_text(),
        arb_home(),
        arb_date(),
        arb_status(),
        arb_text(),
        arb_text(),
        arb_recurrences(),
    )
        .prop_map(
            move |(title, error_class, home, created, status, incident, body, recurrences)| {
                Rule::new(
                    RuleTag::parse(format!("R:{tag_body}")).expect("valid tag"),
                    Title::parse(title).expect("non-empty title"),
                    ErrorClass::parse(error_class).expect("non-empty error class"),
                    home,
                    created,
                    status,
                    Incident::parse(incident).expect("non-empty incident"),
                    Body::parse(body).expect("non-empty body"),
                    recurrences,
                )
            },
        )
}

/// A validated library of up to six rules with distinct tags, varied homes, and
/// **varied statuses** (active / graduated / atticked). Distinct from
/// [`arb_library`] (all-active) because the emit-status policy — suppress
/// atticked, emit active + graduated — is only exercised when a library carries
/// a mix of lifecycle states.
fn arb_library_mixed() -> impl Strategy<Value = Library<Validated>> {
    proptest::collection::vec((arb_tag_body(), arb_home(), arb_text(), arb_status()), 0..6)
        .prop_map(|items| {
            let mut seen = BTreeSet::new();
            let mut rules = Vec::new();
            for (tag_body, home, text, status) in items {
                if !seen.insert(tag_body.clone()) {
                    continue; // keep tags distinct so validation succeeds
                }
                rules.push(Rule::new(
                    RuleTag::parse(format!("R:{tag_body}")).expect("valid tag"),
                    Title::parse(text).expect("non-empty title"),
                    ErrorClass::parse("error class").expect("non-empty error class"),
                    home,
                    Date::parse("2026-08-13").expect("valid date"),
                    status,
                    Incident::parse("incident").expect("non-empty incident"),
                    Body::parse("body").expect("non-empty body"),
                    Vec::new(),
                ));
            }
            Library::from_rules(rules)
                .validate()
                .expect("distinct tags validate")
        })
}

/// A validated library of up to five rules with distinct tags, varied homes and
/// statuses, and **varied recurrence histories** — some rules bitten, some not.
/// Distinct from the libraries above because the recurrence annotation is only
/// exercised when a library carries both kinds.
fn arb_library_recurring() -> impl Strategy<Value = Library<Validated>> {
    proptest::collection::vec(
        (
            arb_tag_body(),
            arb_home(),
            arb_text(),
            arb_status(),
            arb_recurrences(),
        ),
        0..5,
    )
    .prop_map(|items| {
        let mut seen = BTreeSet::new();
        let mut rules = Vec::new();
        for (tag_body, home, text, status, recurrences) in items {
            if !seen.insert(tag_body.clone()) {
                continue; // keep tags distinct so validation succeeds
            }
            rules.push(Rule::new(
                RuleTag::parse(format!("R:{tag_body}")).expect("valid tag"),
                Title::parse(text).expect("non-empty title"),
                ErrorClass::parse("error class").expect("non-empty error class"),
                home,
                Date::parse("2026-08-13").expect("valid date"),
                status,
                Incident::parse("incident").expect("non-empty incident"),
                Body::parse("body").expect("non-empty body"),
                recurrences,
            ));
        }
        Library::from_rules(rules)
            .validate()
            .expect("distinct tags validate")
    })
}

/// The note a rule's recurrence history should produce, rebuilt independently of
/// `emit` so the property is checked against the *specification* rather than
/// against the implementation restating itself.
fn expected_recurrence_note(rule: &Rule) -> Option<String> {
    let latest = rule.recurrences().iter().map(Recurrence::date).max()?;
    Some(format!(
        "> Has recurred {} time(s) since it was written; most recently {latest}.",
        rule.recurrences().len()
    ))
}

/// The tags of every atticked rule in a library.
fn atticked_tags(lib: &Library<Validated>) -> BTreeSet<String> {
    lib.rules()
        .iter()
        .filter(|r| matches!(r.status(), Status::Attic { .. }))
        .map(|r| r.tag().as_str().to_owned())
        .collect()
}

/// A validated library of up to five rules with distinct tags and varied homes.
fn arb_library() -> impl Strategy<Value = Library<Validated>> {
    proptest::collection::vec((arb_tag_body(), arb_home(), arb_text()), 0..5).prop_map(|items| {
        let mut seen = std::collections::BTreeSet::new();
        let mut rules = Vec::new();
        for (tag_body, home, text) in items {
            if !seen.insert(tag_body.clone()) {
                continue; // keep tags distinct so validation succeeds
            }
            rules.push(Rule::new(
                RuleTag::parse(format!("R:{tag_body}")).expect("valid tag"),
                Title::parse(text).expect("non-empty title"),
                ErrorClass::parse("error class").expect("non-empty error class"),
                home,
                Date::parse("2026-08-13").expect("valid date"),
                Status::active(),
                Incident::parse("incident").expect("non-empty incident"),
                Body::parse("body").expect("non-empty body"),
                Vec::new(),
            ));
        }
        Library::from_rules(rules)
            .validate()
            .expect("distinct tags validate")
    })
}

proptest! {
    /// Serialize-then-parse is the identity: the neutral format is lossless over
    /// tag, title, error class, home, created date, status (with payload), and
    /// body — including quotes that must be TOML-escaped.
    #[test]
    fn neutral_round_trip_preserves_the_rule(rule in arb_tag_body().prop_flat_map(arb_rule)) {
        let doc = to_document(&rule);
        let reparsed = parse_document(&doc).expect("a serialized rule must re-parse");
        prop_assert_eq!(reparsed, rule);
    }

    /// Every emitter is a deterministic function of the library: re-emitting the
    /// same library yields byte-identical output. This is the property the
    /// overwrite guard and "rebuild is a no-op" both rely on.
    #[test]
    fn emission_is_idempotent(lib in arb_library()) {
        prop_assert_eq!(emit::claude::emit(&lib), emit::claude::emit(&lib));
        prop_assert_eq!(emit::cursor::emit(&lib), emit::cursor::emit(&lib));
        prop_assert_eq!(emit::copilot::emit(&lib), emit::copilot::emit(&lib));
        prop_assert_eq!(emit::agents::emit(&lib), emit::agents::emit(&lib));
        prop_assert_eq!(emit::claude_rules::emit(&lib), emit::claude_rules::emit(&lib));
    }

    /// **Withdrawn guidance never leaks.** For any library, no atticked rule's
    /// tag appears in the provenance of any file any emitter produces — the
    /// whole-space form of "an active instruction file must not instruct a
    /// retired rule". `sources` (tags), not body substrings, is the honest check.
    #[test]
    fn atticked_rules_never_leak_into_any_emitter(lib in arb_library_mixed()) {
        let atticked = atticked_tags(&lib);
        let outputs = [
            emit::claude::emit(&lib),
            emit::cursor::emit(&lib),
            emit::copilot::emit(&lib),
            emit::agents::emit(&lib),
            emit::claude_rules::emit(&lib),
        ];
        for files in &outputs {
            for file in files {
                for tag in file.sources() {
                    prop_assert!(
                        !atticked.contains(tag.as_str()),
                        "atticked rule {} leaked into {}",
                        tag.as_str(),
                        file.path().as_str()
                    );
                }
            }
        }
    }

    /// **The hand-authored charter is never a target.** For any library, no
    /// emitter produces a file at the repository-root `CLAUDE.md` — the
    /// whole-space form of the collision fixed on 2026-08-22, when the project
    /// layer moved from that constant path to `.claude/rules/<home-slug>.md`.
    ///
    /// This is the artefact that keeps the fix fixed. A regression pointing any
    /// emitter back at the root charter fails here rather than being absorbed
    /// by the write-side marker guard, which is what hid the defect for months
    /// (`[R:prefer-by-construction]`).
    #[test]
    fn no_emitter_ever_targets_the_repo_root_claude_md(lib in arb_library_mixed()) {
        let outputs = [
            emit::claude::emit(&lib),
            emit::cursor::emit(&lib),
            emit::copilot::emit(&lib),
            emit::agents::emit(&lib),
            emit::claude_rules::emit(&lib),
        ];
        for files in &outputs {
            for file in files {
                prop_assert_ne!(
                    file.path().as_str(),
                    "CLAUDE.md",
                    "an emitter targeted the hand-authored project charter"
                );
            }
        }
    }

    /// **A recurrence reaches every emitted format, and only where it is real.**
    /// For any library, every file an emitter produces carries the recurrence
    /// note of each recurred rule it names as a source, and carries no note for a
    /// rule that has not recurred.
    ///
    /// This is the artifact that keeps the annotation wired. `recurrence_note`
    /// is one function, but there are five independent splice sites, and a unit
    /// test of the function proves nothing about whether an emitter calls it —
    /// the same shape of gap that let the project layer point at the root
    /// charter for months. Checked over the union of all five emitters' output.
    #[test]
    fn a_recurrence_is_annotated_in_every_emitted_format(lib in arb_library_recurring()) {
        let outputs = [
            emit::claude::emit(&lib),
            emit::cursor::emit(&lib),
            emit::copilot::emit(&lib),
            emit::agents::emit(&lib),
            emit::claude_rules::emit(&lib),
        ];
        for files in &outputs {
            for file in files {
                let mut expected_notes = 0usize;
                for tag in file.sources() {
                    let rule = lib
                        .rules()
                        .iter()
                        .find(|r| r.tag().as_str() == tag.as_str())
                        .expect("a file's source is a rule of the library");
                    if let Some(note) = expected_recurrence_note(rule) {
                        expected_notes += 1;
                        prop_assert!(
                            file.contents().contains(&note),
                            "{} has recurred but {} carries no note for it",
                            tag.as_str(),
                            file.path().as_str()
                        );
                    }
                }
                // Counted, not merely "absent": a concatenated file holds many
                // rules, so "no note anywhere" is the wrong question. One note
                // per recurred source and no more is what says a rule that has
                // never recurred was not annotated as though it had.
                prop_assert_eq!(
                    file.contents().matches("> Has recurred ").count(),
                    expected_notes,
                    "{} carries the wrong number of recurrence notes",
                    file.path().as_str()
                );
            }
        }
    }

    /// **The project layer is project-scoped.** Every file the `claude_rules`
    /// emitter produces lives under the directory it owns, is named for a home
    /// slug, and carries *only* the rules of that slug's home — so one
    /// repository's project layer can never absorb another project's rules
    /// (P2, one home per rule, enforced at the emission layer).
    ///
    /// Keyed on the slug rather than the `Home` because two distinct project
    /// paths may slugify alike; that collision is `lint`'s to report, and the
    /// slug is this emitter's identity either way.
    #[test]
    fn each_project_layer_file_carries_only_its_own_homes_rules(lib in arb_library_mixed()) {
        for file in emit::claude_rules::emit(&lib) {
            let path = file.path().as_str().to_owned();
            let slug = path
                .strip_prefix(".claude/rules/")
                .and_then(|s| s.strip_suffix(".md"))
                .ok_or_else(|| TestCaseError::fail(format!("unexpected path {path}")))?
                .to_owned();

            for tag in file.sources() {
                let rule = lib
                    .rules()
                    .iter()
                    .find(|r| r.tag() == tag)
                    .ok_or_else(|| TestCaseError::fail("provenance names a rule not in the library"))?;
                // Project and known-domain homes both reach this layer (2026-08-22);
                // `Global` does not — it already has an always-resident home, and a
                // second copy here would put one rule in two Claude files.
                prop_assert!(
                    matches!(rule.home(), Home::Project { .. } | Home::Domain { .. }),
                    "a global rule reached the rules layer: {}",
                    tag.as_str()
                );
                let rule_slug = emit::HomeSlug::of(rule.home());
                prop_assert_eq!(
                    rule_slug.as_str(),
                    slug.as_str(),
                    "rule {} does not belong to the home this file is named for",
                    tag.as_str()
                );
            }
        }
    }

    /// The suppression is exactly Attic — no over-suppression. The Copilot file
    /// concatenates every home, so its provenance must be precisely the set of
    /// non-atticked (active + graduated) rules: nothing withdrawn present,
    /// nothing in-force missing.
    #[test]
    fn active_and_graduated_rules_all_reach_copilot(lib in arb_library_mixed()) {
        let expected: BTreeSet<String> = lib
            .rules()
            .iter()
            .filter(|r| !matches!(r.status(), Status::Attic { .. }))
            .map(|r| r.tag().as_str().to_owned())
            .collect();
        let emitted: BTreeSet<String> = emit::copilot::emit(&lib)
            .iter()
            .flat_map(|f| f.sources())
            .map(|t| t.as_str().to_owned())
            .collect();
        prop_assert_eq!(emitted, expected);
    }

    /// **A rules file never carries an empty `paths:` list.** This is the whole-
    /// space form of the state `LoadSemantics` exists to make unrepresentable.
    ///
    /// The two readings of an empty glob list diverge silently across targets:
    /// Cursor reads a blank `globs` line as "no auto-attach", while a Claude rule
    /// file with `paths: []` loads at `session_start` — measured 2026-08-22, i.e.
    /// resident in every session, the exact opposite. A domain whose language has
    /// no known globs must therefore reach the assistant some other way, never as
    /// an empty glob list rendered into a file that then applies to everything.
    #[test]
    fn no_rules_file_ever_carries_an_empty_paths_list(lib in arb_library_mixed()) {
        for file in emit::claude_rules::emit(&lib) {
            prop_assert!(
                !file.contents().contains("paths: []"),
                "{} carries an empty paths list, which loads always",
                file.path().as_str()
            );
            prop_assert!(
                !file.contents().contains("paths:\n---"),
                "{} carries a paths key with no globs under it",
                file.path().as_str()
            );
        }
    }

    /// **An unknown-language domain never reaches the rules layer.** Its
    /// `LoadSemantics` is `OnRequest`, which the `paths:` front-matter has no
    /// spelling for — the target is two-state (always / glob-scoped) and cannot
    /// say "reachable by description only". Emitting it anyway would promote a
    /// rule meant to be asked for into one resident in every session.
    ///
    /// `arb_home` generates domain names from arbitrary text, so essentially
    /// every generated domain is unknown: this exercises the negative case hard.
    #[test]
    fn unknown_domains_never_reach_the_rules_layer(lib in arb_library_mixed()) {
        for file in emit::claude_rules::emit(&lib) {
            let is_unknown_domain_file = lib.rules().iter().any(|r| {
                matches!(r.home(), Home::Domain { .. })
                    && matches!(emit::LoadSemantics::for_home(r.home()), emit::LoadSemantics::OnRequest)
                    && file.sources().iter().any(|t| t == r.tag())
            });
            prop_assert!(
                !is_unknown_domain_file,
                "{} carries an on-request domain rule",
                file.path().as_str()
            );
        }
    }
}
