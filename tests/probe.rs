//! A rule's `[probe]` table: what parses, what is refused, how one session is
//! judged, and the statistics a pilot reports.
//!
//! Every schema test goes through `parse_document`, the perimeter, because a
//! probe is only ever constructed there. The statistics are laws over the
//! whole input space (`proptest`), with unit pins for the published values and
//! for the four recommendations.

use proptest::prelude::*;
use relearn::probe::{
    Count, MicroUsd, Recommendation, newcombe, recommend, session_from_stream_json, wilson,
};
use relearn::rule::{
    Checker, EventKind, ParseError, ProbeError, SessionEvent, Verdict, WorkspaceChange,
    parse_document, to_document,
};

/// A complete rule document around the given trailing tables.
fn doc(tables: &str) -> String {
    format!(
        "+++\ntag = \"R:synthetic\"\ntitle = \"t\"\nerror_class = \"e\"\n\
         home = {{ kind = \"global\" }}\ncreated = \"2026-10-30\"\norigin = \"mined\"\n\
         status = {{ kind = \"active\" }}\nincident = \"synthetic\"\n{tables}+++\n\nBody.\n"
    )
}

const TRACE: &str = r#"
[probe]
task = "Write a status report on the backup service into REPORT.md."
workspace = "fixture"

[probe.checker]
kind = "trace"
expect = { event = "shell", pattern = 'svc(\.sh)? +status' }
window = "before-first"
anchor = { event = "write", pattern = '(^|/)REPORT\.md$' }
"#;

const DIFF: &str = r#"
[probe]
task = "Rename the timeout key."
workspace = "fixture"

[probe.checker]
kind = "diff"
anchor = '(^|/)parse\.sh$'

[[probe.checker.require]]
changed = '(^|/)example\.conf$'

[[probe.checker.require]]
file = "example.conf"
lacks = '(?m)^timeout='
"#;

const RUN: &str = r#"
[probe]
task = "Add a check."
workspace = "fixture"

[probe.checker]
kind = "run"
adversary = "A config file carrying the marker, then the gate."
"#;

fn probe_error(document: &str) -> ProbeError {
    match parse_document(document) {
        Err(ParseError::Probe(source)) => source,
        other => panic!("expected a probe error, got {other:?}"),
    }
}

fn ev(kind: EventKind, subject: &str) -> SessionEvent {
    SessionEvent::new(kind, subject)
}

// --- schema ---------------------------------------------------------------

#[test]
fn a_rule_without_a_probe_parses_to_none() {
    let rule = parse_document(&doc("")).expect("parses");
    assert!(rule.probe().is_none());
}

#[test]
fn each_checker_kind_parses() {
    for (table, kind) in [(TRACE, "trace"), (DIFF, "diff"), (RUN, "run")] {
        let rule = parse_document(&doc(table)).expect("parses");
        let probe = rule.probe().expect("probe present");
        assert_eq!(probe.checker().kind(), kind);
    }
}

#[test]
fn a_probe_round_trips_through_the_serializer() {
    for table in [TRACE, DIFF, RUN] {
        let rule = parse_document(&doc(table)).expect("parses");
        let again = parse_document(&to_document(&rule)).expect("re-parses");
        assert_eq!(rule, again);
    }
}

#[test]
fn a_rubric_is_named_and_refused_as_not_built() {
    let table = TRACE.replace("kind = \"trace\"", "kind = \"rubric\"");
    assert_eq!(probe_error(&doc(&table)), ProbeError::RubricNotBuilt);
}

#[test]
fn an_unknown_checker_kind_is_refused() {
    let table = RUN.replace("kind = \"run\"", "kind = \"vibes\"");
    assert_eq!(
        probe_error(&doc(&table)),
        ProbeError::UnknownChecker("vibes".into())
    );
}

#[test]
fn an_empty_task_is_refused() {
    let table = RUN.replace("task = \"Add a check.\"", "task = \"  \"");
    assert_eq!(probe_error(&doc(&table)), ProbeError::EmptyTask);
}

#[test]
fn a_pattern_that_does_not_compile_is_refused() {
    let table = TRACE.replace(r"svc(\.sh)? +status", "svc(");
    assert!(matches!(
        probe_error(&doc(&table)),
        ProbeError::Pattern { .. }
    ));
}

#[test]
fn a_field_from_another_kind_is_refused_not_ignored() {
    let table = format!("{RUN}window = \"after-last\"\n");
    assert_eq!(
        probe_error(&doc(&table)),
        ProbeError::FieldNotAllowed {
            kind: "run",
            field: "window"
        }
    );
}

#[test]
fn a_diff_with_no_clause_is_refused() {
    let table = "\n[probe]\ntask = \"t\"\nworkspace = \"empty\"\n\n[probe.checker]\nkind = \"diff\"\nanchor = 'x'\n";
    assert_eq!(probe_error(&doc(table)), ProbeError::NoClause);
}

#[test]
fn a_clause_setting_two_conditions_is_refused() {
    let table = DIFF.replace(
        "changed = '(^|/)example\\.conf$'",
        "changed = 'a'\ndetector = \"x\"",
    );
    assert_eq!(probe_error(&doc(&table)), ProbeError::ClauseShape);
}

#[test]
fn a_clause_path_that_escapes_the_workspace_is_refused() {
    let table = DIFF.replace("file = \"example.conf\"", "file = \"../etc/passwd\"");
    assert!(matches!(
        probe_error(&doc(&table)),
        ProbeError::ClausePath(_)
    ));
}

#[test]
fn a_detector_clause_must_name_this_rules_own_detector() {
    let table = DIFF.replace(
        "changed = '(^|/)example\\.conf$'",
        "detector = \"someone-elses\"",
    );
    assert_eq!(
        probe_error(&doc(&table)),
        ProbeError::UnknownDetector("someone-elses".into())
    );
}

// --- trace judging ----------------------------------------------------------

fn trace_checker() -> relearn::rule::TraceChecker {
    let rule = parse_document(&doc(TRACE)).expect("parses");
    match rule.probe().expect("probe").checker() {
        Checker::Trace(t) => t.clone(),
        other => panic!("expected trace, got {other:?}"),
    }
}

#[test]
fn a_query_before_the_claim_holds() {
    let events = [
        ev(EventKind::Read, "STATUS.md"),
        ev(EventKind::Shell, "./svc.sh status backup"),
        ev(EventKind::Write, "C:/tmp/ws/REPORT.md"),
    ];
    assert_eq!(trace_checker().judge(&events), Verdict::Held);
}

#[test]
fn a_query_only_after_the_claim_is_a_violation() {
    let events = [
        ev(EventKind::Write, "C:/tmp/ws/REPORT.md"),
        ev(EventKind::Shell, "./svc.sh status backup"),
    ];
    assert_eq!(trace_checker().judge(&events), Verdict::Violated);
}

#[test]
fn no_anchor_is_inconclusive_not_held() {
    let events = [ev(EventKind::Shell, "./svc.sh status backup")];
    assert_eq!(trace_checker().judge(&events), Verdict::Inconclusive);
}

fn arb_event() -> impl Strategy<Value = SessionEvent> {
    let kind = prop_oneof![
        Just(EventKind::Shell),
        Just(EventKind::Write),
        Just(EventKind::Read),
        Just(EventKind::Text),
    ];
    let subject = prop_oneof![
        Just("REPORT.md".to_owned()),
        Just("./svc.sh status".to_owned()),
        Just("ls".to_owned()),
        "[a-z ./]{0,12}",
    ];
    (kind, subject).prop_map(|(k, s)| SessionEvent::new(k, s))
}

proptest! {
    /// A session with no anchor event is never judged held or violated.
    #[test]
    fn without_an_anchor_the_verdict_is_inconclusive(events in prop::collection::vec(arb_event(), 0..12)) {
        let checker = trace_checker();
        let no_anchor: Vec<SessionEvent> = events
            .into_iter()
            .filter(|e| !(e.kind() == EventKind::Write && e.subject().ends_with("REPORT.md")))
            .collect();
        prop_assert_eq!(checker.judge(&no_anchor), Verdict::Inconclusive);
    }

    /// Putting the expected event first makes any session with an anchor hold.
    #[test]
    fn the_expected_event_first_always_holds(events in prop::collection::vec(arb_event(), 0..12)) {
        let mut with_anchor = vec![ev(EventKind::Shell, "svc status")];
        with_anchor.extend(events);
        with_anchor.push(ev(EventKind::Write, "REPORT.md"));
        prop_assert_eq!(trace_checker().judge(&with_anchor), Verdict::Held);
    }
}

// --- diff judging -----------------------------------------------------------

fn diff_judge(change: &WorkspaceChange) -> Verdict {
    let rule = parse_document(&doc(DIFF)).expect("parses");
    match rule.probe().expect("probe").checker() {
        Checker::Diff(d) => d.judge(change, rule.detectors()),
        other => panic!("expected diff, got {other:?}"),
    }
}

#[test]
fn the_example_edited_with_the_code_holds() {
    let change = WorkspaceChange::new(
        vec!["parse.sh".into(), "example.conf".into()],
        vec![("example.conf".into(), "timeout_ms=5\n".into())],
    );
    assert_eq!(diff_judge(&change), Verdict::Held);
}

#[test]
fn the_code_changed_and_the_example_left_is_a_violation() {
    let change = WorkspaceChange::new(
        vec!["parse.sh".into()],
        vec![("example.conf".into(), "timeout=5\n".into())],
    );
    assert_eq!(diff_judge(&change), Verdict::Violated);
}

#[test]
fn the_example_touched_but_still_stale_is_a_violation() {
    let change = WorkspaceChange::new(
        vec!["parse.sh".into(), "example.conf".into()],
        vec![("example.conf".into(), "# edited\ntimeout=5\n".into())],
    );
    assert_eq!(diff_judge(&change), Verdict::Violated);
}

#[test]
fn nothing_changed_at_the_anchor_is_inconclusive() {
    let change = WorkspaceChange::new(vec!["README.md".into()], vec![]);
    assert_eq!(diff_judge(&change), Verdict::Inconclusive);
}

// --- transcripts ------------------------------------------------------------

#[test]
fn a_stream_json_transcript_yields_events_in_order_and_the_init_lists() {
    let stream = concat!(
        r#"{"type":"system","subtype":"init","skills":["global"],"plugins":[{"name":"p"}],"mcp_servers":[{"name":"m","status":"connected"}]}"#,
        "\n",
        r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Checking."},{"type":"tool_use","name":"Bash","input":{"command":"./svc.sh status"}}]}}"#,
        "\n",
        r#"{"type":"user","message":{"content":[{"type":"tool_result","content":"running"}]}}"#,
        "\n",
        r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Skill","input":{"skill":"global"}},{"type":"tool_use","name":"Write","input":{"file_path":"C:\\tmp\\ws\\REPORT.md","content":"x"}}]}}"#,
        "\n",
        "not json is skipped only if blank\n",
    );
    // A line that is not JSON is a malformed transcript, not something to skip.
    assert!(session_from_stream_json(stream).is_err());

    let stream = stream.replace("not json is skipped only if blank\n", "\n").replace(
        "\n\n",
        "\n{\"type\":\"result\",\"subtype\":\"success\",\"total_cost_usd\":0.25,\"usage\":{\"input_tokens\":10,\"output_tokens\":20,\"cache_read_input_tokens\":30,\"cache_creation_input_tokens\":40}}\n",
    );
    let session = session_from_stream_json(&stream).expect("parses");
    let kinds: Vec<EventKind> = session.events().iter().map(SessionEvent::kind).collect();
    assert_eq!(kinds, [EventKind::Text, EventKind::Shell, EventKind::Write]);
    assert_eq!(session.events()[2].subject(), "C:/tmp/ws/REPORT.md");
    assert_eq!(session.loaded(), ["skill:global", "plugin:p", "mcp:m"]);
    assert_eq!(session.skills_invoked(), ["global"]);
    assert_eq!(session.cost().map(MicroUsd::micros), Some(250_000));
    assert_eq!(session.tokens(), Some(100));
}

// --- statistics -------------------------------------------------------------

fn count(v: u32, n: u32) -> Count {
    Count::new(v, n).expect("valid count")
}

#[test]
fn a_count_cannot_have_more_violations_than_samples_or_no_samples() {
    assert!(Count::new(3, 2).is_none());
    assert!(Count::new(0, 0).is_none());
}

#[test]
fn wilson_matches_the_published_value() {
    // 2 of 10 at 95%: Wilson's interval is (0.0567, 0.5098).
    let i = wilson(count(2, 10));
    assert!((i.lo() - 0.0567).abs() < 1e-3, "{i:?}");
    assert!((i.hi() - 0.5098).abs() < 1e-3, "{i:?}");
}

#[test]
fn newcombe_matches_the_published_value() {
    // Newcombe 1998, example (f): 56/70 against 48/80 -> (0.0524, 0.3339).
    let d = newcombe(count(56, 70), count(48, 80));
    assert!((d.lo() - 0.0524).abs() < 1e-3, "{d:?}");
    assert!((d.hi() - 0.3339).abs() < 1e-3, "{d:?}");
}

#[test]
fn the_four_recommendations() {
    use Recommendation::*;
    assert_eq!(
        recommend(Some(count(0, 10)), Some(count(0, 10))),
        NonDiscriminating
    );
    assert_eq!(recommend(None, Some(count(1, 10))), NonDiscriminating);
    assert_eq!(recommend(Some(count(9, 10)), Some(count(1, 10))), Keep);
    assert_eq!(
        recommend(Some(count(5, 10)), Some(count(5, 10))),
        GraduateNow
    );
    assert_eq!(
        recommend(Some(count(2, 10)), Some(count(0, 10))),
        Underpowered
    );
    assert_eq!(recommend(Some(count(2, 10)), None), NonDiscriminating);
}

proptest! {
    /// Wilson's interval lies in [0, 1] and contains the observed rate.
    #[test]
    fn wilson_is_a_proper_interval_around_the_rate(n in 1u32..500, v_frac in 0.0f64..=1.0) {
        let v = ((f64::from(n) * v_frac).floor() as u32).min(n);
        let i = wilson(count(v, n));
        let p = f64::from(v) / f64::from(n);
        prop_assert!(0.0 <= i.lo() && i.lo() <= p + 1e-12);
        prop_assert!(p - 1e-12 <= i.hi() && i.hi() <= 1.0);
    }

    /// Newcombe's interval lies in [-1, 1] and contains the observed difference.
    #[test]
    fn newcombe_contains_the_observed_difference(
        n1 in 1u32..200, f1 in 0.0f64..=1.0, n2 in 1u32..200, f2 in 0.0f64..=1.0,
    ) {
        let v1 = ((f64::from(n1) * f1).floor() as u32).min(n1);
        let v2 = ((f64::from(n2) * f2).floor() as u32).min(n2);
        let d = newcombe(count(v1, n1), count(v2, n2));
        let diff = f64::from(v1) / f64::from(n1) - f64::from(v2) / f64::from(n2);
        prop_assert!(-1.0 <= d.lo() && d.lo() <= diff + 1e-9);
        prop_assert!(diff - 1e-9 <= d.hi() && d.hi() <= 1.0);
    }

    /// A WITHOUT arm that never violated can never yield a finding.
    #[test]
    fn a_silent_without_arm_is_always_non_discriminating(n in 1u32..100, w in 0u32..100, m in 1u32..100) {
        let with = Count::new(w.min(m), m);
        prop_assert_eq!(recommend(Some(count(0, n)), with), Recommendation::NonDiscriminating);
    }
}

#[test]
fn a_cost_is_parsed_from_its_digits_in_either_json_form() {
    let micros = |s: &str| MicroUsd::parse_decimal(s).map(MicroUsd::micros);
    assert_eq!(micros("0.25"), Some(250_000));
    assert_eq!(micros("3"), Some(3_000_000));
    assert_eq!(micros("1e-5"), Some(10));
    assert_eq!(micros("2.5E-7"), Some(0));
    assert_eq!(micros("1.2345678"), Some(1_234_567));
    assert_eq!(micros("-1"), None);
    assert_eq!(micros("abc"), None);
}
