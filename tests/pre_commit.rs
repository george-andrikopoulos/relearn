//! The `git-pre-commit` event and the `listed-path` engine (ARCHITECTURE.md,
//! "Proposed 2026-10-31: detector engines", decision 1, approved 2026-11-01).
//!
//! A commit-time detector reads the **index**: a staged path, and for
//! `line-regex` the staged content. `listed-path` hits a staged path that is on
//! a machine-local list, so its fixtures carry a synthetic `list` and the real
//! list never reaches a rule file. Both run at parse like every other fixture.
//! These tests pin what parses, what is refused, which emitter reads which
//! detector, and the self-exclusion of the generated commit dispatcher.

use relearn::emit::{hooks, repo};
use relearn::library::{Library, Validated};
use relearn::rule::{
    Detection, DetectorError, Event, ParseError, Rule, Subject, parse_document, to_document,
};

/// A complete rule document around the given detector tables.
fn doc(detectors: &str) -> String {
    format!(
        "+++\ntag = \"R:synthetic-commit\"\ntitle = \"t\"\nerror_class = \"e\"\n\
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

/// A listed-path detector over the whole tree, with synthetic list entries.
const LISTED: &str = r#"
[[detector]]
name = "listed-file-staged"
engine = "listed-path"
files = ["**"]
events = ["git-pre-commit"]
action = "block"
message = "This path is on the clone's list of files never to commit."

[[detector.fixture]]
path = "config/local-secrets.toml"
list = ["config/local-secrets.toml", "machine.env"]
expect = "hit"

[[detector.fixture]]
path = "config/shared.toml"
list = ["config/local-secrets.toml", "machine.env"]
expect = "clean"

[[detector.fixture]]
path = "nested/machine.env"
list = ["machine.env"]
expect = "clean"
"#;

/// A line-regex detector that runs at both events.
const BOTH: &str = r#"
[[detector]]
name = "no-marker-both"
engine = "line-regex"
pattern = 'FORBIDDEN_MARKER'
files = ["**/*.sh"]
events = ["claude-code-pre-tool-use", "git-pre-commit"]
action = "block"
message = "Remove the marker."

[[detector.fixture]]
path = "bin/run.sh"
input = 'echo FORBIDDEN_MARKER'
expect = "hit"

[[detector.fixture]]
path = "bin/run.sh"
input = 'echo fine'
expect = "clean"
"#;

// ---------------------------------------------------------------------------
// What parses.
// ---------------------------------------------------------------------------

#[test]
fn a_listed_path_detector_parses_and_runs_only_at_commit() {
    let r = rule(LISTED);
    let d = &r.detectors()[0];
    assert_eq!(d.engine().as_str(), "listed-path");
    assert!(d.runs_at(Event::GitPreCommit));
    assert!(!d.runs_at(Event::ClaudeCodePreToolUse));
}

#[test]
fn a_listed_path_hit_is_an_exact_path_on_the_list_never_a_suffix() {
    let r = rule(LISTED);
    let d = &r.detectors()[0];
    let list = vec!["machine.env".to_owned(), "config/a.toml".to_owned()];
    let at = |path: &str| d.evaluate(&Subject::new(path, "", &[]).with_listed(&list));
    assert_eq!(at("machine.env"), Detection::Hit);
    assert_eq!(at("config/a.toml"), Detection::Hit);
    assert_eq!(at("sub/machine.env"), Detection::Clean);
    assert_eq!(at("config/a.toml.bak"), Detection::Clean);
    assert_eq!(at("other.env"), Detection::Clean);
}

#[test]
fn a_listed_path_detector_with_no_list_never_hits() {
    // The self-exclusion sweep and a probe both evaluate without a list: the
    // machine-local list is not something relearn can see.
    let r = rule(LISTED);
    let d = &r.detectors()[0];
    assert_eq!(
        d.evaluate(&Subject::new("machine.env", "", &[])),
        Detection::Clean
    );
}

#[test]
fn a_line_regex_detector_may_run_at_both_events() {
    let r = rule(BOTH);
    let d = &r.detectors()[0];
    assert!(d.runs_at(Event::ClaudeCodePreToolUse));
    assert!(d.runs_at(Event::GitPreCommit));
}

#[test]
fn a_listed_path_rule_round_trips_through_the_serializer() {
    let r = rule(LISTED);
    let again = parse_document(&to_document(&r)).expect("the serialized rule parses");
    assert_eq!(again.detectors(), r.detectors());
}

// ---------------------------------------------------------------------------
// What is refused.
// ---------------------------------------------------------------------------

#[test]
fn listed_path_at_write_time_is_refused() {
    let err = detector_error(&LISTED.replace(
        "events = [\"git-pre-commit\"]",
        "events = [\"claude-code-pre-tool-use\", \"git-pre-commit\"]",
    ));
    assert_eq!(
        err,
        DetectorError::EventNotForEngine {
            engine: "listed-path",
            expected: "git-pre-commit only",
        }
    );
}

#[test]
fn sibling_case_at_commit_is_refused() {
    let err = detector_error(
        r#"
[[detector]]
name = "case"
engine = "sibling-case"
files = ["**"]
events = ["git-pre-commit"]
action = "block"
message = "m"

[[detector.fixture]]
path = "a/Readme.md"
siblings = ["README.md"]
expect = "hit"

[[detector.fixture]]
path = "a/x.md"
siblings = ["README.md"]
expect = "clean"
"#,
    );
    assert_eq!(
        err,
        DetectorError::EventNotForEngine {
            engine: "sibling-case",
            expected: "claude-code-pre-tool-use only",
        }
    );
}

#[test]
fn a_listed_path_fixture_needs_a_list_and_no_input() {
    let err = detector_error(&LISTED.replacen(
        "list = [\"config/local-secrets.toml\", \"machine.env\"]\nexpect = \"hit\"",
        "input = 'x'\nexpect = \"hit\"",
        1,
    ));
    assert_eq!(
        err,
        DetectorError::FixtureInputMismatch {
            index: 1,
            engine: "listed-path",
        }
    );
}

#[test]
fn a_list_entry_that_is_not_a_relative_path_is_refused() {
    for bad in ["../outside", "/etc/x", "a//b", "C:/x", "a\\\\b"] {
        let err = detector_error(&LISTED.replacen(
            "list = [\"machine.env\"]",
            &format!("list = [\"{bad}\"]"),
            1,
        ));
        assert!(
            matches!(err, DetectorError::ListEntry { index: 3, .. }),
            "{bad}: {err:?}"
        );
    }
}

#[test]
fn listed_path_reads_no_pattern() {
    let err = detector_error(&LISTED.replace(
        "engine = \"listed-path\"",
        "engine = \"listed-path\"\npattern = 'x'",
    ));
    assert_eq!(
        err,
        DetectorError::FieldNotUsed {
            engine: "listed-path",
            field: "pattern",
        }
    );
}

#[test]
fn a_listed_path_fixture_that_disagrees_stops_the_parse() {
    let err = detector_error(&format!(
        "{LISTED}\n[[detector.fixture]]\npath = \"machine.env\"\nlist = [\"machine.env\"]\nexpect = \"clean\"\n"
    ));
    assert!(
        matches!(err, DetectorError::FixtureDisagrees { index: 4, .. }),
        "{err:?}"
    );
}

// ---------------------------------------------------------------------------
// Which emitter reads which detector.
// ---------------------------------------------------------------------------

fn paths(files: &[relearn::emit::OutputFile]) -> Vec<String> {
    files.iter().map(|f| f.path().as_str().to_owned()).collect()
}

#[test]
fn a_commit_only_detector_gets_no_hook_script() {
    let lib = library(LISTED);
    assert!(
        hooks::emit(&lib).is_empty(),
        "{:?}",
        paths(&hooks::emit(&lib))
    );
}

#[test]
fn a_detector_at_both_events_gets_a_hook_script_and_a_dispatcher_entry() {
    let lib = library(BOTH);
    let hook_paths = paths(&hooks::emit(&lib));
    assert!(hook_paths.contains(&"hooks/no-marker-both.sh".to_owned()));
    let repo_files = repo::emit(&lib);
    let dispatcher = repo_files
        .iter()
        .find(|f| f.path().as_str() == "gates/relearn-pre-commit.sh")
        .expect("a dispatcher for a commit-time detector");
    assert!(dispatcher.contents().contains("no-marker-both"));
    let runner = repo_files
        .iter()
        .find(|f| f.path().as_str() == "gates/test-gates.sh")
        .expect("a runner");
    assert!(runner.contents().contains("commit_case 'no-marker-both'"));
}

#[test]
fn a_write_only_detector_never_reaches_the_dispatcher() {
    let only_write = BOTH.replace(
        "events = [\"claude-code-pre-tool-use\", \"git-pre-commit\"]",
        "events = [\"claude-code-pre-tool-use\"]",
    );
    let lib = library(&only_write);
    assert!(repo::emit(&lib).is_empty());
}

#[test]
fn the_runner_checks_that_a_listed_path_detector_fails_closed() {
    let lib = library(LISTED);
    let runner = repo::emit(&lib)
        .into_iter()
        .find(|f| f.path().as_str() == "gates/test-gates.sh")
        .expect("a runner");
    // No list configured, and a list holding a line that is not a path: both
    // must come back as "could not check", never as a pass.
    assert!(
        runner
            .contents()
            .contains("commit_case 'listed-file-staged' disarmed block")
    );
    assert!(runner.contents().contains("'../outside'"));
}

// ---------------------------------------------------------------------------
// Self-exclusion: the dispatcher quotes every commit-time pattern.
// ---------------------------------------------------------------------------

#[test]
fn the_dispatcher_is_skipped_for_its_own_detectors_and_swept_for_the_rest() {
    let lib = library(BOTH);
    let files = repo::emit_with_roles(&lib);
    let swept: Vec<hooks::Swept<'_>> = files
        .iter()
        .map(|(f, r)| hooks::Swept::hook(f, r))
        .collect();
    assert!(
        hooks::self_matches(&lib, &swept).is_empty(),
        "a commit-time detector fired on the dispatcher that quotes it"
    );

    // A write-only detector whose pattern the dispatcher happens to contain is
    // a real finding: the dispatcher is not its definition.
    let other = r#"
[[detector]]
name = "write-only-marker"
engine = "line-regex"
pattern = 'FORBIDDEN_MARKER'
files = ["**/*.sh"]
events = ["claude-code-pre-tool-use"]
action = "warn"
message = "m"

[[detector.fixture]]
path = "x.sh"
input = 'FORBIDDEN_MARKER'
expect = "hit"

[[detector.fixture]]
path = "x.sh"
input = 'fine'
expect = "clean"
"#;
    let lib = library(&format!("{BOTH}{other}"));
    let files = repo::emit_with_roles(&lib);
    let swept: Vec<hooks::Swept<'_>> = files
        .iter()
        .map(|(f, r)| hooks::Swept::hook(f, r))
        .collect();
    let found: Vec<(String, String)> = hooks::self_matches(&lib, &swept)
        .iter()
        .map(|m| (m.detector().to_owned(), m.file().to_owned()))
        .collect();
    assert_eq!(
        found,
        vec![(
            "write-only-marker".to_owned(),
            "gates/relearn-pre-commit.sh".to_owned()
        )]
    );
}

// ---------------------------------------------------------------------------
// Controls and contribution.
// ---------------------------------------------------------------------------

#[test]
fn a_commit_only_detector_is_not_mapped_as_a_hook() {
    use relearn::resolve::ControlMap;
    let lib = library(LISTED);
    // A hand entry for `hook:<name>` is refused only when a hook script for
    // that name exists; a commit-only detector emits none.
    let map = ControlMap::parse(
        "here = \"x\"\n[[control]]\ncontrol = \"hook:listed-file-staged\"\nroot = \"y\"\npath = \"h.sh\"\n",
    )
    .expect("the map parses");
    assert!(map.with_detectors(&lib).is_ok());
}

#[test]
fn a_contributed_list_is_offered_to_the_banned_terms_scan() {
    use relearn::contribute::Contribution;
    let d = doc(LISTED).replace(
        "incident = \"synthetic\"",
        "incident = \"synthetic\"\npublished_incident = \"A synthetic published account.\"",
    );
    let r = parse_document(&d).expect("parses");
    let contribution = Contribution::of(&r).expect("contributable");
    let fields: Vec<(&str, &str)> = contribution.authored_texts().collect();
    assert!(
        fields.contains(&("detector.fixture.list", "machine.env")),
        "{fields:?}"
    );
}

// ---------------------------------------------------------------------------
// The committed pack.
// ---------------------------------------------------------------------------

/// `repo-pack/README.md`'s commit-time table lists every detector the
/// dispatcher runs, its rule and its install state.
#[test]
fn the_repo_pack_readme_lists_every_commit_time_detector() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("repo-pack/README.md")).expect("a README");
    let section = readme
        .split("## Commit-time detectors")
        .nth(1)
        .expect("a commit-time section");
    let mut listed: Vec<(String, String)> = section
        .lines()
        .take_while(|l| !l.starts_with("## "))
        .filter_map(|line| {
            let cells: Vec<&str> = line.split('|').map(str::trim).collect();
            let name = cells.get(1)?.strip_prefix('`')?.strip_suffix('`')?;
            let tag = cells.get(2)?.strip_prefix("`[")?.strip_suffix("]`")?;
            Some((name.to_owned(), tag.to_owned()))
        })
        .collect();
    listed.sort();

    let library = relearn::fsio::load_rules(&root.join("rules"))
        .expect("the corpus loads")
        .validate()
        .expect("the corpus validates");
    let mut expected: Vec<(String, String)> = library
        .rules()
        .iter()
        .flat_map(|r| {
            r.detectors()
                .iter()
                .filter(|d| d.runs_at(Event::GitPreCommit))
                .map(move |d| (d.name().as_str().to_owned(), r.tag().as_str().to_owned()))
        })
        .collect();
    expected.sort();
    assert!(
        !expected.is_empty(),
        "the corpus has no commit-time detector"
    );
    assert_eq!(listed, expected);
}
