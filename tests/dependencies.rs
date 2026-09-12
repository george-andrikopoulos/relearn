//! Every dependency in every manifest is named in the decisions log, or the
//! build fails — the mechanical half of `[R:price-every-dependency]`.
//!
//! A crate enters a tree as one line in a manifest: no design discussion, no
//! diff worth reading, nothing that looks like a decision was taken. This gate
//! makes it look like one, by refusing the manifest line until the log carries
//! an entry naming it.
//!
//! **What it proves, and what it cannot.** It proves a dependency is *named*
//! in an entry that is explicitly about dependencies. It cannot read the entry
//! and judge whether the price is real — whether the rejected list is honest,
//! whether the default features were weighed, whether writing the thing by hand
//! was considered. And it is blind by construction to the other half of the
//! rule: a dependency *refused* leaves no manifest line, so there is no
//! artefact to check against and no gate can ever be written for it. A human
//! reader is the only detector there, and this file must never be cited as
//! covering that half.
//!
//! **The marker.** A decisions-log row prices a dependency when its **Decision**
//! cell — the second column — begins with `Dependency:` or `Dependencies:`. The
//! crate names are the backticked tokens in that cell.
//!
//! Two narrowings, each closing a way the check could have measured nothing.
//! A mention anywhere else in the log does not count: `serde` appears in this
//! log only as a `#[serde(default)]` attribute and as the rejected
//! `serde_yaml`, and `sha2` only inside another decision's rejected
//! alternative, so a substring search would have read three crates as priced
//! and reported a gate that passes without measuring. And the marker must
//! *open* the cell rather than appear anywhere in the row, so that a row
//! discussing this convention — such as the one recording it — cannot price a
//! crate by naming it in passing. A detector that matches its own definition is
//! `[R:detector-excludes-own-definitions]`, and this one would have.
//!
//! **The grandfather list shrinks and never grows.** The rule was written on
//! 2026-09-12, when nine dependencies were already in the tree and none had an
//! entry of its own. Retrofitting them would mean reconstructing rejected lists
//! nobody remembers, which `[R:decisions-log-records-rejected-alternatives]`
//! warns produces the flattering version. They are exempted by name instead, so
//! the gate binds every dependency added *after* the rule existed — which is
//! what the rule asks for: record it at the moment it is added. Two further
//! tests keep the exemption honest: an entry that is now priced must be removed,
//! and an entry whose crate has left the manifest must be removed.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The backtick, spelled so this file can talk about the marker without the
/// character appearing bare in the source.
const TICK: char = '\u{0060}';

/// Dependencies that predate `[R:price-every-dependency]` (2026-09-12) and are
/// exempt from it. **This list may only shrink.** Adding a name here to quiet
/// the gate would be adding a dependency without pricing it, using the control
/// that exists to stop exactly that.
const GRANDFATHERED: &[&str] = &[
    "anyhow",
    "clap",
    "proptest",
    "serde",
    "sha2",
    "tempfile",
    "thiserror",
    "toml",
    "trybuild",
];

/// The repository root, resolved from the manifest dir so the test does not
/// depend on the working directory it is run from.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Every `Cargo.toml` in the repository, `target/` and `.git/` excluded — a
/// crate added later cannot escape the gate by arriving as a new manifest.
fn manifests() -> Vec<PathBuf> {
    let mut found = Vec::new();
    collect_manifests(&repo_root(), &mut found);
    found.sort();
    found
}

fn collect_manifests(dir: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let skip = matches!(
                path.file_name().and_then(|n| n.to_str()),
                Some("target" | ".git" | "node_modules")
            );
            if !skip {
                collect_manifests(&path, found);
            }
        } else if path.file_name().and_then(|n| n.to_str()) == Some("Cargo.toml") {
            found.push(path);
        }
    }
}

/// Every dependency name declared in every manifest: runtime, dev, build, and
/// the per-target tables. The manifest **key** is the name, which is what a
/// reader of the file sees — a renamed dependency is priced under the name the
/// manifest gives it, not the one crates.io does.
fn declared_dependencies() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for manifest in manifests() {
        let text = fs::read_to_string(&manifest).expect("a readable manifest");
        let value: toml::Value = text.parse().expect("a manifest that is valid TOML");
        collect_dependency_tables(&value, &mut names);
        if let Some(targets) = value.get("target").and_then(toml::Value::as_table) {
            for target in targets.values() {
                collect_dependency_tables(target, &mut names);
            }
        }
    }
    names
}

fn collect_dependency_tables(value: &toml::Value, names: &mut BTreeSet<String>) {
    for table in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(deps) = value.get(table).and_then(toml::Value::as_table) else {
            continue;
        };
        // allow:clone: the set owns its names, outliving the parsed document
        names.extend(deps.keys().cloned());
    }
}

/// The decisions-log section of `ARCHITECTURE.md`: every line after its heading
/// and before the next top-level heading.
fn decisions_log() -> Vec<String> {
    let text = fs::read_to_string(repo_root().join("ARCHITECTURE.md"))
        .expect("ARCHITECTURE.md is readable");
    text.lines()
        // the working tree may be CRLF; a trailing CR is not content
        .map(str::trim_end)
        .skip_while(|line| !line.starts_with("## Decisions log"))
        .skip(1)
        .take_while(|line| !line.starts_with("## "))
        .map(str::to_owned)
        .collect()
}

/// Crate names priced by the decisions log: the backticked tokens in the
/// Decision cell of any row whose decision opens with the dependency marker.
fn priced_dependencies() -> BTreeSet<String> {
    let mut priced = BTreeSet::new();
    for line in decisions_log() {
        let Some(decision) = decision_cell(&line) else {
            continue;
        };
        let marker = decision.trim_start_matches(['*', ' ']);
        if marker.starts_with("Dependency:") || marker.starts_with("Dependencies:") {
            priced.extend(backticked_tokens(decision));
        }
    }
    priced
}

/// The **Decision** cell of a decisions-log table row — the second column — or
/// `None` for a line that is not a row.
fn decision_cell(line: &str) -> Option<&str> {
    line.starts_with('|')
        .then(|| line.split('|').nth(2).map(str::trim))
        .flatten()
}

/// Every backtick-delimited token in a line. Exact tokens, never substrings, so
/// `serde` is not matched by `serde_yaml` or by a `#[serde(default)]` mention.
fn backticked_tokens(line: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find(TICK) {
        let after = &rest[open + 1..];
        let Some(close) = after.find(TICK) else { break };
        tokens.push(after[..close].to_owned());
        rest = &after[close + 1..];
    }
    tokens
}

#[test]
fn every_dependency_is_priced_in_the_decisions_log() {
    let priced = priced_dependencies();
    let exempt: BTreeSet<String> = GRANDFATHERED.iter().map(|s| (*s).to_owned()).collect();
    let unpriced: Vec<String> = declared_dependencies()
        .into_iter()
        .filter(|name| !priced.contains(name) && !exempt.contains(name))
        .collect();

    assert!(
        unpriced.is_empty(),
        "dependencies with no entry in the ARCHITECTURE decisions log: {}\n\n\
         A dependency is a decision that arrives as one line in a manifest. Add a row to \
         the log: what it is for, why this one, and what was refused — including writing \
         it by hand, with a rough size, when that was a live option. Feature flags are \
         part of the price (what the default set drags in, and why it is not wanted); \
         dev-only is part of the price and lowers it. [R:price-every-dependency]\n\n\
         The row must say Dependency: (or Dependencies:) and name the crate in backticks, \
         or this gate cannot see it.",
        unpriced.join(", "),
    );
}

#[test]
fn no_grandfathered_dependency_is_already_priced() {
    let priced = priced_dependencies();
    let redundant: Vec<&str> = GRANDFATHERED
        .iter()
        .copied()
        .filter(|name| priced.contains(*name))
        .collect();

    assert!(
        redundant.is_empty(),
        "these dependencies now have a decisions-log entry and must be removed from \
         GRANDFATHERED in this file: {}\n\n\
         The exemption list only ever shrinks. Left in place, it would go on excusing a \
         dependency whose entry someone later deletes.",
        redundant.join(", "),
    );
}

#[test]
fn no_grandfathered_dependency_has_left_the_manifest() {
    let declared = declared_dependencies();
    let stale: Vec<&str> = GRANDFATHERED
        .iter()
        .copied()
        .filter(|name| !declared.contains(*name))
        .collect();

    assert!(
        stale.is_empty(),
        "these names are exempted but are no longer a dependency of anything; remove them \
         from GRANDFATHERED in this file: {}\n\n\
         A stale exemption is a standing permission to re-add the crate unpriced.",
        stale.join(", "),
    );
}

#[test]
fn the_decisions_log_is_where_this_gate_thinks_it_is() {
    let rows = decisions_log()
        .iter()
        .filter(|line| line.starts_with('|'))
        .count();
    assert!(
        rows >= 5,
        "the `## Decisions log` section of ARCHITECTURE.md is missing, renamed, or has no \
         table rows (found {rows}). That silently empties the priced set, and while every \
         dependency is still grandfathered this gate would keep passing while measuring \
         nothing. [R:guarantee-needs-a-reader]"
    );
}
