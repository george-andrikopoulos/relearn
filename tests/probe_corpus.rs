//! The committed probes against what they need on disk, and against the
//! instruction layer they must never reach.
//!
//! A probe's checker is only evidence if it can be shown to say both things
//! (`[R:evidence-must-discriminate]`), so every probe carries a held case and a
//! violated case, and `scripts/probe-run.sh --self-test` runs both. This file
//! makes sure the cases exist in the shape the runner reads; the runner proves
//! they produce the verdicts their names claim.

use std::fs;
use std::path::{Path, PathBuf};

use relearn::fsio;
use relearn::rule::{Checker, Rule, Workspace};

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn probed_rules() -> Vec<Rule> {
    let library = fsio::load_rules(&root().join("rules"))
        .expect("the corpus loads")
        .validate()
        .expect("the corpus validates");
    library
        .rules()
        .iter()
        .filter(|r| r.probe().is_some())
        .cloned() // allow:clone: the test owns its rules after the library is dropped
        .collect()
}

fn body(rule: &Rule) -> &str {
    rule.tag()
        .as_str()
        .strip_prefix("R:")
        .expect("a tag starts R:")
}

#[test]
fn every_probe_has_its_fixture_and_a_case_of_each_verdict() {
    let rules = probed_rules();
    assert!(
        !rules.is_empty(),
        "no rule carries a probe; the check measured nothing"
    );
    for rule in &rules {
        let probe = rule.probe().expect("filtered on probe");
        let dir = root().join("tests/probes").join(body(rule));
        let workspace = dir.join("workspace");
        match probe.workspace() {
            Workspace::Fixture => {
                assert!(workspace.is_dir(), "{} has no workspace/", dir.display())
            }
            Workspace::Empty => assert!(
                !workspace.exists(),
                "{} is `empty` but has a workspace/",
                dir.display()
            ),
        }
        let check = dir.join("check.sh");
        let cases = dir.join("cases");
        match probe.checker() {
            Checker::Trace(_) => {
                for case in ["held.jsonl", "violated.jsonl"] {
                    assert!(
                        cases.join(case).is_file(),
                        "{} lacks cases/{case}",
                        dir.display()
                    );
                }
                assert!(
                    !check.exists(),
                    "{}: check.sh belongs to a run checker",
                    dir.display()
                );
            }
            Checker::Diff(_) | Checker::Run(_) => {
                for case in ["held", "violated"] {
                    assert!(
                        cases.join(case).is_dir(),
                        "{} lacks cases/{case}/",
                        dir.display()
                    );
                }
                let is_run = matches!(probe.checker(), Checker::Run(_));
                assert_eq!(
                    check.is_file(),
                    is_run,
                    "{}: check.sh iff a run checker",
                    dir.display()
                );
            }
        }
    }
}

#[test]
fn every_probe_directory_belongs_to_a_probe() {
    let bodies: Vec<String> = probed_rules().iter().map(|r| body(r).to_owned()).collect();
    for entry in fs::read_dir(root().join("tests/probes")).expect("tests/probes exists") {
        let name = entry.expect("a readable entry").file_name();
        let name = name.to_string_lossy();
        assert!(
            bodies.iter().any(|b| b == name.as_ref()),
            "tests/probes/{name} belongs to no rule carrying a probe"
        );
    }
}

/// Every `.rs` file on the build path: the emitters.
fn emitter_sources() -> Vec<PathBuf> {
    let mut paths = vec![root().join("src/emit.rs")];
    for entry in fs::read_dir(root().join("src/emit")).expect("the emit directory") {
        paths.push(entry.expect("a readable entry").path());
    }
    paths
}

/// A probe task that reached an instruction layer would put the answer in the
/// WITH arm, and the measurement would measure the leak.
#[test]
fn no_emitter_reads_a_probe() {
    let mut checked = 0;
    for path in emitter_sources() {
        let source = fs::read_to_string(&path).expect("a readable source file");
        let code: String = source
            .lines()
            .map(|line| line.find("//").map_or(line, |at| &line[..at]))
            .collect::<Vec<_>>()
            .join("\n");
        for marker in ["probe()", "Probe", "crate::probe"] {
            assert!(
                !code.contains(marker),
                "{} reads a probe ({marker}); no emitter may",
                path.display()
            );
        }
        checked += 1;
    }
    assert!(checked >= 6, "the scan reached only {checked} file(s)");
}

/// The production half of the same guarantee: no committed emitted file, in
/// any pack or layer, contains a probe task.
#[test]
fn no_committed_emitted_file_contains_a_probe_task() {
    let tasks: Vec<String> = probed_rules()
        .iter()
        .map(|r| r.probe().expect("filtered").task().as_str().to_owned())
        .collect();
    let mut stack: Vec<PathBuf> = [
        "skills",
        "claude-pack",
        "copilot-pack",
        "hooks-pack",
        ".claude/rules",
        ".cursor/rules",
        "AGENTS.md",
        ".github/copilot-instructions.md",
    ]
    .iter()
    .map(|p| root().join(p))
    .filter(|p| p.exists())
    .collect();
    let mut read = 0;
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            for entry in fs::read_dir(&path).expect("a readable directory") {
                stack.push(entry.expect("a readable entry").path());
            }
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        read += 1;
        for task in &tasks {
            assert!(
                !text.contains(task.as_str()),
                "{} contains a probe task",
                path.display()
            );
        }
    }
    assert!(
        read > 0,
        "no emitted file was read; the check measured nothing"
    );
}
