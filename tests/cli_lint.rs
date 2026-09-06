//! `relearn lint` exit-code policy, exercised through the **real binary** — the
//! same channel CI runs, not the dispatch function behind it
//! (`[R:verify-through-production-path]`). The exit code is the whole feature
//! here: a gate whose verdict is right and whose status is wrong is the failure
//! `[R:verdict-survives-the-channel]` names, and it is only observable from
//! outside the process.
//!
//! The binary is located with `CARGO_BIN_EXE_relearn`, which cargo sets for the
//! crate's own bins and which carries the platform's executable suffix — never a
//! constructed `target/<profile>/relearn` path, which ignores both
//! `CARGO_TARGET_DIR` and `.exe` (`[R:xplat-fixtures]`).

use std::fs;
use std::path::Path;
use std::process::Command;

/// Write a one-rule library into a fresh temp dir and return it. `recurrence`
/// is spliced into the front-matter verbatim when non-empty.
fn library(dir: &Path, recurrence: &str) {
    fs::create_dir_all(dir).expect("create the rules dir");
    let doc = format!(
        r#"+++
tag = "R:demo"
title = "A demo rule"
error_class = "a demo error class"
home = {{ kind = "global" }}
created = "2026-08-16"
origin = "mined"
status = {{ kind = "active" }}
incident = "The triggering incident."
{recurrence}+++

Do the thing.
"#
    );
    fs::write(dir.join("demo.md"), doc).expect("write the rule file");
}

const A_RECURRENCE: &str = r#"
[[recurrence]]
date = "2026-08-24"
incident = "It happened again."
"#;

fn lint(rules: &Path, deny: Option<&str>) -> (bool, String) {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_relearn"));
    cmd.arg("lint").arg("--rules").arg(rules);
    if let Some(level) = deny {
        cmd.arg("--deny").arg(level);
    }
    let out = cmd.output().expect("the relearn binary runs");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    (out.status.success(), text)
}

#[test]
fn a_clean_library_succeeds_at_every_deny_level() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    library(&rules, "");
    for deny in [None, Some("warning"), Some("error")] {
        let (ok, text) = lint(&rules, deny);
        assert!(ok, "a clean library must succeed (deny={deny:?}): {text}");
    }
}

/// The default is unchanged by the flag's introduction: a warning still fails
/// the run, so no existing invocation silently became permissive.
#[test]
fn a_warning_fails_the_run_by_default() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    library(&rules, A_RECURRENCE);
    let (ok, text) = lint(&rules, None);
    assert!(
        !ok,
        "an unheld recurrence must fail the default run: {text}"
    );
    assert!(text.contains("unheld recurrence"), "{text}");
}

#[test]
fn deny_warning_is_the_default_spelled_out() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    library(&rules, A_RECURRENCE);
    let (ok, _) = lint(&rules, Some("warning"));
    assert!(!ok, "--deny warning must match the default");
}

/// The point of the flag. A governance finding — one that may legitimately stand
/// for weeks because clearing it is work in another repository — is **reported
/// but not fatal**, so the gate can run in CI without going permanently red. It
/// is made non-fatal, never hidden: the finding must still be printed.
#[test]
fn deny_error_reports_a_warning_without_failing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    library(&rules, A_RECURRENCE);
    let (ok, text) = lint(&rules, Some("error"));
    assert!(ok, "--deny error must not fail on a warning: {text}");
    assert!(
        text.contains("unheld recurrence"),
        "the finding must still be printed, only made non-fatal: {text}"
    );
}

#[test]
fn an_unknown_deny_level_is_rejected_before_anything_is_read() {
    let dir = tempfile::tempdir().expect("tempdir");
    let rules = dir.path().join("rules");
    // Deliberately no rules directory contents: the value must be rejected by
    // argument parsing, before the loader ever looks at the path.
    let (ok, _) = lint(&rules, Some("catastrophe"));
    assert!(!ok, "an unknown --deny value must be a parse error");
}
