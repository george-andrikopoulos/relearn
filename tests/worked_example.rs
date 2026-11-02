//! Every emitted block `docs/worked-example.md` shows is what `relearn build`
//! actually writes for that rule, byte for byte.
//!
//! **Why this exists.** The document says "Every block above is real output"
//! and shows five generated files, footers and `sha256` included. Until this
//! test, only its rule-file sample was checked (`tests/doc_samples.rs`): an
//! emitter change, or a change to the rule's body, would have left five stale
//! blocks presented as the tool's output, under a sentence saying they were.
//!
//! **Through the production path** (`[R:verify-through-production-path]`).
//! This runs the commands the document's "Reproducing this" section gives a
//! reader — the rule copied alone into a directory, then `relearn build
//! --rules <dir> --out <dir>` on the real binary with its default targets — not
//! the emitters called in-process, which would skip target selection.
//!
//! **Scope, both directions** (`[R:measure-the-claim-not-a-subset]`). Every
//! block the document places under a `### … \`<path>\`` heading must equal the
//! file at that path, and every file the build writes must be shown. A new
//! default target therefore fails here until the document shows it.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const DOCUMENT: &str = "docs/worked-example.md";
const RULE_FILE: &str = "rules/order-by-explicit-rank.md";

/// The build line the document gives a reader, verbatim. The test runs these
/// arguments and asserts the document still shows this line, so the two can
/// only change together: a reproduction the test does not run is one more
/// claim nobody checks.
const SHOWN_BUILD: &str =
    "relearn build --rules /tmp/one-rule --out /tmp/example --rules-layer project=relearn";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// One emitted file as the document presents it.
struct Shown {
    heading_line: usize,
    path: String,
    content: String,
}

/// Every `### ` heading that names a path in backticks, with the first fenced
/// block after it. A heading with no backticked path is prose, not a claim
/// about a file. Carriage returns are stripped first: the document is not
/// pinned to LF, so a Windows checkout hands this CRLF (`[R:xplat-fixtures]`).
fn shown_files(text: &str) -> Vec<Shown> {
    let lines: Vec<&str> = text.lines().map(|l| l.trim_end_matches('\r')).collect();
    let mut found = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let Some(heading) = lines[i].strip_prefix("### ") else {
            i += 1;
            continue;
        };
        let Some(path) = heading.split('`').nth(1) else {
            i += 1;
            continue;
        };
        let Some(open) = (i + 1..lines.len()).find(|&j| lines[j].starts_with("```")) else {
            break;
        };
        let Some(close) = (open + 1..lines.len()).find(|&j| lines[j].starts_with("```")) else {
            break;
        };
        found.push(Shown {
            heading_line: i + 1,
            path: path.to_owned(),
            content: lines[open + 1..close].join("\n"),
        });
        i = close + 1;
    }
    found
}

/// Every file under `dir`, as `/`-separated paths relative to it.
fn files_under(dir: &Path, base: &Path, out: &mut BTreeSet<String>) {
    for entry in fs::read_dir(dir).expect("read the build output") {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            files_under(&path, base, out);
        } else {
            let relative = path.strip_prefix(base).expect("under the output root");
            let parts: Vec<String> = relative
                .components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect();
            out.insert(parts.join("/"));
        }
    }
}

/// Run the document's reproduction: the one rule alone, built with the
/// default targets. Returns the output directory.
fn build_the_one_rule(scratch: &Path) -> PathBuf {
    let rules = scratch.join("one-rule");
    let out = scratch.join("example");
    fs::create_dir_all(&rules).expect("create the one-rule directory");
    fs::copy(
        repo_root().join(RULE_FILE),
        rules.join("order-by-explicit-rank.md"),
    )
    .expect("copy the rule");
    let status = Command::new(env!("CARGO_BIN_EXE_relearn"))
        .arg("build")
        .arg("--rules")
        .arg(&rules)
        .arg("--out")
        .arg(&out)
        .args(["--rules-layer", "project=relearn"])
        .output()
        .expect("run the relearn binary");
    assert!(
        status.status.success(),
        "relearn build failed on the worked example's rule: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    out
}

/// **The assertion.**
#[test]
fn every_block_the_worked_example_shows_is_what_relearn_build_writes() {
    let text = fs::read_to_string(repo_root().join(DOCUMENT)).expect("read the worked example");
    let shown = shown_files(&text);
    let scratch = tempfile::tempdir().expect("a scratch directory");
    let out = build_the_one_rule(scratch.path());

    for block in &shown {
        let written = fs::read_to_string(out.join(&block.path)).unwrap_or_else(|e| {
            panic!(
                "{DOCUMENT}:{} shows `{}` as build output, and the build wrote no such file: {e}",
                block.heading_line, block.path
            )
        });
        let written = written.replace('\r', "");
        assert_eq!(
            block.content,
            written.trim_end_matches('\n'),
            "{DOCUMENT}:{} shows `{}`, and it is not what `relearn build` writes for \
             {RULE_FILE} today. Re-run the document's \"Reproducing this\" commands and \
             paste the file in.",
            block.heading_line,
            block.path
        );
    }

    let mut written = BTreeSet::new();
    files_under(&out, &out, &mut written);
    let claimed: BTreeSet<String> = shown.iter().map(|b| b.path.clone()).collect(); // allow:clone: the set outlives the borrow of `shown` it is compared beside
    assert_eq!(
        claimed, written,
        "the files {DOCUMENT} shows are not the files the build writes: a target was \
         added or removed, and the document should follow it"
    );
}

/// The document shows the build this file runs. Found 2026-11-02: the shown
/// command (`--out /tmp/example`, no `--rules-layer`) wrote four files while
/// the document presented five, because the project layer is named for the
/// output directory.
#[test]
fn the_document_shows_the_build_this_test_runs() {
    let text = fs::read_to_string(repo_root().join(DOCUMENT)).expect("read the worked example");
    assert!(
        text.lines()
            .any(|l| l.trim_end_matches('\r') == SHOWN_BUILD),
        "{DOCUMENT} no longer shows `{SHOWN_BUILD}` on a line of its own. Change the \
         document and this test's arguments together."
    );
}

/// The extractor, pinned on both line endings: on CRLF a naive one finds the
/// headings and no fences, and the set comparison above would then fail for
/// the wrong reason — or, with an empty build, pass for one.
#[test]
fn the_extractor_reads_a_heading_and_its_block_whatever_the_line_endings() {
    let lf = "### A file — `dir/a.md`\n\nprose\n\n```markdown\nline one\nline two\n```\n\
              ### Not a file\n\n```\nignored\n```\n";
    for text in [lf.to_owned(), lf.replace('\n', "\r\n")] {
        let shown = shown_files(&text);
        assert_eq!(shown.len(), 1, "{text:?}");
        assert_eq!(shown[0].path, "dir/a.md");
        assert_eq!(shown[0].content, "line one\nline two");
    }
}
