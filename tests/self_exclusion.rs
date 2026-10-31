//! `check`'s self-exclusion sweep skips a detector's own generated script and
//! the fixture runner **by identity**, not by a path glob.
//!
//! Both files must contain what the detector looks for: its script quotes the
//! pattern, and the runner pipes every hit fixture through it. A glob such as
//! `**/hooks/*.sh` would skip them too, but it would also stop the detector
//! reading a real repository's `hooks/*.sh` at runtime, which was refused on
//! 2026-10-30. The sweep instead asks the hooks emitter which file it made.

use relearn::emit::OutputFile;
use relearn::emit::hooks::{HookRole, Swept, emit_with_roles, self_matches};
use relearn::library::{Library, Validated};
use relearn::rule::{Detection, Subject, parse_document};

/// A rule whose detector reads every path and whose pattern is a token the
/// generated script must quote: the hazard, in its plainest form.
fn library() -> Library<Validated> {
    let document = r#"+++
tag = "R:synthetic-self"
title = "t"
error_class = "e"
home = { kind = "global" }
created = "2026-10-31"
origin = "mined"
status = { kind = "active" }
incident = "synthetic"

[[detector]]
name = "no-marker-token"
engine = "line-regex"
pattern = 'MARKER-TOKEN'
files = ["**"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "Remove the marker."

[[detector.fixture]]
path = "notes.txt"
input = 'a MARKER-TOKEN line'
expect = "hit"

[[detector.fixture]]
path = "notes.txt"
input = 'nothing here'
expect = "clean"
+++

Body.
"#;
    let rule = parse_document(document).expect("the synthetic rule parses");
    let mut library = Library::new();
    library.push(rule);
    library.validate().expect("one rule validates")
}

fn file_for<'a>(files: &'a [(OutputFile, HookRole)], path: &str) -> &'a OutputFile {
    &files
        .iter()
        .find(|(f, _)| f.path().as_str() == path)
        .unwrap_or_else(|| panic!("the hooks target emits {path}"))
        .0
}

#[test]
fn the_generated_files_really_carry_the_pattern_and_its_hit_fixture() {
    // Without this the next test could pass because nothing was there to find.
    let library = library();
    let files = emit_with_roles(&library);
    let script = file_for(&files, "hooks/no-marker-token.sh");
    let runner = file_for(&files, "hooks/test-detectors.sh");
    let detector = &library.rules()[0].detectors()[0];
    for file in [script, runner] {
        assert_eq!(
            detector.evaluate(&Subject::new(file.path().as_str(), file.contents(), &[])),
            Detection::Hit,
            "{} must contain a hit, or the skip below proves nothing",
            file.path().as_str()
        );
    }
}

#[test]
fn a_detector_does_not_fire_on_its_own_script_or_the_runner() {
    let library = library();
    let files = emit_with_roles(&library);
    let swept: Vec<Swept<'_>> = files.iter().map(|(f, r)| Swept::hook(f, r)).collect();
    let matches = self_matches(&library, &swept);
    assert!(matches.is_empty(), "{matches:?}");
}

#[test]
fn the_same_text_at_the_same_path_is_still_swept_when_the_hooks_target_did_not_make_it() {
    // Identity, not path: an emitted file the hooks target did not write is
    // read like any other, even at a path where a hook script would live.
    let library = library();
    let files = emit_with_roles(&library);
    let script = file_for(&files, "hooks/no-marker-token.sh");
    let swept = [Swept::emitted(script)];
    let matches = self_matches(&library, &swept);
    assert_eq!(matches.len(), 1, "{matches:?}");
    assert_eq!(matches[0].file(), "hooks/no-marker-token.sh");
}

#[test]
fn a_real_repositorys_hook_scripts_stay_in_scope_at_runtime() {
    // The runtime scope is unchanged: only the pack and the install directory
    // are excluded, so a repository's own hooks are read.
    let library = library();
    let detector = &library.rules()[0].detectors()[0];
    for path in [
        "/repo/hooks/no-marker-token.sh",
        "/repo/hooks/test-detectors.sh",
        "/repo/.githooks/pre-commit",
    ] {
        assert_eq!(
            detector.evaluate(&Subject::new(path, "MARKER-TOKEN", &[])),
            Detection::Hit,
            "{path}"
        );
    }
}

#[test]
fn another_detectors_script_is_not_skipped() {
    // Skipping is per detector: the script of a different detector is read.
    let library = library();
    let files = emit_with_roles(&library);
    let script = file_for(&files, "hooks/no-marker-token.sh");
    let other = HookRole::script_of("some-other-detector").expect("a valid name");
    let swept = [Swept::hook(script, &other)];
    assert_eq!(self_matches(&library, &swept).len(), 1);
}
