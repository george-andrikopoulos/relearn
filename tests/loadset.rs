//! `relearn budget`'s load set: every file counted once and classified once,
//! and one pin per finding, each over a synthetic tree built in a temp
//! directory. The trees are built at test time rather than committed, because
//! a committed `CLAUDE.md` or `.claude/rules/` under `tests/` would itself be
//! loaded on demand by a Claude Code session working in this repository.

use std::fs;
use std::path::{Path, PathBuf};

use proptest::prelude::*;
use relearn::fsio::{Ancestors, discover_load_set};
use relearn::loadset::{
    Authorship, Coverage, Finding, LayerSlug, Limits, LoadSet, LoadedFile, LoaderChars, Measured,
    Outcome, Residency, Share, assess,
};

const MARKER: &str = "\n<!-- relearn:generated v0 sha256=0 rules=R:x -- DO NOT EDIT -->\n";

fn put(root: &Path, rel: &str, text: &str) {
    let path = root.join(rel);
    fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
    fs::write(path, text).expect("write fixture");
}

fn limits(limit: usize, budget: usize) -> Limits {
    Limits::new(LoaderChars::new(limit), LoaderChars::new(budget)).expect("valid limits")
}

fn relearn() -> LayerSlug {
    LayerSlug::project("relearn")
}

/// A user dir and a project dir side by side, neither an ancestor of the other.
fn trees() -> (tempfile::TempDir, PathBuf, PathBuf) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let user = tmp.path().join("home").join(".claude");
    let project = tmp.path().join("work").join("relearn");
    fs::create_dir_all(&user).expect("mkdir user");
    fs::create_dir_all(&project).expect("mkdir project");
    (tmp, user, project)
}

fn codes(findings: &[Finding]) -> Vec<&'static str> {
    findings.iter().map(Finding::code).collect()
}

#[test]
fn a_file_reached_by_import_and_by_the_rules_dir_is_counted_once() {
    let (_tmp, user, project) = trees();
    put(
        &project,
        "CLAUDE.md",
        "charter\n@.claude/rules/project-relearn.md\n",
    );
    put(
        &project,
        ".claude/rules/project-relearn.md",
        &format!("rules{MARKER}"),
    );
    let set =
        LoadSet::new(discover_load_set(&project, Some(&user), Ancestors::Skip).expect("walk"));
    assert_eq!(set.files().len(), 2);
    let a = assess(&set, &relearn(), Coverage::Whole, limits(100, 100), None);
    assert!(a.findings().is_empty(), "{:?}", a.findings());
}

#[test]
fn over_limit_fires_on_the_total_and_on_a_single_file() {
    let (_tmp, user, project) = trees();
    put(&user, "CLAUDE.md", &"u".repeat(60));
    put(&project, "CLAUDE.md", &"p".repeat(60));
    let set =
        LoadSet::new(discover_load_set(&project, Some(&user), Ancestors::Skip).expect("walk"));
    let a = assess(&set, &relearn(), Coverage::Whole, limits(100, 90), None);
    assert_eq!(codes(a.findings()), ["OVER_LIMIT"]);
    assert_eq!(a.outcome(), Outcome::Findings);

    let a = assess(&set, &relearn(), Coverage::Whole, limits(50, 50), None);
    assert_eq!(
        codes(a.findings()),
        ["OVER_LIMIT", "OVER_LIMIT", "OVER_LIMIT"]
    );
}

#[test]
fn over_budget_fires_below_the_limit_and_on_the_project_share() {
    let (_tmp, user, project) = trees();
    put(&user, "CLAUDE.md", &"u".repeat(40));
    put(&project, "CLAUDE.md", &"p".repeat(40));
    let set =
        LoadSet::new(discover_load_set(&project, Some(&user), Ancestors::Skip).expect("walk"));
    let a = assess(&set, &relearn(), Coverage::Whole, limits(100, 70), None);
    assert_eq!(
        a.findings(),
        [Finding::OverBudget {
            measured: Measured::Total,
            chars: LoaderChars::new(80),
            budget: LoaderChars::new(70),
        }]
    );
    let a = assess(
        &set,
        &relearn(),
        Coverage::Whole,
        limits(100, 100),
        Some(LoaderChars::new(30)),
    );
    assert_eq!(
        a.findings(),
        [Finding::OverBudget {
            measured: Measured::ProjectShare,
            chars: LoaderChars::new(40),
            budget: LoaderChars::new(30),
        }]
    );
}

#[test]
fn path_scoped_files_do_not_count_toward_the_total() {
    let (_tmp, user, project) = trees();
    put(
        &user,
        "rules/domain-rust.md",
        &format!("---\npaths:\n  - \"**/*.rs\"\n---\n{}", "r".repeat(500)),
    );
    put(&project, "CLAUDE.md", "p");
    let set =
        LoadSet::new(discover_load_set(&project, Some(&user), Ancestors::Skip).expect("walk"));
    assert_eq!(set.always_total(), LoaderChars::new(1));
    let a = assess(&set, &relearn(), Coverage::Whole, limits(1000, 10), None);
    assert!(a.findings().is_empty());
}

#[test]
fn duplicate_layer_fires_when_one_home_loads_from_user_and_project() {
    let (_tmp, user, project) = trees();
    let rust = format!("---\npaths: [\"**/*.rs\"]\n---\nrust{MARKER}");
    put(&user, "rules/domain-rust.md", &rust);
    put(&project, ".claude/rules/domain-rust.md", &rust);
    let set =
        LoadSet::new(discover_load_set(&project, Some(&user), Ancestors::Skip).expect("walk"));
    let a = assess(&set, &relearn(), Coverage::Whole, limits(1000, 1000), None);
    assert_eq!(codes(a.findings()), ["DUPLICATE_LAYER"]);
}

#[test]
fn a_hand_authored_file_with_a_layer_name_is_not_a_duplicate() {
    let (_tmp, user, project) = trees();
    put(&user, "rules/domain-rust.md", "mine");
    put(
        &project,
        ".claude/rules/domain-rust.md",
        &format!("gen{MARKER}"),
    );
    let set =
        LoadSet::new(discover_load_set(&project, Some(&user), Ancestors::Skip).expect("walk"));
    let a = assess(&set, &relearn(), Coverage::Whole, limits(1000, 1000), None);
    assert!(a.findings().is_empty());
}

#[test]
fn foreign_project_layer_fires_for_another_projects_home() {
    let (_tmp, user, project) = trees();
    put(
        &project,
        ".claude/rules/project-relearn.md",
        &format!("own{MARKER}"),
    );
    put(
        &project,
        ".claude/rules/project-other.md",
        &format!("theirs{MARKER}"),
    );
    let set =
        LoadSet::new(discover_load_set(&project, Some(&user), Ancestors::Skip).expect("walk"));
    let a = assess(&set, &relearn(), Coverage::Whole, limits(1000, 1000), None);
    assert_eq!(codes(a.findings()), ["FOREIGN_PROJECT_LAYER"]);
}

#[test]
fn no_user_dir_is_unchecked_never_a_pass() {
    let (_tmp, _user, project) = trees();
    put(&project, "CLAUDE.md", "p");
    let set = LoadSet::new(discover_load_set(&project, None, Ancestors::Skip).expect("walk"));
    let a = assess(
        &set,
        &relearn(),
        Coverage::UserUnchecked,
        limits(10, 10),
        None,
    );
    assert_eq!(a.outcome(), Outcome::Unchecked);
    let a = assess(
        &set,
        &relearn(),
        Coverage::ProjectOnly,
        limits(10, 10),
        None,
    );
    assert_eq!(a.outcome(), Outcome::Clean);
}

#[test]
fn rules_dirs_are_walked_recursively_and_imports_resolve_against_the_importer() {
    let (_tmp, user, project) = trees();
    put(&user, "rules/ecc/common/a.md", "aaaa");
    put(&project, "docs/CLAUDE-extra.md", "extra");
    put(&project, ".claude/CLAUDE.md", "@../docs/CLAUDE-extra.md");
    let set =
        LoadSet::new(discover_load_set(&project, Some(&user), Ancestors::Skip).expect("walk"));
    let shown: Vec<&str> = set.files().iter().map(LoadedFile::shown).collect();
    assert!(shown.iter().any(|s| s.ends_with("a.md")), "{shown:?}");
    assert!(
        shown.iter().any(|s| s.ends_with("CLAUDE-extra.md")),
        "{shown:?}"
    );
}

#[test]
fn imports_stop_after_four_hops() {
    let (_tmp, _user, project) = trees();
    put(&project, "CLAUDE.md", "@h1.md");
    put(&project, "h1.md", "@h2.md");
    put(&project, "h2.md", "@h3.md");
    put(&project, "h3.md", "@h4.md");
    put(&project, "h4.md", "@h5.md");
    put(&project, "h5.md", "too deep");
    let set = LoadSet::new(discover_load_set(&project, None, Ancestors::Skip).expect("walk"));
    let shown: Vec<&str> = set.files().iter().map(LoadedFile::shown).collect();
    assert!(shown.iter().any(|s| s.ends_with("h4.md")), "{shown:?}");
    assert!(!shown.iter().any(|s| s.ends_with("h5.md")), "{shown:?}");
}

fn arb_file() -> impl Strategy<Value = LoadedFile> {
    (
        0usize..12,
        prop_oneof![
            Just(Share::User),
            Just(Share::Ancestor),
            Just(Share::Project)
        ],
        any::<bool>(),
        0usize..5_000,
    )
        .prop_map(|(key, share, scoped, chars)| {
            let residency = if scoped {
                Residency::PathScoped(vec!["**/*.rs".into()])
            } else {
                Residency::Always
            };
            LoadedFile::new(
                PathBuf::from(format!("f{key}")),
                format!("f{key}"),
                share,
                residency,
                LoaderChars::new(chars),
                Authorship::HandAuthored,
            )
        })
}

proptest! {
    /// Every file in a load set is counted exactly once and classified exactly
    /// once: keys are unique, always + scoped is the sum over the files, and
    /// the per-share totals partition the always total.
    #[test]
    fn every_file_is_counted_once_and_classified_once(files in prop::collection::vec(arb_file(), 0..40)) {
        let set = LoadSet::new(files.clone());
        let mut keys: Vec<&Path> = set.files().iter().map(LoadedFile::key).collect();
        let n = keys.len();
        keys.sort();
        keys.dedup();
        prop_assert_eq!(keys.len(), n);

        // The first occurrence of each key is the one kept.
        let mut expected = Vec::new();
        for f in &files {
            if !expected.iter().any(|e: &LoadedFile| e.key() == f.key()) {
                expected.push(f.clone());
            }
        }
        prop_assert_eq!(set.files(), expected.as_slice());

        let sum: usize = set.files().iter().map(|f| f.chars().get()).sum();
        prop_assert_eq!(set.always_total().get() + set.scoped_total().get(), sum);
        let shares = set.share_total(Share::User).get()
            + set.share_total(Share::Ancestor).get()
            + set.share_total(Share::Project).get();
        prop_assert_eq!(shares, set.always_total().get());
    }
}

#[test]
fn ancestor_directories_load_between_the_bound_and_the_project() {
    let (tmp, user, project) = trees();
    put(&tmp.path().join("work"), "CLAUDE.md", "above");
    put(&tmp.path().join("work"), ".claude/rules/up.md", "up");
    let set = LoadSet::new(
        discover_load_set(&project, Some(&user), Ancestors::Below(tmp.path())).expect("walk"),
    );
    let shares: Vec<Share> = set.files().iter().map(LoadedFile::share).collect();
    assert_eq!(shares, [Share::Ancestor, Share::Ancestor]);
    assert_eq!(set.share_total(Share::Ancestor), LoaderChars::new(7));
}
