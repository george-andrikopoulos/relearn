//! A destination is a typed list of controls, parsed once at the perimeter.
//!
//! Written before the module (red-green). These exercise the public API only —
//! `Controls::parse`, the accessors, and the `Display` that renders a list back
//! to the text a rule file holds.
//!
//! The load-bearing test is [`every_corpus_destination_round_trips_byte_for_byte`]:
//! it is the entire safety argument for changing the representation without
//! changing the on-disk format. If rendering is not the exact inverse of
//! parsing, every rule file and every emitted instruction file moves, and
//! `relearn verify` would be the thing that told us — after the fact.

use relearn::rule::{ControlError, ControlKind, Controls};

/// Every destination the real corpus holds, including the three that name two
/// controls and the one whose artefact name contains a space.
const CORPUS: &[&str] = &[
    "hook:no-case-collision",
    "hook:no-anyhow-in-lib",
    "hook:no-unwrap-in-src",
    "hook:banned-name-in-output",
    "test:the_catalogue_has_not_grown_without_a_migration_to_carry_it",
    "test:the_repository_derives_no_queue_role_from_a_spin_mode (documents) + test:spin_mode_does_not_change_a_single_queue_end_letter (renderers)",
    "hook:gate-verdict-intact (pipeline half) + hook:multiline-pattern-eol (edit half)",
    "test:client/src/app/font_tests.rs + gate:verify.sh emoji_ban",
    "test:tests/ledger.rs::every_enforced_by_row_names_an_artefact_or_declares_itself_exposed + test:tests/pack_counts.rs",
];

#[test]
fn every_corpus_destination_round_trips_byte_for_byte() {
    for written in CORPUS {
        let parsed = Controls::parse(written).expect("a corpus destination parses");
        assert_eq!(
            &parsed.to_string(),
            written,
            "rendering must be the exact inverse of parsing"
        );
    }
}

#[test]
fn a_name_containing_a_space_survives() {
    // `gate:verify.sh emoji_ban` — the case the old whitespace-splitting reader
    // in `report` silently truncated to nothing.
    let parsed = Controls::parse("gate:verify.sh emoji_ban").expect("parses");
    let [only] = parsed.as_slice() else {
        panic!("expected exactly one control")
    };
    assert_eq!(only.name(), "verify.sh emoji_ban");
    assert_eq!(only.kind(), ControlKind::Gate);
    assert_eq!(only.covers(), None);
}

#[test]
fn a_coverage_note_is_read_and_is_not_part_of_the_name() {
    let parsed = Controls::parse("hook:gate-verdict-intact (pipeline half)").expect("parses");
    let [only] = parsed.as_slice() else {
        panic!("expected exactly one control")
    };
    assert_eq!(only.name(), "gate-verdict-intact");
    assert_eq!(only.covers(), Some("pipeline half"));
}

#[test]
fn a_name_may_contain_parentheses_that_are_not_a_note() {
    // The note is found from the END, so an interior paren with no trailing one
    // stays part of the name. Splitting on the FIRST `(` would have eaten half
    // of every test name that takes an argument.
    let parsed = Controls::parse("test:a_test_of_fn(x)_behaviour").expect("parses");
    let [only] = parsed.as_slice() else {
        panic!("expected exactly one control")
    };
    assert_eq!(only.name(), "a_test_of_fn(x)_behaviour");
    assert_eq!(only.covers(), None);
}

#[test]
fn two_controls_are_two_entries_not_one_sentence() {
    let parsed = Controls::parse("hook:a (first) + gate:b").expect("parses");
    assert_eq!(parsed.as_slice().len(), 2);
    assert_eq!(parsed.as_slice()[0].covers(), Some("first"));
    assert_eq!(parsed.as_slice()[1].kind(), ControlKind::Gate);
    assert_eq!(parsed.as_slice()[1].covers(), None);
}

#[test]
fn kinds_are_distinct_and_sorted() {
    let parsed = Controls::parse("test:a + hook:b + test:c").expect("parses");
    assert_eq!(
        parsed.kinds(),
        vec![ControlKind::UnitTest, ControlKind::Hook]
    );
}

#[test]
fn a_destination_naming_nothing_is_refused() {
    assert_eq!(Controls::parse(""), Err(ControlError::Empty));
    assert_eq!(Controls::parse("   "), Err(ControlError::Empty));
}

#[test]
fn prose_without_a_kind_is_refused() {
    // The state this module exists to make unreachable. Before it,
    // `to = "we added a test"` parsed, emitted, and reported no control kind —
    // a graduation that named nothing a tool could find.
    assert!(matches!(
        Controls::parse("we added a test"),
        Err(ControlError::NoKind(_))
    ));
}

#[test]
fn an_unknown_kind_is_refused() {
    assert!(matches!(
        Controls::parse("lint:something"),
        Err(ControlError::UnknownKind(k)) if k == "lint"
    ));
}

#[test]
fn a_kind_naming_no_artefact_is_refused() {
    assert!(matches!(
        Controls::parse("hook:"),
        Err(ControlError::NoName(_))
    ));
}

#[test]
fn an_empty_coverage_note_is_refused() {
    assert!(matches!(
        Controls::parse("hook:a ()"),
        Err(ControlError::EmptyCoverage(_))
    ));
}

#[test]
fn one_bad_entry_fails_the_whole_destination() {
    // All-or-nothing. A half-read destination would emit a note naming one
    // control while the rule believes it has two.
    assert!(Controls::parse("hook:good + garbage").is_err());
}

#[test]
fn a_prefix_is_not_the_published_spelling_for_the_two_test_kinds() {
    // Rule files have always written `test:`, and every existing file must stay
    // byte-identical; a report needs to tell a unit test from a property test.
    assert_eq!(ControlKind::UnitTest.prefix(), "test");
    assert_eq!(ControlKind::UnitTest.published(), "unit-test");
    assert_eq!(ControlKind::PropertyTest.prefix(), "property");
    assert_eq!(ControlKind::PropertyTest.published(), "property-test");
}
