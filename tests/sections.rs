//! The `sections` engine and the `claude-code-post-tool-use` event
//! (ARCHITECTURE.md, "Proposed 2026-10-31: detector engines", decision 2,
//! approved 2026-11-01 with PostToolUse warn-only).
//!
//! A `sections` detector reads a **whole file**: every section opened by a
//! heading of its level must contain a line matching `require`. It runs at
//! `PreToolUse` on `Write` (the whole content), at `PostToolUse` on `Edit` and
//! `MultiEdit` (the file read back from disk, warn-shaped, because the edit has
//! already happened), and at `git-pre-commit` (the staged content).

use proptest::prelude::*;
use relearn::emit::{hooks, repo};
use relearn::library::{Library, Validated};
use relearn::rule::{
    Detection, DetectorError, Event, ParseError, Rule, Subject, parse_document, to_document,
};

fn doc(detectors: &str) -> String {
    format!(
        "+++\ntag = \"R:synthetic-sections\"\ntitle = \"t\"\nerror_class = \"e\"\n\
         home = {{ kind = \"global\" }}\ncreated = \"2026-11-01\"\norigin = \"mined\"\n\
         status = {{ kind = \"active\" }}\nincident = \"synthetic\"\n{detectors}+++\n\nBody.\n"
    )
}

fn rule(detectors: &str) -> Rule {
    parse_document(&doc(detectors)).expect("the synthetic rule parses")
}

fn detector_error(detectors: &str) -> DetectorError {
    match parse_document(&doc(detectors)) {
        Err(ParseError::Detector { source, .. }) => source,
        other => panic!("expected a detector error, got {other:?}"),
    }
}

fn library(detectors: &str) -> Library<Validated> {
    let mut library = Library::new();
    library.push(rule(detectors));
    library.validate().expect("validates")
}

const LEDGER: &str = r#"
[[detector]]
name = "entry-names-artefact"
engine = "sections"
heading = 3
require = 'Enforced by|NOTHING YET'
files = ["**/FEATURES.md"]
events = ["claude-code-pre-tool-use", "claude-code-post-tool-use", "git-pre-commit"]
action = "block"
message = "A ledger entry names no enforcing artefact."

[[detector.fixture]]
path = "FEATURES.md"
input = '''
## Group

### Has one
What: x.
**Enforced by:** a test.

### Lacks one
What: y.
'''
expect = "hit"

[[detector.fixture]]
path = "FEATURES.md"
input = '''
## Group

### Has one
**Enforced by:** a test.

### Exposed
NOTHING YET, exposed.
'''
expect = "clean"
"#;

fn ledger() -> Rule {
    rule(LEDGER)
}

fn verdict(text: &str) -> Detection {
    let r = ledger();
    r.detectors()[0].evaluate(&Subject::new("FEATURES.md", text, &[]))
}

// ---------------------------------------------------------------------------
// The judgement.
// ---------------------------------------------------------------------------

#[test]
fn a_section_without_the_required_line_is_a_hit() {
    assert_eq!(
        verdict("### A\nEnforced by: t\n### B\nnothing\n"),
        Detection::Hit
    );
    assert_eq!(
        verdict("### A\nEnforced by: t\n### B\nNOTHING YET\n"),
        Detection::Clean
    );
}

#[test]
fn a_file_with_no_section_of_the_level_is_clean() {
    assert_eq!(verdict("# Title\n\n## Group\ntext\n"), Detection::Clean);
    assert_eq!(verdict(""), Detection::Clean);
}

#[test]
fn the_heading_line_itself_does_not_satisfy_the_section() {
    assert_eq!(verdict("### Enforced by nobody\ntext\n"), Detection::Hit);
}

#[test]
fn a_deeper_heading_stays_inside_the_section_and_a_shallower_one_closes_it() {
    // The required line under a level-4 subsection still counts for its parent.
    assert_eq!(
        verdict("### A\n#### detail\nEnforced by: t\n"),
        Detection::Clean
    );
    // A level-2 heading closes the section: the line after it is not in A.
    assert_eq!(
        verdict("### A\ntext\n## Next group\nEnforced by: t\n"),
        Detection::Hit
    );
}

#[test]
fn a_heading_inside_a_fenced_block_opens_no_section() {
    assert_eq!(
        verdict("### A\nEnforced by: t\n```\n### not a heading\n```\n"),
        Detection::Clean
    );
    assert_eq!(
        verdict("### A\nEnforced by: t\n~~~\n### not a heading\n~~~\n"),
        Detection::Clean
    );
}

#[test]
fn a_hash_without_a_space_is_not_a_heading() {
    assert_eq!(verdict("### A\nEnforced by: t\n###B\n"), Detection::Clean);
}

#[test]
fn a_crlf_file_is_judged_like_its_lf_twin() {
    let lf = "### A\nEnforced by: t\n### B\ntext\n";
    assert_eq!(verdict(&lf.replace('\n', "\r\n")), verdict(lf));
}

proptest! {
    /// The verdict is the per-section law: a hit exactly when some section
    /// lacks the required line, whatever else the sections hold.
    #[test]
    fn a_hit_is_exactly_a_section_missing_its_line(
        sections in prop::collection::vec((any::<bool>(), 0usize..3), 0..8)
    ) {
        let mut text = String::from("# Ledger\n\n");
        for (i, (has, filler)) in sections.iter().enumerate() {
            text.push_str(&format!("### Entry {i}\n"));
            for _ in 0..*filler {
                text.push_str("What: a line.\n");
            }
            if *has {
                text.push_str("**Enforced by:** a test.\n");
            }
        }
        let expected = if sections.iter().any(|(has, _)| !has) {
            Detection::Hit
        } else {
            Detection::Clean
        };
        prop_assert_eq!(verdict(&text), expected);
    }
}

// ---------------------------------------------------------------------------
// What parses and what is refused.
// ---------------------------------------------------------------------------

#[test]
fn a_sections_detector_runs_at_all_three_events() {
    let r = ledger();
    let d = &r.detectors()[0];
    assert!(d.runs_at(Event::ClaudeCodePreToolUse));
    assert!(d.runs_at(Event::ClaudeCodePostToolUse));
    assert!(d.runs_at(Event::GitPreCommit));
}

#[test]
fn a_heading_level_outside_one_to_six_is_refused() {
    for bad in ["0", "7", "-1", "300"] {
        let err = detector_error(&LEDGER.replace("heading = 3", &format!("heading = {bad}")));
        assert!(
            matches!(err, DetectorError::HeadingLevel(_)),
            "{bad}: {err:?}"
        );
    }
}

#[test]
fn sections_needs_a_heading_and_a_required_pattern() {
    assert_eq!(
        detector_error(&LEDGER.replace("heading = 3\n", "")),
        DetectorError::FieldRequired {
            engine: "sections",
            field: "heading",
        }
    );
    assert_eq!(
        detector_error(&LEDGER.replace("require = 'Enforced by|NOTHING YET'\n", "")),
        DetectorError::FieldRequired {
            engine: "sections",
            field: "require",
        }
    );
}

#[test]
fn sections_reads_no_forbidden_pattern_and_line_regex_reads_no_section_fields() {
    assert_eq!(
        detector_error(&LEDGER.replace("heading = 3", "heading = 3\npattern = 'x'")),
        DetectorError::FieldNotUsed {
            engine: "sections",
            field: "pattern",
        }
    );
    let line = r#"
[[detector]]
name = "l"
engine = "line-regex"
pattern = 'x'
heading = 2
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
            engine: "line-regex",
            field: "heading",
        }
    );
}

#[test]
fn a_required_pattern_outside_the_shared_dialect_is_refused() {
    let err = detector_error(&LEDGER.replace("'Enforced by|NOTHING YET'", r"'\d'"));
    assert!(matches!(err, DetectorError::Pattern { .. }), "{err:?}");
}

#[test]
fn post_tool_use_is_only_for_a_whole_file_engine() {
    let line = r#"
[[detector]]
name = "l"
engine = "line-regex"
pattern = 'x'
files = ["**"]
events = ["claude-code-post-tool-use"]
action = "warn"
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
        DetectorError::EventNotForEngine {
            engine: "line-regex",
            expected: "claude-code-pre-tool-use or git-pre-commit",
        }
    );
}

#[test]
fn a_sections_rule_round_trips_through_the_serializer() {
    let r = ledger();
    let again = parse_document(&to_document(&r)).expect("the serialized rule parses");
    assert_eq!(again.detectors(), r.detectors());
}

// ---------------------------------------------------------------------------
// What each emitter writes.
// ---------------------------------------------------------------------------

fn file<'a>(files: &'a [relearn::emit::OutputFile], path: &str) -> &'a str {
    files
        .iter()
        .find(|f| f.path().as_str() == path)
        .unwrap_or_else(|| panic!("no {path}"))
        .contents()
}

#[test]
fn one_hook_script_serves_write_before_and_edit_after() {
    let lib = library(LEDGER);
    let files = hooks::emit(&lib);
    let script = file(&files, "hooks/entry-names-artefact.sh");
    assert!(script.contains("PreToolUse"), "reads the Write payload");
    assert!(script.contains("PostToolUse"), "reads the edited file back");
    let snippet = file(&files, "hooks/settings-snippet.md");
    assert!(snippet.contains("\"PostToolUse\""), "{snippet}");
    let runner = file(&files, "hooks/test-detectors.sh");
    assert!(
        runner.contains("run_post 'entry-names-artefact.sh'"),
        "{runner}"
    );
}

#[test]
fn a_write_only_sections_detector_registers_no_post_tool_use_hook() {
    let only_write = LEDGER.replace(
        "events = [\"claude-code-pre-tool-use\", \"claude-code-post-tool-use\", \"git-pre-commit\"]",
        "events = [\"claude-code-pre-tool-use\"]",
    );
    let lib = library(&only_write);
    let files = hooks::emit(&lib);
    let snippet = file(&files, "hooks/settings-snippet.md");
    assert!(!snippet.contains("PostToolUse"), "{snippet}");
    assert!(repo::emit(&lib).is_empty());
}

/// A detector that runs only after an edit still compiles to a hook script,
/// so it is a hook to the controls map and to the README's table: one
/// predicate decides that, and the hooks emitter agrees with it.
#[test]
fn an_after_edit_only_detector_is_still_a_hook() {
    use relearn::resolve::ControlMap;
    let after_only = LEDGER.replace(
        "events = [\"claude-code-pre-tool-use\", \"claude-code-post-tool-use\", \"git-pre-commit\"]",
        "events = [\"claude-code-post-tool-use\"]",
    );
    let lib = library(&after_only);
    let d = &lib.rules()[0].detectors()[0];
    assert!(d.has_hook_script());
    let files = hooks::emit(&lib);
    let snippet = file(&files, "hooks/settings-snippet.md");
    assert!(!snippet.contains("\"PreToolUse\""), "{snippet}");
    assert!(snippet.contains("\"PostToolUse\""), "{snippet}");
    let runner = file(&files, "hooks/test-detectors.sh");
    assert!(
        !runner.contains("\nrun 'entry-names-artefact.sh'"),
        "{runner}"
    );
    let map = ControlMap::parse(
        "here = \"x\"\n[[control]]\ncontrol = \"hook:entry-names-artefact\"\nroot = \"y\"\npath = \"h.sh\"\n",
    )
    .expect("the map parses");
    assert!(
        map.with_detectors(&lib).is_err(),
        "the detector is the hook's home"
    );
    let commit_only = LEDGER.replace(
        "events = [\"claude-code-pre-tool-use\", \"claude-code-post-tool-use\", \"git-pre-commit\"]",
        "events = [\"git-pre-commit\"]",
    );
    let lib = library(&commit_only);
    assert!(!lib.rules()[0].detectors()[0].has_hook_script());
    assert!(hooks::emit(&lib).is_empty());
}

#[test]
fn the_dispatcher_carries_a_commit_time_sections_detector() {
    let lib = library(LEDGER);
    let files = repo::emit(&lib);
    let dispatcher = file(&files, "gates/relearn-pre-commit.sh");
    assert!(dispatcher.contains("d_entry_names_artefact()"));
    let runner = file(&files, "gates/test-gates.sh");
    assert!(runner.contains("commit_case 'entry-names-artefact' hit block"));
}
