//! The `hooks` target and the two lint findings detectors brought.
//!
//! The scripts' *behaviour* is checked on the production path by the pack's own
//! `test-detectors.sh`, which `scripts/check.sh` runs; these tests hold what
//! Rust can see: what is emitted, that a script's header round-trips through
//! the clobber guard and `verify`, and when the findings fire.

use std::fs;

use relearn::emit::{self, HeaderStyle};
use relearn::fsio::{self, VerifyStatus};
use relearn::library::{Library, Validated};
use relearn::lint::{self, Finding};
use relearn::resolve::ControlMap;
use relearn::rule::parse_document;

const DETECTOR: &str = r#"
[[detector]]
name = "no-unwrap"
engine = "line-regex"
pattern = '\.unwrap\('
files = ["src/**/*.rs"]
events = ["claude-code-pre-tool-use"]
action = "block"
message = "Propagate with ?."

[[detector.fixture]]
path = "src/lib.rs"
input = 'f().unwrap()'
expect = "hit"

[[detector.fixture]]
path = "src/lib.rs"
input = 'f()?'
expect = "clean"
"#;

/// A rule document with the given status and detector tables.
fn rule(tag: &str, status: &str, detectors: &str) -> String {
    format!(
        "+++\ntag = \"{tag}\"\ntitle = \"t\"\nerror_class = \"e {tag}\"\n\
         home = {{ kind = \"global\" }}\ncreated = \"2026-10-30\"\norigin = \"mined\"\n\
         status = {status}\nincident = \"synthetic\"\n{detectors}+++\n\nBody.\n"
    )
}

fn library(docs: &[String]) -> Library<Validated> {
    let mut lib = Library::new();
    for doc in docs {
        lib.push(parse_document(doc).expect("the fixture rule parses"));
    }
    lib.validate().expect("the fixture library validates")
}

const ACTIVE: &str = "{ kind = \"active\" }";
const GRADUATED: &str = "{ kind = \"graduated\", to = \"hook:no-unwrap\", date = \"2026-10-30\" }";

#[test]
fn a_library_without_detectors_emits_no_hook_pack() {
    let lib = library(&[rule("R:a", ACTIVE, "")]);
    assert!(emit::hooks::emit(&lib).is_empty());
}

#[test]
fn the_pack_holds_one_script_per_detector_a_runner_and_a_snippet() {
    let lib = library(&[rule("R:a", GRADUATED, DETECTOR)]);
    let files = emit::hooks::emit(&lib);
    let paths: Vec<&str> = files.iter().map(|f| f.path().as_str()).collect();
    assert_eq!(
        paths,
        [
            "hooks/no-unwrap.sh",
            "hooks/test-detectors.sh",
            "hooks/settings-snippet.md"
        ]
    );
    assert_eq!(files[0].header(), HeaderStyle::Shell);
    assert_eq!(files[1].header(), HeaderStyle::Shell);
    assert_eq!(files[2].header(), HeaderStyle::Markup);
    let script = files[0].contents();
    assert!(script.starts_with("#!/usr/bin/env bash\n"), "{script}");
    assert!(script.contains("pattern='\\.unwrap\\('"), "{script}");
    assert!(script.contains("exit 2"), "a block action exits 2");
    assert!(
        !script.contains("additionalContext"),
        "a block action never writes the warn channel"
    );
}

#[test]
fn a_warn_action_allows_and_speaks_on_the_documented_channel() {
    let lib = library(&[rule(
        "R:a",
        GRADUATED,
        &DETECTOR.replace("action = \"block\"", "action = \"warn\""),
    )]);
    let script = emit::hooks::emit(&lib)[0].contents().to_owned();
    assert!(script.contains("additionalContext"), "{script}");
    assert!(
        !script.contains("exit 2"),
        "a warn must never block, on any path: {script}"
    );
}

/// No failure path of a generated script may exit 2: exit 2 is a block, and a
/// script that cannot run must be loud, not a refusal of the tool call.
#[test]
fn a_generated_script_fails_with_exit_1_never_2() {
    let lib = library(&[rule("R:a", GRADUATED, DETECTOR)]);
    let script = emit::hooks::emit(&lib)[0].contents().to_owned();
    let fail_fn = script
        .split("fail() {")
        .nth(1)
        .and_then(|rest| rest.split('}').next())
        .expect("the script defines fail()");
    assert!(fail_fn.contains("exit 1"), "{fail_fn}");
    assert_eq!(
        script.matches("exit 2").count(),
        1,
        "exactly one exit 2, the verdict: {script}"
    );
}

#[test]
fn a_written_script_verifies_clean_and_a_hand_edit_is_caught() {
    let dir = tempfile::tempdir().expect("tempdir");
    let lib = library(&[rule("R:a", GRADUATED, DETECTOR)]);
    let files = emit::hooks::emit(&lib);
    fsio::write_all(dir.path(), &files).expect("write succeeds");

    let reports = fsio::verify_all(dir.path(), &files).expect("verify reads succeed");
    assert!(
        reports.iter().all(|r| r.status().is_clean()),
        "a freshly written pack verifies clean"
    );

    let script = dir.path().join("hooks/no-unwrap.sh");
    let on_disk = fs::read_to_string(&script).expect("read back");
    let last = on_disk.lines().last().expect("a header line");
    assert!(
        last.starts_with("# <!-- relearn:generated"),
        "the header is a shell comment: {last}"
    );
    fs::write(&script, on_disk.replace("exit 2", "exit 0")).expect("tamper");
    let reports = fsio::verify_all(dir.path(), &files).expect("verify reads succeed");
    assert_eq!(*reports[0].status(), VerifyStatus::HandEdited);

    // And the clobber guard still recognises the script as relearn's to replace.
    fsio::write_all(dir.path(), &files).expect("a marked script is regenerated");
}

#[test]
fn a_prose_held_rule_with_an_unnamed_detector_is_a_graduation_candidate() {
    let lib = library(&[rule("R:a", ACTIVE, DETECTOR)]);
    let candidates: Vec<Finding> = lint::lint(&lib)
        .into_iter()
        .filter(|f| matches!(f, Finding::GraduationCandidate { .. }))
        .collect();
    assert_eq!(candidates.len(), 1, "{candidates:?}");
    assert!(
        candidates[0].to_string().contains("hook:no-unwrap"),
        "{}",
        candidates[0]
    );

    let partial = "{ kind = \"partial\", by = \"hook:other\", uncovered = \"the rest\", date = \"2026-10-30\" }";
    let lib = library(&[rule("R:a", partial, DETECTOR)]);
    assert!(
        lint::lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::GraduationCandidate { .. })),
        "a partial rule whose `by` does not list the detector has news too"
    );
}

#[test]
fn a_rule_already_naming_its_detector_is_not_a_candidate() {
    let lib = library(&[rule("R:a", GRADUATED, DETECTOR)]);
    assert!(
        !lint::lint(&lib)
            .iter()
            .any(|f| matches!(f, Finding::GraduationCandidate { .. }))
    );
}

#[test]
fn a_named_hook_nothing_places_is_reported_and_a_detector_or_map_entry_places_it() {
    let elsewhere = "{ kind = \"graduated\", to = \"hook:elsewhere\", date = \"2026-10-30\" }";
    let lib = library(&[rule("R:a", elsewhere, "")]);
    let unplaced = lint::unplaced_hooks(&lib, None);
    assert_eq!(unplaced.len(), 1, "{unplaced:?}");
    assert!(unplaced[0].to_string().contains("hook:elsewhere"));

    let map = ControlMap::parse(
        "here = \"x\"\n[[control]]\ncontrol = \"hook:elsewhere\"\nroot = \"y\"\npath = \"h.sh\"\n",
    )
    .expect("the map parses");
    assert!(lint::unplaced_hooks(&lib, Some(&map)).is_empty());

    let defined = library(&[rule("R:b", GRADUATED, DETECTOR)]);
    assert!(
        lint::unplaced_hooks(&defined, None).is_empty(),
        "a hook a detector defines is placed by the corpus itself"
    );
}

/// `[R:detector-excludes-own-definitions]`, through the binary: a detector
/// scoped to everything fires on the instruction layers that quote its rule,
/// and `check` refuses the corpus, naming detector and file. The implicit
/// excludes keep it off the rule file itself; nothing keeps it off a skill.
#[test]
fn check_refuses_a_detector_that_fires_on_the_layers_it_is_emitted_into() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    fs::create_dir(&rules).expect("rules dir");
    let wide = DETECTOR.replace("files = [\"src/**/*.rs\"]", "files = [\"**\"]");
    let doc = rule("R:a", GRADUATED, &wide).replace("Body.", "Never call .unwrap( in a library.");
    fs::write(rules.join("a.md"), doc).expect("write the rule");

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_relearn"))
        .args(["check", "--rules"])
        .arg(&rules)
        .output()
        .expect("relearn runs");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(!out.status.success(), "{stdout}{stderr}");
    assert!(
        stdout.contains("self-match: detector `no-unwrap` fires on skills/global/SKILL.md"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("fires on rules/"),
        "the implicit exclude keeps it off the rule file: {stdout}"
    );

    // The same corpus scoped to source files is clean.
    let narrow =
        rule("R:a", GRADUATED, DETECTOR).replace("Body.", "Never call .unwrap( in a library.");
    fs::write(rules.join("a.md"), narrow).expect("rewrite the rule");
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_relearn"))
        .args(["check", "--rules"])
        .arg(&rules)
        .output()
        .expect("relearn runs");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn a_detector_and_a_map_entry_for_one_hook_is_two_homes_and_refused() {
    let lib = library(&[rule("R:a", GRADUATED, DETECTOR)]);
    let map = ControlMap::parse(
        "here = \"x\"\n[[control]]\ncontrol = \"hook:no-unwrap\"\nroot = \"y\"\npath = \"h.sh\"\n",
    )
    .expect("the map parses");
    let err = map
        .with_detectors(&lib)
        .expect_err("two homes for one hook");
    assert!(err.to_string().contains("hook:no-unwrap"), "{err}");
}

/// **Only the hooks and repo targets read a detector**: hooks for write time,
/// repo for the commit-time dispatcher. An instruction layer is prose for a
/// model; a detector is a program for a shell. If another emitter began to
/// print one — a pattern quoted into a skill — that layer becomes text a
/// detector fires on, and the self-exclusion sweep in `check` turns red for a
/// reason nobody chose. Read from the source, comments stripped, because a
/// build that quietly consults a field arrives as an improvement.
#[test]
fn no_emitter_but_hooks_and_repo_reads_a_detector() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut checked = 0;
    let mut paths = vec![root.join("src/emit.rs")];
    for entry in fs::read_dir(root.join("src/emit")).expect("the emit directory") {
        paths.push(entry.expect("a readable entry").path());
    }
    for path in paths {
        if path
            .file_name()
            .is_some_and(|n| n == "hooks.rs" || n == "repo.rs")
        {
            continue;
        }
        let source = fs::read_to_string(&path).expect("a readable source file");
        let code: String = source
            .lines()
            .map(|line| line.find("//").map_or(line, |at| &line[..at]))
            .collect::<Vec<_>>()
            .join("\n");
        for marker in ["detectors()", "Detector"] {
            assert!(
                !code.contains(marker),
                "{} reads a detector ({marker}); only emit::hooks and emit::repo may",
                path.display()
            );
        }
        checked += 1;
    }
    assert!(checked >= 6, "the scan reached only {checked} file(s)");
}

/// During migration the hand-written original and the generated script share a
/// file name. A generated hook is wired only where it is **installed**,
/// `hooks/relearn/<name>.sh`; the original registered at `hooks/<name>.sh` must
/// not count, or `controls` would report the cut-over done before it happened.
#[test]
fn a_generated_hook_is_wired_only_at_its_install_path() {
    use relearn::resolve::{self, Found, HookRegistry, RelPath, RootName, Snapshot};

    let lib = library(&[rule("R:a", GRADUATED, DETECTOR)]);
    let map = ControlMap::parse("here = \"relearn\"\n")
        .expect("the map parses")
        .with_detectors(&lib)
        .expect("no hand entry for the detector");
    let here = RootName::parse("relearn").expect("a root name");
    let mut snapshot = Snapshot::new();
    snapshot.supply(here.clone()); // allow:clone: the snapshot owns the supplied root and `record` borrows it again
    snapshot.record(
        &here,
        RelPath::parse("hooks-pack/hooks/no-unwrap.sh").expect("a path"),
        Found::File("#!/usr/bin/env bash\n".to_owned()),
    );

    let settings = |command: &str| {
        HookRegistry::parse(&format!(
            r#"{{"hooks":{{"PreToolUse":[{{"matcher":"Write","hooks":[{{"type":"command","command":"{command}"}}]}}]}}}}"#
        ))
        .expect("settings parse")
    };
    let verdict = |registry: &HookRegistry| {
        resolve::resolve(&lib, &map, &snapshot, Some(registry)).rows()[0]
            .verdict()
            .label()
    };

    let original = settings("bash ~/.claude/hooks/no-unwrap.sh");
    assert_eq!(verdict(&original), "RESOLVED_NOT_WIRED");
    let generated = settings("bash ~/.claude/hooks/relearn/no-unwrap.sh");
    assert_eq!(verdict(&generated), "RESOLVED_AND_WIRED");
    // JSON-escaped, as a settings file on Windows holds it.
    let windows = settings(r"bash C:\\Users\\u\\.claude\\hooks\\relearn\\no-unwrap.sh");
    assert_eq!(verdict(&windows), "RESOLVED_AND_WIRED");
}
