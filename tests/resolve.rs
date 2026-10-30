//! `relearn controls` — every control a rule names gets exactly one verdict.
//!
//! A `partial` or `graduated` rule names the artefacts that hold it, and until
//! 2026-10-30 nothing checked that any of them existed or ran. This file pins
//! each verdict against **synthetic roots** under `tests/fixtures/controls/`,
//! through the real binary — the exit code is the half a gate reads, and it is
//! only observable outside the process (`[R:verdict-survives-the-channel]`).
//!
//! It also holds the one check that runs on every push: every control this
//! repository itself holds resolves and is wired, through the same
//! `fsio::snapshot` the command uses (`[R:verify-through-production-path]`).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use proptest::prelude::*;
use relearn::fsio;
use relearn::resolve::{self, Found, HookRegistry, Outcome, RelPath, RootName, Snapshot};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/controls")
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// A synthetic rule whose status names `controls`.
fn rule(body: &str, controls: &str) -> String {
    format!(
        r#"+++
tag = "R:{body}"
title = "A synthetic rule"
error_class = "a synthetic error class"
home = {{ kind = "global" }}
created = "2026-10-30"
origin = "mined"
status = {{ kind = "graduated", to = "{controls}", date = "2026-10-30" }}
incident = "Synthetic."
+++

Synthetic.
"#
    )
}

/// A temporary corpus: one rule per `(tag body, controls)`, and a map.
fn corpus(rules: &[(&str, &str)], map: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("a temp dir");
    let rules_dir = dir.path().join("rules");
    fs::create_dir_all(&rules_dir).expect("create the rules dir");
    for (body, controls) in rules {
        fs::write(rules_dir.join(format!("{body}.md")), rule(body, controls))
            .expect("write a rule");
    }
    fs::write(
        dir.path().join("controls.toml"),
        format!("here = \"corpus\"\n{map}"),
    )
    .expect("write the map");
    dir
}

/// Run `relearn controls` over a corpus; returns the exit code and stdout.
fn run(dir: &Path, extra: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_relearn"))
        .arg("controls")
        .arg("--rules")
        .arg(dir.join("rules"))
        .arg("--map")
        .arg(dir.join("controls.toml"))
        .args(extra)
        .output()
        .expect("run the relearn binary");
    let code = output.status.code().expect("an exit code, not a signal");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (code, format!("{stdout}{stderr}"))
}

fn root(name: &str, sub: &str) -> String {
    format!("{name}={}", fixtures().join(sub).display())
}

fn settings() -> String {
    fixtures().join("settings.json").display().to_string()
}

/// The table line for one control, so an assertion reads the verdict that
/// control got rather than any verdict anywhere in the output.
fn line_for<'a>(out: &'a str, control: &str) -> &'a str {
    out.lines()
        .find(|l| l.contains(&format!(" {control} ")) || l.ends_with(&format!(" {control}")))
        .unwrap_or_else(|| panic!("no table line for `{control}` in:\n{out}"))
}

const HOOKS: &str = r#"
[[control]]
control = "hook:wired"
root = "hooks"
path = "hooks/wired.sh"

[[control]]
control = "hook:unwired"
root = "hooks"
path = "hooks/unwired.sh"

[[control]]
control = "hook:absent"
root = "hooks"
path = "hooks/absent.sh"
"#;

#[test]
fn a_hook_the_settings_register_is_resolved_and_wired_and_exits_zero() {
    let dir = corpus(&[("a", "hook:wired")], &only(HOOKS, "hook:wired"));
    let (code, out) = run(
        dir.path(),
        &[
            "--root",
            &root("hooks", "hooks-root"),
            "--settings",
            &settings(),
        ],
    );
    assert!(
        line_for(&out, "hook:wired").starts_with("RESOLVED_AND_WIRED"),
        "{out}"
    );
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("PASS"), "{out}");
}

#[test]
fn a_hook_present_but_unregistered_is_not_wired_even_when_other_settings_name_it() {
    // settings.json names unwired.sh in `permissions` and `statusLine`. Neither
    // is a hook registration, and a text search would have accepted both.
    let dir = corpus(&[("a", "hook:unwired")], &only(HOOKS, "hook:unwired"));
    let (code, out) = run(
        dir.path(),
        &[
            "--root",
            &root("hooks", "hooks-root"),
            "--settings",
            &settings(),
        ],
    );
    assert!(
        line_for(&out, "hook:unwired").starts_with("RESOLVED_NOT_WIRED"),
        "{out}"
    );
    assert_eq!(code, 1, "{out}");
}

#[test]
fn a_hook_whose_script_is_absent_is_missing() {
    let dir = corpus(&[("a", "hook:absent")], &only(HOOKS, "hook:absent"));
    let (code, out) = run(
        dir.path(),
        &[
            "--root",
            &root("hooks", "hooks-root"),
            "--settings",
            &settings(),
        ],
    );
    assert!(
        line_for(&out, "hook:absent").starts_with("MISSING"),
        "{out}"
    );
    assert_eq!(code, 1, "{out}");
}

#[test]
fn a_hook_with_no_settings_file_is_uncheckable_and_exits_two() {
    let dir = corpus(&[("a", "hook:wired")], &only(HOOKS, "hook:wired"));
    let (code, out) = run(dir.path(), &["--root", &root("hooks", "hooks-root")]);
    assert!(
        line_for(&out, "hook:wired").starts_with("UNCHECKABLE"),
        "{out}"
    );
    assert_eq!(code, 2, "a disarmed check is not a pass:\n{out}");
    assert!(!out.contains("PASS"), "{out}");
}

#[test]
fn a_control_whose_root_is_not_supplied_is_uncheckable_and_exits_two() {
    let dir = corpus(&[("a", "hook:wired")], &only(HOOKS, "hook:wired"));
    let (code, out) = run(dir.path(), &["--settings", &settings()]);
    assert!(
        line_for(&out, "hook:wired").starts_with("UNCHECKABLE"),
        "{out}"
    );
    assert_eq!(code, 2, "{out}");
}

#[test]
fn a_definite_failure_outranks_an_uncheckable_control() {
    let map = format!(
        "{}{}",
        only(HOOKS, "hook:absent"),
        only(HOOKS, "hook:wired")
    );
    let dir = corpus(&[("a", "hook:absent"), ("b", "hook:wired")], &map);
    // No --settings: hook:wired is UNCHECKABLE, hook:absent is MISSING whether
    // or not anything registers it. A definite failure must not be reported as
    // "disarmed" because something else could not be checked.
    let (code, out) = run(dir.path(), &["--root", &root("hooks", "hooks-root")]);
    assert!(
        line_for(&out, "hook:absent").starts_with("MISSING"),
        "{out}"
    );
    assert!(
        line_for(&out, "hook:wired").starts_with("UNCHECKABLE"),
        "{out}"
    );
    assert_eq!(code, 1, "{out}");
}

#[test]
fn a_control_the_map_does_not_place_is_unmapped_and_fails() {
    let dir = corpus(&[("a", "hook:nowhere")], "");
    let (code, out) = run(dir.path(), &[]);
    assert!(
        line_for(&out, "hook:nowhere").starts_with("UNMAPPED"),
        "{out}"
    );
    assert_eq!(code, 1, "{out}");
}

#[test]
fn a_map_entry_no_rule_names_fails_the_run() {
    let dir = corpus(&[], &only(HOOKS, "hook:wired"));
    let (code, out) = run(
        dir.path(),
        &[
            "--root",
            &root("hooks", "hooks-root"),
            "--settings",
            &settings(),
        ],
    );
    assert!(
        out.contains("hook:wired"),
        "the stale entry is named:\n{out}"
    );
    assert_eq!(code, 1, "{out}");
}

const TESTS: &str = r#"
[[control]]
control = "test:runs_by_default"
root = "code"
path = "tests/suite.rs"
function = "runs_by_default"

[[control]]
control = "test:ignored_by_default"
root = "code"
path = "tests/suite.rs"
function = "ignored_by_default"

[[control]]
control = "test:not_a_test"
root = "code"
path = "tests/suite.rs"
function = "not_a_test"

[[control]]
control = "test:only_in_a_comment"
root = "code"
path = "tests/suite.rs"
function = "only_in_a_comment"

[[control]]
control = "property:the_only_case"
root = "code"
path = "."
function = "the_only_case"

[[control]]
control = "test:tests/whole.rs"
root = "code"
path = "tests/whole.rs"

[[control]]
control = "test:tests/no_tests.rs"
root = "code"
path = "tests/no_tests.rs"

[[control]]
control = "gate:scripts/gate.sh"
root = "code"
path = "scripts/gate.sh"
invoked_by = "scripts/invoker.sh"

[[control]]
control = "gate:scripts/gate.sh silently"
root = "code"
path = "scripts/gate.sh"
invoked_by = "scripts/silent-invoker.sh"

[[control]]
control = "gate:gate.sh check_it"
root = "code"
path = "scripts/gate.sh"
function = "check_it"
invoked_by = "scripts/gate.sh"

[[control]]
control = "gate:gate.sh never_called"
root = "code"
path = "scripts/gate.sh"
function = "never_called"
invoked_by = "scripts/gate.sh"
"#;

/// The map with only the entry for `control`.
fn only(map: &str, control: &str) -> String {
    map.split("[[control]]")
        .filter(|chunk| chunk.contains(&format!("control = \"{control}\"")))
        .map(|chunk| format!("\n[[control]]{chunk}"))
        .collect()
}

/// Resolve one code-root control and return its table line.
fn verdict_of(control: &str) -> (i32, String) {
    let dir = corpus(&[("a", control)], &only(TESTS, control));
    let (code, out) = run(dir.path(), &["--root", &root("code", "code-root")]);
    (code, line_for(&out, control).to_owned())
}

#[test]
fn a_test_function_the_harness_runs_is_wired() {
    let (code, line) = verdict_of("test:runs_by_default");
    assert!(line.starts_with("RESOLVED_AND_WIRED"), "{line}");
    assert_eq!(code, 0);
}

#[test]
fn an_ignored_test_is_not_wired() {
    let (code, line) = verdict_of("test:ignored_by_default");
    assert!(line.starts_with("RESOLVED_NOT_WIRED"), "{line}");
    assert_eq!(code, 1);
}

#[test]
fn a_function_with_no_test_attribute_is_not_wired() {
    let (code, line) = verdict_of("test:not_a_test");
    assert!(line.starts_with("RESOLVED_NOT_WIRED"), "{line}");
    assert_eq!(code, 1);
}

#[test]
fn a_test_named_only_in_a_comment_is_missing() {
    let (code, line) = verdict_of("test:only_in_a_comment");
    assert!(line.starts_with("MISSING"), "{line}");
    assert_eq!(code, 1);
}

#[test]
fn a_test_function_is_found_anywhere_under_its_root() {
    let (code, line) = verdict_of("property:the_only_case");
    assert!(line.starts_with("RESOLVED_AND_WIRED"), "{line}");
    assert!(
        line.contains("tests/whole.rs"),
        "names where it was found: {line}"
    );
    assert_eq!(code, 0);
}

#[test]
fn a_test_file_with_a_test_is_wired_and_one_without_is_not() {
    assert!(
        verdict_of("test:tests/whole.rs")
            .1
            .starts_with("RESOLVED_AND_WIRED")
    );
    assert!(
        verdict_of("test:tests/no_tests.rs")
            .1
            .starts_with("RESOLVED_NOT_WIRED")
    );
}

#[test]
fn a_gate_its_invoker_runs_is_wired_and_a_commented_out_call_is_not() {
    assert!(
        verdict_of("gate:scripts/gate.sh")
            .1
            .starts_with("RESOLVED_AND_WIRED")
    );
    assert!(
        verdict_of("gate:scripts/gate.sh silently")
            .1
            .starts_with("RESOLVED_NOT_WIRED")
    );
}

#[test]
fn a_gate_function_is_wired_by_a_call_and_not_by_its_own_definition() {
    assert!(
        verdict_of("gate:gate.sh check_it")
            .1
            .starts_with("RESOLVED_AND_WIRED")
    );
    assert!(
        verdict_of("gate:gate.sh never_called")
            .1
            .starts_with("RESOLVED_NOT_WIRED")
    );
}

#[test]
fn a_type_control_has_no_resolver_and_is_uncheckable() {
    let dir = corpus(&[("a", "type:SomeWitness")], "");
    let (code, out) = run(dir.path(), &[]);
    assert!(
        line_for(&out, "type:SomeWitness").starts_with("UNCHECKABLE"),
        "{out}"
    );
    assert_eq!(code, 2, "{out}");
}

#[test]
fn a_root_the_map_never_names_is_refused() {
    let dir = corpus(&[("a", "hook:wired")], &only(HOOKS, "hook:wired"));
    let (code, out) = run(dir.path(), &["--root", &root("hookz", "hooks-root")]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("hookz"), "{out}");
}

#[test]
fn a_map_entry_that_describes_no_artefact_is_refused() {
    for bad in [
        // A gate with nothing to run it.
        "[[control]]\ncontrol = \"gate:g\"\nroot = \"code\"\npath = \"scripts/gate.sh\"\n",
        // A hook with a function.
        "[[control]]\ncontrol = \"hook:h\"\nroot = \"hooks\"\npath = \"hooks/wired.sh\"\nfunction = \"f\"\n",
        // A path that climbs out of its root.
        "[[control]]\ncontrol = \"hook:h\"\nroot = \"hooks\"\npath = \"../hooks/wired.sh\"\n",
        // Two answers for one control.
        "[[control]]\ncontrol = \"hook:h\"\nroot = \"hooks\"\npath = \"a.sh\"\n[[control]]\ncontrol = \"hook:h\"\nroot = \"hooks\"\npath = \"b.sh\"\n",
    ] {
        let dir = corpus(&[], bad);
        let (code, out) = run(dir.path(), &[]);
        assert_eq!(code, 1, "refused:\n{bad}\n{out}");
    }
}

// ---------------------------------------------------------------------------
// The real corpus.
// ---------------------------------------------------------------------------

fn real_inputs() -> (
    relearn::library::Library<relearn::library::Validated>,
    resolve::ControlMap,
) {
    let library = fsio::load_rules(&repo_root().join("rules"))
        .expect("the corpus loads")
        .validate()
        .expect("the corpus validates");
    let map =
        fsio::read_control_map(&repo_root().join("controls.toml")).expect("controls.toml parses");
    (library, map)
}

/// Every control the corpus names whose map entry places it in **this**
/// repository resolves and is wired; every other one is placed somewhere, and
/// no map entry is stale. Runs on every push through `cargo test`, and needs
/// nothing cloned beside the repository — which is why the full check is a
/// separate command and this is its in-tree half.
#[test]
fn every_control_this_repository_holds_resolves_and_is_wired() {
    let (library, map) = real_inputs();
    let snapshot = fsio::snapshot(&map, &repo_root(), &[]).expect("the repository root reads");
    let resolution = resolve::resolve(&library, &map, &snapshot, None);

    assert!(
        resolution.unclaimed().is_empty(),
        "controls.toml places controls no rule names: {:?}",
        resolution.unclaimed()
    );
    let mut held_here = 0;
    for row in resolution.rows() {
        let label = row.verdict().label();
        assert_ne!(
            label,
            "UNMAPPED",
            "{} names {} and controls.toml does not place it",
            row.tag(),
            row.control()
        );
        let here = map
            .locate(&resolve::ControlId::of(row.control()))
            .is_some_and(|loc| loc.root() == map.here());
        if here {
            held_here += 1;
            assert_eq!(
                label,
                "RESOLVED_AND_WIRED",
                "{} {}: {}",
                row.tag(),
                row.control(),
                row.verdict()
            );
        } else {
            assert_eq!(
                label,
                "UNCHECKABLE",
                "{} {}: {}",
                row.tag(),
                row.control(),
                row.verdict()
            );
        }
    }
    // A floor, so a map that stopped placing anything here fails rather than
    // passing over nothing.
    assert!(
        held_here >= 2,
        "only {held_here} control(s) resolved in this repository"
    );
}

// ---------------------------------------------------------------------------
// Laws, over the real corpus and generated states of the world.
// ---------------------------------------------------------------------------

fn found_strategy() -> impl Strategy<Value = Found> {
    prop_oneof![
        Just(Found::Absent),
        Just(Found::File("#[test]\nfn x() {}\n".to_owned())),
        Just(Found::File("fn x() {}\n".to_owned())),
        Just(Found::File(String::new())),
    ]
}

proptest! {
    /// Whatever is supplied and whatever is on disk, every control occurrence
    /// in the corpus gets exactly one verdict, in corpus order; the run passes
    /// only when every verdict is RESOLVED_AND_WIRED; and an UNCHECKABLE
    /// verdict can never produce a pass.
    #[test]
    fn every_control_gets_exactly_one_verdict_and_only_all_wired_passes(
        supply in proptest::collection::vec(any::<bool>(), 8),
        contents in proptest::collection::vec(found_strategy(), 64),
        with_settings in any::<bool>(),
        registered in proptest::collection::vec(any::<bool>(), 32),
    ) {
        let (library, map) = real_inputs();
        let roots: Vec<&RootName> = map.roots().into_iter().collect();
        let mut snapshot = Snapshot::new();
        for (at, root) in roots.iter().enumerate() {
            if supply[at % supply.len()] {
                snapshot.supply((*root).clone());
            }
        }
        for (at, (root, path)) in map.reads().into_iter().enumerate() {
            snapshot.record(root, path.clone(), contents[at % contents.len()].clone());
        }
        let names: Vec<String> = map
            .reads()
            .into_iter()
            .map(|(_, p): (&RootName, &RelPath)| p.file_name().to_owned())
            .collect();
        let commands: Vec<String> = names
            .iter()
            .enumerate()
            .filter(|(at, _)| registered[at % registered.len()])
            .map(|(_, n)| format!(r#"{{"type":"command","command":"bash ~/h/{n}"}}"#))
            .collect();
        let json = format!(
            r#"{{"hooks":{{"PreToolUse":[{{"matcher":"*","hooks":[{}]}}]}}}}"#,
            commands.join(",")
        );
        let registry = HookRegistry::parse(&json).expect("generated settings parse");
        let settings = with_settings.then_some(&registry);

        let resolution = resolve::resolve(&library, &map, &snapshot, settings);

        let expected: Vec<(String, String)> = {
            let mut rules: Vec<_> = library.rules().iter().collect();
            rules.sort_by(|a, b| a.tag().as_str().cmp(b.tag().as_str()));
            rules
                .into_iter()
                .flat_map(|r| {
                    r.status()
                        .named_controls()
                        .map(|c| c.as_slice().iter().map(|c| (r.tag().to_string(), c.to_string())).collect::<Vec<_>>())
                        .unwrap_or_default()
                })
                .collect()
        };
        let got: Vec<(String, String)> = resolution
            .rows()
            .iter()
            .map(|row| (row.tag().to_string(), row.control().to_string()))
            .collect();
        prop_assert_eq!(&got, &expected);

        let all_wired = resolution.rows().iter().all(|r| r.verdict().label() == "RESOLVED_AND_WIRED");
        let any_uncheckable = resolution.rows().iter().any(|r| r.verdict().label() == "UNCHECKABLE");
        prop_assert_eq!(resolution.outcome() == Outcome::Pass, all_wired && resolution.unclaimed().is_empty());
        if any_uncheckable {
            prop_assert_ne!(resolution.outcome(), Outcome::Pass);
        }
    }
}
