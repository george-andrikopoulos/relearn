//! The probe tally split by load: a WITH-arm violation where the rule's skill
//! was never opened is a delivery failure, not a prose failure.
//!
//! The split is a law over results (`proptest`: it partitions the WITH arm),
//! with unit pins for `LoadFailure`, for the name an account-synced copy
//! loads under, and for the two deliveries the split does not apply to.

use proptest::prelude::*;
use relearn::probe::{
    Count, Delivery, Load, LoadEvidence, LoadRate, Recommendation, SkillName, parse_results,
    recommend, render, tally,
};
use relearn::rule::RuleTag;

fn count(v: u32, n: u32) -> Count {
    Count::new(v, n).expect("valid count")
}

fn rate(loaded: u32, sessions: u32) -> LoadRate {
    LoadRate::new(loaded, sessions).expect("valid load rate")
}

// --- the recommendation -----------------------------------------------------

#[test]
fn a_skill_never_opened_is_a_load_failure_not_a_graduation() {
    // The 2026-10-31 shape: both arms violate every time, and no WITH session
    // opened the skill, so the rule's text never reached the model.
    let evidence = LoadEvidence::new(rate(0, 10), None);
    assert_eq!(
        recommend(Some(count(10, 10)), Some(count(10, 10)), Some(evidence)),
        Recommendation::LoadFailure
    );
    // Without the split, the same arms read as the prose failing.
    assert_eq!(
        recommend(Some(count(10, 10)), Some(count(10, 10)), None),
        Recommendation::GraduateNow
    );
}

#[test]
fn a_rarely_opened_skill_that_held_whenever_opened_is_a_load_failure() {
    let evidence = LoadEvidence::new(rate(1, 12), Some(count(0, 1)));
    assert_eq!(
        recommend(Some(count(8, 12)), Some(count(7, 12)), Some(evidence)),
        Recommendation::LoadFailure
    );
}

#[test]
fn a_rarely_opened_skill_that_failed_when_opened_is_still_a_prose_failure() {
    // Low load, but the prose violated in the sessions it reached: firmer
    // delivery would deliver prose that does not hold.
    let evidence = LoadEvidence::new(rate(2, 12), Some(count(2, 2)));
    assert_eq!(
        recommend(Some(count(8, 12)), Some(count(9, 12)), Some(evidence)),
        Recommendation::GraduateNow
    );
}

#[test]
fn a_well_opened_skill_is_judged_on_its_arms_as_before() {
    let evidence = LoadEvidence::new(rate(9, 10), Some(count(9, 9)));
    assert_eq!(
        recommend(Some(count(9, 10)), Some(count(9, 10)), Some(evidence)),
        Recommendation::GraduateNow
    );
}

#[test]
fn a_measurable_reduction_is_kept_even_when_load_is_low() {
    let evidence = LoadEvidence::new(rate(1, 12), Some(count(0, 1)));
    assert_eq!(
        recommend(Some(count(12, 12)), Some(count(1, 12)), Some(evidence)),
        Recommendation::Keep
    );
}

#[test]
fn a_silent_without_arm_stays_non_discriminating_whatever_the_load() {
    let evidence = LoadEvidence::new(rate(0, 10), None);
    assert_eq!(
        recommend(Some(count(0, 10)), Some(count(0, 10)), Some(evidence)),
        Recommendation::NonDiscriminating
    );
}

#[test]
fn a_load_rate_needs_at_least_one_session_and_no_more_loads_than_sessions() {
    assert!(LoadRate::new(0, 0).is_none());
    assert!(LoadRate::new(3, 2).is_none());
    assert_eq!(rate(6, 9).loaded(), 6);
    assert_eq!(rate(6, 9).sessions(), 9);
}

// --- the name a skill is invoked under -------------------------------------

#[test]
fn a_skill_matches_its_own_name_and_an_account_synced_plugin_prefix() {
    let global = SkillName::new("global").expect("non-empty");
    assert!(global.is_invoked_as("global"));
    assert!(global.is_invoked_as("anthropic-skills:global"));
    assert!(!global.is_invoked_as("domain-rust"));
    assert!(!global.is_invoked_as("global-extra"));
    assert!(!global.is_invoked_as("x:global-extra"));
    assert!(SkillName::new("").is_none());
}

// --- the tally ---------------------------------------------------------------

const RESULTS: &str = "#tag\tarm\tsample\toutcome\tcost_usd\ttokens\tloaded\tskills_invoked\n\
R:a\twith\t1\theld\t0.1\t10\tskill:global\tglobal\n\
R:a\twith\t2\tviolated\t0.1\t10\tskill:global\t\n\
R:a\twith\t3\tviolated\t0.1\t10\tskill:global\tplug:global\n\
R:a\twith\t4\tinconclusive\t0.1\t10\tskill:global\tdomain-rust\n\
R:a\twithout\t1\tviolated\t0.1\t10\t\t\n\
R:b\twith\t1\theld\t0.1\t10\t\t\n\
R:b\twithout\t1\tviolated\t0.1\t10\t\t\n\
R:c\twith\t1\theld\t0.1\t10\t\t\n\
R:c\twithout\t1\tviolated\t0.1\t10\t\t\n";

fn delivery(tag: &RuleTag) -> Delivery {
    match tag.as_str() {
        "R:a" => Delivery::Skill(SkillName::new("global").expect("non-empty")),
        "R:b" => Delivery::RulesLayer,
        _ => Delivery::NotInLibrary,
    }
}

#[test]
fn the_with_arm_is_split_by_whether_the_rules_own_skill_was_opened() {
    let rows = parse_results(RESULTS).expect("parses");
    let (tallies, _) = tally(&rows, delivery);
    let Load::Split(split) = tallies[0].load() else {
        panic!("R:a is delivered as a skill, so its WITH arm is split");
    };
    // Opened as `global` (held) and as `plug:global` (violated); not opened
    // in the empty row (violated) or when only another skill was opened
    // (inconclusive).
    assert_eq!(split.loaded().count(), Some(count(1, 2)));
    assert_eq!(split.not_loaded().count(), Some(count(1, 1)));
    // Judged sessions are the denominator, inconclusive included.
    assert_eq!(split.rate(), Some(rate(2, 4)));
}

#[test]
fn a_rules_layer_and_an_unknown_rule_are_never_given_a_load_rate() {
    let rows = parse_results(RESULTS).expect("parses");
    let (tallies, _) = tally(&rows, delivery);
    assert!(matches!(tallies[1].load(), Load::RulesLayer));
    assert!(matches!(tallies[2].load(), Load::NotInLibrary));
    let table = render(&tallies, relearn::probe::MicroUsd::from_micros(0));
    let line_b = table
        .lines()
        .find(|l| l.contains("`R:b`"))
        .expect("R:b row");
    // Cells: rule, without, with, reduction, unscored x2, then the three load
    // cells. The load-rate cell says why there is no split; the two split
    // cells are empty, not a rate of zero.
    let cells: Vec<&str> = line_b.split('|').map(str::trim).collect();
    assert_eq!(cells[7], "n/a: rules layer, no Skill call", "{line_b}");
    assert_eq!((cells[8], cells[9]), ("—", "—"), "{line_b}");
    let line_c = table
        .lines()
        .find(|l| l.contains("`R:c`"))
        .expect("R:c row");
    assert!(line_c.contains("n/a: not in the library"), "{line_c}");
}

#[test]
fn a_results_file_without_the_skills_column_reads_as_nothing_opened() {
    let rows = parse_results("R:a\twith\t1\theld\n").expect("four columns are enough");
    let (tallies, _) = tally(&rows, delivery);
    let Load::Split(split) = tallies[0].load() else {
        panic!("split expected");
    };
    assert_eq!(split.rate(), Some(rate(0, 1)));
}

// --- laws ------------------------------------------------------------------

fn arb_row() -> impl Strategy<Value = String> {
    (
        prop::sample::select(vec!["with", "without"]),
        prop::sample::select(vec![
            "held",
            "violated",
            "inconclusive",
            "contaminated",
            "error",
        ]),
        prop::sample::select(vec![
            "",
            "global",
            "p:global",
            "domain-rust",
            "global,domain-rust",
        ]),
    )
        .prop_map(|(arm, outcome, invoked)| format!("R:a\t{arm}\t1\t{outcome}\t\t\t\t{invoked}"))
}

proptest! {
    /// Every WITH session lands on exactly one side of the split, so the two
    /// halves add back to the whole arm, outcome by outcome.
    #[test]
    fn the_split_partitions_the_with_arm(rows in prop::collection::vec(arb_row(), 1..40)) {
        let text = rows.join("\n");
        let rows = parse_results(&text).expect("generated rows parse");
        let (tallies, _) = tally(&rows, delivery);
        let t = &tallies[0];
        let Load::Split(split) = t.load() else {
            return Err(TestCaseError::fail("R:a is a skill delivery"));
        };
        let whole = t.arm(relearn::probe::Arm::With);
        let (l, n) = (split.loaded(), split.not_loaded());
        prop_assert_eq!(l.held() + n.held(), whole.held());
        prop_assert_eq!(l.violated() + n.violated(), whole.violated());
        let (li, lc, lf) = l.unscored();
        let (ni, nc, nf) = n.unscored();
        prop_assert_eq!((li + ni, lc + nc, lf + nf), whole.unscored());
    }

    /// A load rate whose interval reaches one half is never a load failure:
    /// the outcome is reserved for skills measurably left unopened.
    #[test]
    fn a_load_rate_reaching_one_half_is_never_a_load_failure(
        sessions in 1u32..60, frac in 0.0f64..=1.0, wo in 1u32..30, wi in 0u32..30,
    ) {
        let loaded = ((f64::from(sessions) * frac).floor() as u32).min(sessions);
        let r = rate(loaded, sessions);
        prop_assume!(r.interval().hi() >= 0.5);
        let without = count(wo.min(30), 30);
        let with = count(wi.min(30), 30);
        let evidence = LoadEvidence::new(r, Count::new(0, loaded));
        prop_assert_ne!(
            recommend(Some(without), Some(with), Some(evidence)),
            Recommendation::LoadFailure
        );
    }
}
