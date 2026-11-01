//! The `duplicate-paragraph` repository gate (ARCHITECTURE.md, "Proposed
//! 2026-10-31: detector engines", the `R:one-home-per-rule` row, approved
//! 2026-11-01 as the fifth engine, `warn`).
//!
//! The collect stage splits every tracked file in scope into paragraphs
//! (runs of non-blank lines), normalises whitespace to single spaces, and
//! prints `path<TAB>paragraph`. The judge reports a paragraph of at least
//! `min_bytes` bytes that appears in two different files. Bytes, not
//! characters: the generated `awk` runs under `LC_ALL=C`, where `length`
//! counts bytes, and the Rust judge must agree with it on every input.

use proptest::prelude::*;
use relearn::emit::repo;
use relearn::library::{Library, Validated};
use relearn::rule::{Detection, DetectorError, Gate, ParseError, parse_document, to_document};

fn document(detector: &str) -> String {
    format!(
        "+++\ntag = \"R:synthetic-dup\"\ntitle = \"t\"\nerror_class = \"e\"\n\
         home = {{ kind = \"global\" }}\ncreated = \"2026-11-01\"\norigin = \"mined\"\n\
         status = {{ kind = \"active\" }}\nincident = \"synthetic\"\n{detector}+++\n\nBody.\n"
    )
}

fn detector_error(detector: &str) -> DetectorError {
    match parse_document(&document(detector)) {
        Err(ParseError::Detector { source, .. }) => source,
        other => panic!("expected a detector error, got {other:?}"),
    }
}

fn gate(detector: &str) -> Gate {
    let rule = parse_document(&document(detector)).expect("the synthetic gate parses");
    rule.gates()
        .first()
        .cloned() // allow:clone: the test owns one gate beyond the parsed rule's lifetime
        .expect("one gate")
}

fn library(detector: &str) -> Library<Validated> {
    let mut library = Library::new();
    library.push(parse_document(&document(detector)).expect("parses"));
    library.validate().expect("validates")
}

const PARA: &str = "One home for every fact, and a reference everywhere else.";

fn dup() -> String {
    format!(
        r#"
[[detector]]
name = "one-home"
engine = "duplicate-paragraph"
min_bytes = 40
files = ["**/*.md"]
exclude = ["vendor/**"]
events = ["repository-gate"]
action = "warn"
message = "The same paragraph lives in two files."

[[detector.fixture]]
records = ["README.md\t{PARA}", "docs/guide.md\t{PARA}"]
expect = "hit"

[[detector.fixture]]
records = ["README.md\t{PARA}", "README.md\t{PARA}", "docs/guide.md\tA paraphrase of the same idea, in other words."]
expect = "clean"
"#
    )
}

fn judge(records: &[&str]) -> Detection {
    gate(&dup())
        .judge_lines(records)
        .expect("well-formed records")
}

// ---------------------------------------------------------------------------
// The judgement.
// ---------------------------------------------------------------------------

#[test]
fn one_paragraph_in_two_files_is_a_hit() {
    let a = format!("a.md\t{PARA}");
    let b = format!("b/c.md\t{PARA}");
    assert_eq!(judge(&[&a, &b]), Detection::Hit);
}

#[test]
fn the_same_paragraph_twice_in_one_file_is_not() {
    let a = format!("a.md\t{PARA}");
    assert_eq!(judge(&[&a, &a]), Detection::Clean);
}

#[test]
fn a_paragraph_below_the_floor_is_not_read() {
    // 39 bytes against a floor of 40; one more byte crosses it.
    let short = "x".repeat(39);
    let (a, b) = (format!("a.md\t{short}"), format!("b.md\t{short}"));
    assert_eq!(judge(&[&a, &b]), Detection::Clean);
    let long = "x".repeat(40);
    let (a, b) = (format!("a.md\t{long}"), format!("b.md\t{long}"));
    assert_eq!(judge(&[&a, &b]), Detection::Hit);
}

#[test]
fn the_floor_counts_bytes_not_characters() {
    // 20 two-byte characters: 20 characters, 40 bytes.
    let text = "é".repeat(20);
    let (a, b) = (format!("a.md\t{text}"), format!("b.md\t{text}"));
    assert_eq!(judge(&[&a, &b]), Detection::Hit);
}

#[test]
fn a_file_outside_the_scope_is_not_read() {
    let a = format!("a.md\t{PARA}");
    let excluded = format!("vendor/a.md\t{PARA}");
    let not_markdown = format!("src/a.rs\t{PARA}");
    assert_eq!(judge(&[&a, &excluded, &not_markdown]), Detection::Clean);
}

proptest! {
    /// A hit is exactly a long-enough paragraph held by two distinct paths,
    /// whatever the order of the records.
    #[test]
    fn a_hit_is_exactly_a_shared_long_paragraph(
        held in prop::collection::vec((0usize..4, 0usize..3), 0..10)
    ) {
        let paths = ["a.md", "b.md", "c/d.md", "e.md"];
        let texts = [
            "alpha alpha alpha alpha alpha alpha alpha alpha",
            "beta beta beta beta beta beta beta beta beta beta",
            "short",
        ];
        let lines: Vec<String> = held
            .iter()
            .map(|(p, t)| format!("{}\t{}", paths[*p], texts[*t]))
            .collect();
        let refs: Vec<&str> = lines.iter().map(String::as_str).collect();
        let shared = (0..2).any(|t| {
            let mut owners: Vec<usize> =
                held.iter().filter(|(_, x)| *x == t).map(|(p, _)| *p).collect();
            owners.sort_unstable();
            owners.dedup();
            owners.len() >= 2
        });
        let expected = if shared { Detection::Hit } else { Detection::Clean };
        prop_assert_eq!(judge(&refs), expected);
        let mut reversed = refs.clone();
        reversed.reverse();
        prop_assert_eq!(judge(&reversed), expected);
    }
}

// ---------------------------------------------------------------------------
// What parses and what is refused.
// ---------------------------------------------------------------------------

#[test]
fn a_paragraph_record_must_be_whitespace_normalised() {
    for bad in [" leading", "trailing ", "two  spaces", ""] {
        let detector = dup().replace(
            &format!("records = [\"README.md\\t{PARA}\", \"docs/guide.md"),
            &format!("records = [\"README.md\\t{bad}\", \"docs/guide.md"),
        );
        let err = detector_error(&detector);
        assert!(
            matches!(err, DetectorError::Record { .. }),
            "{bad:?}: {err:?}"
        );
    }
}

#[test]
fn min_bytes_is_required_and_range_checked() {
    assert_eq!(
        detector_error(&dup().replace("min_bytes = 40\n", "")),
        DetectorError::FieldRequired {
            engine: "duplicate-paragraph",
            field: "min_bytes",
        }
    );
    for bad in ["0", "-5", "100001"] {
        let err = detector_error(&dup().replace("min_bytes = 40", &format!("min_bytes = {bad}")));
        assert!(matches!(err, DetectorError::MinBytes(_)), "{bad}: {err:?}");
    }
}

#[test]
fn min_bytes_on_another_engine_is_refused() {
    let line = r#"
[[detector]]
name = "l"
engine = "line-regex"
pattern = 'x'
min_bytes = 3
files = ["**"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "m"

[[detector.fixture]]
path = "a"
input = 'x'
expect = "hit"

[[detector.fixture]]
path = "a"
input = 'y'
expect = "clean"
"#;
    assert_eq!(
        detector_error(line),
        DetectorError::FieldNotUsed {
            engine: "a write engine",
            field: "min_bytes",
        }
    );
    let root = r#"
[[detector]]
name = "r"
engine = "root-documents"
min_bytes = 3
files = ["*.md"]
events = ["repository-gate"]
action = "block"
message = "m"

[[detector.fixture]]
records = ["NOTES.md"]
expect = "hit"

[[detector.fixture]]
records = ["docs/a.md"]
expect = "clean"
"#;
    assert_eq!(
        detector_error(root),
        DetectorError::FieldNotUsed {
            engine: "root-documents",
            field: "min_bytes",
        }
    );
}

#[test]
fn a_duplicate_paragraph_gate_round_trips_through_the_serializer() {
    let rule = parse_document(&document(&dup())).expect("parses");
    let again = parse_document(&to_document(&rule)).expect("the serialized rule parses");
    assert_eq!(again.gates(), rule.gates());
}

// ---------------------------------------------------------------------------
// The emitted script.
// ---------------------------------------------------------------------------

#[test]
fn the_repo_target_emits_the_gate_and_its_records_reach_the_runner() {
    let files = repo::emit(&library(&dup()));
    let script = files
        .iter()
        .find(|f| f.path().as_str() == "gates/one-home.sh")
        .expect("the gate script")
        .contents();
    assert!(script.contains("export RL_MIN='40'"), "{script}");
    assert!(script.contains("Records: path, paragraph"), "{script}");
    let runner = files
        .iter()
        .find(|f| f.path().as_str() == "gates/test-gates.sh")
        .expect("the runner")
        .contents();
    assert!(
        runner.contains("tree_case 'one-home.sh' 'one-home' hit warn"),
        "{runner}"
    );
}
