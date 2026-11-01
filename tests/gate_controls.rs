//! How a rule names a commit-time detector or a repository gate as its
//! control: `gate:<name>`, the existing `Gate` kind (George, 2026-11-01).
//!
//! A hook installs to one fixed path, so its `hook:` entry is generated from
//! the detector. A gate, or the commit dispatcher, is copied into another
//! repository's gate, and only that repository knows where; so its `gate:`
//! entry is **authored** in `controls.toml` (root, script, invoker), exactly
//! like every hand-written gate already mapped there. What this file pins is
//! the other half: `lint` proposes `gate:<name>` for graduation, as it
//! proposes `hook:<name>`, and a detector at both events is proposed as both.

use relearn::library::{Library, Validated};
use relearn::lint::{self, Finding};
use relearn::resolve::ControlMap;
use relearn::rule::parse_document;

fn rule(status: &str, detectors: &str) -> String {
    format!(
        "+++\ntag = \"R:synthetic-gate-control\"\ntitle = \"t\"\nerror_class = \"e\"\n\
         home = {{ kind = \"global\" }}\ncreated = \"2026-11-01\"\norigin = \"mined\"\n\
         status = {status}\nincident = \"synthetic\"\n{detectors}+++\n\nBody.\n"
    )
}

fn library(doc: &str) -> Library<Validated> {
    let mut library = Library::new();
    library.push(parse_document(doc).expect("the synthetic rule parses"));
    library.validate().expect("validates")
}

const ACTIVE: &str = "{ kind = \"active\" }";

/// A line-regex detector at the given events.
fn line_regex(events: &str) -> String {
    format!(
        r#"
[[detector]]
name = "no-marker"
engine = "line-regex"
pattern = 'MARKER'
files = ["**"]
events = [{events}]
action = "block"
message = "m"

[[detector.fixture]]
path = "a.txt"
input = 'MARKER'
expect = "hit"

[[detector.fixture]]
path = "a.txt"
input = 'clean'
expect = "clean"
"#
    )
}

const ROOT_GATE: &str = r#"
[[detector]]
name = "root-docs"
engine = "root-documents"
files = ["*.md"]
exclude = ["README.md"]
events = ["repository-gate"]
action = "warn"
message = "m"

[[detector.fixture]]
records = ["NOTES.md"]
expect = "hit"

[[detector.fixture]]
records = ["README.md"]
expect = "clean"
"#;

/// The controls each graduation candidate proposes, as its message prints them.
fn proposed(lib: &Library<Validated>) -> Vec<String> {
    lint::lint(lib)
        .into_iter()
        .filter_map(|f| match f {
            Finding::GraduationCandidate { controls, .. } => Some(controls),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn a_commit_only_detector_is_proposed_as_a_gate() {
    let lib = library(&rule(ACTIVE, &line_regex("\"git-pre-commit\"")));
    assert_eq!(proposed(&lib), vec!["gate:no-marker".to_owned()]);
}

#[test]
fn a_repository_gate_is_proposed_as_a_gate() {
    let lib = library(&rule(ACTIVE, ROOT_GATE));
    assert_eq!(proposed(&lib), vec!["gate:root-docs".to_owned()]);
}

#[test]
fn a_detector_at_both_events_is_proposed_as_both() {
    let lib = library(&rule(
        ACTIVE,
        &line_regex("\"claude-code-pre-tool-use\", \"git-pre-commit\""),
    ));
    assert_eq!(
        proposed(&lib),
        vec!["hook:no-marker".to_owned(), "gate:no-marker".to_owned()]
    );
}

#[test]
fn a_gate_the_status_names_is_not_proposed_again() {
    let partial = "{ kind = \"partial\", by = \"gate:no-marker\", uncovered = \"the write-time half\", date = \"2026-11-01\" }";
    let lib = library(&rule(
        partial,
        &line_regex("\"claude-code-pre-tool-use\", \"git-pre-commit\""),
    ));
    assert_eq!(proposed(&lib), vec!["hook:no-marker".to_owned()]);
    let graduated = "{ kind = \"graduated\", to = \"gate:root-docs\", date = \"2026-11-01\" }";
    assert!(proposed(&library(&rule(graduated, ROOT_GATE))).is_empty());
}

/// The authored half: a `gate:` entry placing a commit-time detector in the
/// repository that installed the dispatcher is accepted beside the generated
/// `hook:` entries, because no detector generates a `gate:` entry to collide
/// with it.
#[test]
fn an_authored_gate_entry_for_a_commit_time_detector_is_accepted() {
    let lib = library(&rule(
        ACTIVE,
        &line_regex("\"claude-code-pre-tool-use\", \"git-pre-commit\""),
    ));
    let map = ControlMap::parse(
        "here = \"relearn\"\n[[control]]\ncontrol = \"gate:no-marker\"\nroot = \"consumer\"\n\
         path = \".githooks/relearn-pre-commit.sh\"\nfunction = \"d_no_marker\"\n\
         invoked_by = \".githooks/relearn-pre-commit.sh\"\n",
    )
    .expect("the map parses");
    assert!(map.with_detectors(&lib).is_ok());
}
