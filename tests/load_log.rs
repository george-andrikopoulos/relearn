//! The live load log: what the generated hook appends, and what the reader
//! makes of it — COLD, STALE, SHADOWED and LOADED, home by home.
//!
//! Every log here is synthetic text. The reader is pure, so a log is a string
//! and a window is two dates; the hook's own behaviour is pinned on the
//! production path by the pack's `test-load-log.sh`, which `check.sh` runs.

use proptest::prelude::*;
use relearn::loadlog::{
    ContentHash, Expected, Form, LoadFinding, LoadLogError, Location, Window, parse_log, read,
};
use relearn::loadset::{LayerSlug, Outcome};
use relearn::rule::Date;

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

fn slug(s: &str) -> LayerSlug {
    LayerSlug::parse(s).expect("a layer slug")
}

fn hash(s: &str) -> ContentHash {
    ContentHash::parse(s).expect("64 hex digits")
}

fn date(s: &str) -> Date {
    Date::parse(s).expect("a date")
}

fn window(since: &str, until: &str) -> Window {
    Window::new(date(since), date(until)).expect("since <= until")
}

fn skill(ts: &str, name: &str, location: &str, h: Option<&str>) -> String {
    let hash = h.map_or("null".to_owned(), |h| format!("\"{h}\""));
    let path = if location == "null" {
        "null".to_owned()
    } else {
        format!("\"/x/skills/{name}/SKILL.md\"")
    };
    let location = if location == "null" {
        "null".to_owned()
    } else {
        format!("\"{location}\"")
    };
    format!(
        r#"{{"v":1,"ts":"{ts}","session":"s1","event":"skill","name":"{name}","path":{path},"location":{location},"hash":{hash}}}"#
    )
}

fn rules_file(ts: &str, stem: &str, location: &str, h: &str) -> String {
    format!(
        r#"{{"v":1,"ts":"{ts}","session":"s2","event":"instructions","name":null,"path":"/p/.claude/rules/{stem}.md","location":"{location}","hash":"{h}","load_reason":"session_start"}}"#
    )
}

fn expected() -> Expected {
    Expected::new([
        ((slug("global"), Form::Skill), hash(A)),
        ((slug("project-relearn"), Form::RulesLayer), hash(A)),
    ])
}

fn homes() -> Vec<LayerSlug> {
    vec![slug("global"), slug("domain-java"), slug("project-relearn")]
}

// --- parsing -------------------------------------------------------------

#[test]
fn a_skill_load_is_attributed_to_its_home_and_location() {
    let log = parse_log(&skill("2026-10-31T10:00:00Z", "global", "user", Some(A))).expect("parses");
    let e = &log.events()[0];
    assert_eq!(e.home(), &slug("global"));
    assert_eq!(e.form(), Form::Skill);
    assert_eq!(e.location(), Location::User);
    assert_eq!(e.hash(), Some(&hash(A)));
    assert_eq!(e.date(), date("2026-10-31"));
}

#[test]
fn an_account_synced_name_is_the_same_home_from_another_location() {
    let log = parse_log(&skill(
        "2026-10-31T10:00:00Z",
        "anthropic-skills:global",
        "account-synced",
        Some(B),
    ))
    .expect("parses");
    assert_eq!(log.events()[0].home(), &slug("global"));
    assert_eq!(log.events()[0].location(), Location::AccountSynced);
}

#[test]
fn a_rules_file_load_is_attributed_by_its_stem() {
    let log = parse_log(&rules_file(
        "2026-10-31T10:00:00Z",
        "project-relearn",
        "project",
        A,
    ))
    .expect("parses");
    let e = &log.events()[0];
    assert_eq!(e.home(), &slug("project-relearn"));
    assert_eq!(e.form(), Form::RulesLayer);
    assert_eq!(e.location(), Location::Project);
}

#[test]
fn a_load_that_is_not_a_relearn_home_is_counted_and_skipped() {
    let text = [
        skill("2026-10-31T10:00:00Z", "pdf", "user", None),
        r#"{"v":1,"ts":"2026-10-31T10:00:00Z","session":"s","event":"instructions","name":null,"path":"/p/CLAUDE.md","location":"project","hash":null}"#.to_owned(),
    ]
    .join("\n");
    let log = parse_log(&text).expect("parses");
    assert!(log.events().is_empty());
    assert_eq!(log.ignored(), 2);
}

#[test]
fn a_malformed_line_names_its_line() {
    let text = format!(
        "{}\nnot json\n",
        skill("2026-10-31T10:00:00Z", "global", "user", None)
    );
    assert!(matches!(
        parse_log(&text),
        Err(LoadLogError::NotJson { line: 2, .. })
    ));
    let bad_location = skill("2026-10-31T10:00:00Z", "global", "attic", None);
    assert!(matches!(
        parse_log(&bad_location),
        Err(LoadLogError::Location { line: 1, .. })
    ));
}

#[test]
fn a_window_cannot_end_before_it_starts() {
    assert!(Window::new(date("2026-10-31"), date("2026-10-30")).is_none());
}

// --- the report ----------------------------------------------------------

#[test]
fn a_home_never_loaded_in_the_window_is_cold() {
    let log = parse_log(&skill("2026-10-31T10:00:00Z", "global", "user", Some(A))).expect("parses");
    let report = read(
        log.events(),
        window("2026-10-01", "2026-10-31"),
        &expected(),
        &homes(),
    );
    assert!(report.findings().contains(&LoadFinding::Cold {
        home: slug("domain-java")
    }));
    assert!(!report.findings().iter().any(|f| matches!(
        f,
        LoadFinding::Cold { home } if *home == slug("global")
    )));
}

#[test]
fn a_load_with_a_hash_build_would_not_emit_is_stale_and_says_where() {
    let log = parse_log(&skill(
        "2026-10-31T10:00:00Z",
        "anthropic-skills:global",
        "account-synced",
        Some(B),
    ))
    .expect("parses");
    let report = read(
        log.events(),
        window("2026-10-31", "2026-10-31"),
        &expected(),
        &homes(),
    );
    assert!(report.findings().iter().any(|f| matches!(
        f,
        LoadFinding::Stale { home, form: Form::Skill, location: Location::AccountSynced, found: Some(h), sessions: 1 }
            if *home == slug("global") && *h == hash(B)
    )));
}

#[test]
fn a_copy_with_no_marker_is_stale_because_build_always_writes_one() {
    let log = parse_log(&skill("2026-10-31T10:00:00Z", "global", "user", None)).expect("parses");
    let report = read(
        log.events(),
        window("2026-10-31", "2026-10-31"),
        &expected(),
        &homes(),
    );
    assert!(report.findings().iter().any(|f| matches!(
        f,
        LoadFinding::Stale {
            found: None,
            location: Location::User,
            ..
        }
    )));
}

#[test]
fn one_home_from_two_location_classes_in_a_window_is_shadowed() {
    let text = [
        skill("2026-10-30T10:00:00Z", "global", "user", Some(A)),
        skill(
            "2026-10-31T10:00:00Z",
            "plug:global",
            "account-synced",
            Some(A),
        ),
    ]
    .join("\n");
    let log = parse_log(&text).expect("parses");
    let report = read(
        log.events(),
        window("2026-10-30", "2026-10-31"),
        &expected(),
        &homes(),
    );
    assert!(report.findings().iter().any(|f| matches!(
        f,
        LoadFinding::Shadowed { home, form: Form::Skill, locations }
            if *home == slug("global") && locations == &vec![Location::User, Location::AccountSynced]
    )));
    // A narrower window sees one class, and nothing is shadowed.
    let narrow = read(
        log.events(),
        window("2026-10-31", "2026-10-31"),
        &expected(),
        &homes(),
    );
    assert!(
        !narrow
            .findings()
            .iter()
            .any(|f| matches!(f, LoadFinding::Shadowed { .. }))
    );
}

#[test]
fn a_window_with_no_load_at_all_is_unchecked_not_all_cold() {
    let log = parse_log(&skill("2026-09-01T10:00:00Z", "global", "user", Some(A))).expect("parses");
    let report = read(
        log.events(),
        window("2026-10-01", "2026-10-31"),
        &expected(),
        &homes(),
    );
    assert_eq!(report.outcome(), Outcome::Unchecked);
}

#[test]
fn a_current_load_from_one_place_with_every_home_seen_is_clean() {
    let text = [
        skill("2026-10-31T10:00:00Z", "global", "user", Some(A)),
        rules_file("2026-10-31T10:00:00Z", "project-relearn", "project", A),
    ]
    .join("\n");
    let log = parse_log(&text).expect("parses");
    let homes = vec![slug("global"), slug("project-relearn")];
    let report = read(
        log.events(),
        window("2026-10-31", "2026-10-31"),
        &expected(),
        &homes,
    );
    assert_eq!(report.outcome(), Outcome::Clean, "{:?}", report.findings());
    assert_eq!(report.loaded(&slug("global"), Location::User), 1);
}

// --- the hook's shared shape ---------------------------------------------

#[test]
fn the_skill_name_is_read_from_the_field_the_transcript_parser_reads() {
    use relearn::probe::skill_named;
    let skill = serde_json::json!({"skill": "global", "args": "x"});
    let command = serde_json::json!({"command": "domain-rust"});
    let neither = serde_json::json!({"args": "x"});
    assert_eq!(skill_named(&skill), Some("global"));
    assert_eq!(skill_named(&command), Some("domain-rust"));
    assert_eq!(skill_named(&neither), None);
}

#[test]
fn the_pack_logs_both_events_and_never_exits_two() {
    let files = relearn::emit::load_log::emit();
    let paths: Vec<&str> = files.iter().map(|f| f.path().as_str()).collect();
    assert_eq!(
        paths,
        vec![
            "hooks/load-log-settings-snippet.md",
            "hooks/relearn-load-log.sh",
            "hooks/test-load-log.sh",
        ]
    );
    let script = files[1].contents();
    assert!(script.contains("PostToolUse)"), "handles the Skill call");
    assert!(
        script.contains("InstructionsLoaded)"),
        "handles rules-layer loads"
    );
    assert!(
        !script.lines().any(|l| l.trim_start().starts_with("exit 2")),
        "exit 2 on PostToolUse would put this script's stderr in front of the model"
    );
    let snippet = files[0].contents();
    assert!(snippet.contains(r#""matcher": "Skill""#));
    assert!(snippet.contains(r#""InstructionsLoaded""#));
}

#[test]
fn a_detector_cannot_take_a_load_log_file_name() {
    for name in ["relearn-load-log", "test-load-log"] {
        assert!(
            relearn::rule::DetectorName::is_reserved(name),
            "{name} is a load-log pack file installed beside the detectors"
        );
    }
}

// --- laws ----------------------------------------------------------------

fn arb_line() -> impl Strategy<Value = String> {
    (
        prop::sample::select(vec![
            "global",
            "p:global",
            "domain-java",
            "pdf",
            "project-relearn",
        ]),
        prop::sample::select(vec!["user", "project", "account-synced", "null"]),
        prop::sample::select(vec!["2026-10-29", "2026-10-30", "2026-10-31", "2026-11-01"]),
        prop::bool::ANY,
    )
        .prop_map(|(name, loc, day, current)| {
            skill(
                &format!("{day}T00:00:00Z"),
                name,
                loc,
                Some(if current { A } else { B }),
            )
        })
}

proptest! {
    /// Every home load inside the window is counted exactly once in LOADED,
    /// and nothing outside it is counted at all.
    #[test]
    fn loaded_counts_every_in_window_home_load_once(lines in prop::collection::vec(arb_line(), 0..30)) {
        let log = parse_log(&lines.join("\n")).expect("generated lines parse");
        let w = window("2026-10-30", "2026-10-31");
        let report = read(log.events(), w, &expected(), &homes());
        let in_window = log.events().iter().filter(|e| w.contains(e.date())).count();
        prop_assert_eq!(report.loaded_total() as usize, in_window);
    }
}

#[test]
fn every_runner_case_expects_the_name_the_transcript_parser_reads() {
    for case in relearn::emit::load_log::skill_cases() {
        let input: serde_json::Value =
            serde_json::from_str(case.tool_input()).expect("a case's tool_input is JSON");
        assert_eq!(
            relearn::probe::skill_named(&input),
            case.name(),
            "the hook's jq and relearn's parser must agree on {}",
            case.tool_input()
        );
    }
}
