//! A decision on a domain enum is an exhaustive policy method, never a
//! `matches!` at the call site.
//!
//! **Why this exists.** On 2026-09-16 `Status::Partial` shipped and was
//! immediately found to be silently excluded from one lint finding and silently
//! flagged by another, because both were decided by
//! `matches!(status, Status::Active)`. Adding a variant does not break a
//! `matches!`, an `if let`, or a named wildcard arm — the new variant just takes
//! the other branch, and no tool anywhere reports it. A sweep that day found
//! nine such decisions across four enums, five of which would have answered
//! *permissively*. All nine were converted into policy methods that match with
//! no catch-all, so the compiler now blocks a new variant at each.
//!
//! That fixed the nine. It said nothing about the tenth, written tomorrow. This
//! is the tenth's guard.
//!
//! **What it is deliberately not.** A blanket ban on `matches!` over these enums
//! would also ban the legitimate identity test (`VerifyStatus::is_clean` asks
//! "is this the `Ok` variant", which is not a policy) and every
//! `assert!(matches!(..))` in the suite — and a gate that must be muted for
//! correct code is a muted gate. So the ban is narrowed twice:
//!
//! * **Test code is exempt.** A test asserting a specific variant is correct.
//! * **An enum's own module is exempt.** Whoever adds a variant is editing that
//!   file and is looking straight at the variant list; the danger is the
//!   decision taken three modules away by someone who will never see the edit.
//!
//! **Scope, stated rather than implied** (`[R:measure-the-claim-not-a-subset]`).
//! Three textual shapes are caught: `matches!`, `if let`, and `let … else`. A
//! wildcard arm inside a `match` — `_ =>` or the named form `other =>` that a
//! grep for `_ =>` misses — is **not** caught, because knowing what is being
//! matched needs a parser, not a substring. One such arm existed in `poke.rs`
//! and only the compiler found it. This guard is therefore a floor, not a
//! ceiling, and the residual gap is carried in `TODO.md`.
//!
//! The detector reads `src/` and lives in `tests/`, so it never scans its own
//! list of banned names (`[R:detector-excludes-own-definitions]`).

use std::fs;
use std::path::{Path, PathBuf};

/// Each domain enum, and the file that defines it — which is the one place a
/// `matches!` on it is allowed outside a test.
///
/// The policy enums are here too (`Emittability`, `ProseCoverage`, `Provenance`,
/// `RecurrenceRole`, `Federation`): they are the answers, and a call site that
/// re-decides one by pattern-matching has reintroduced exactly the problem the
/// policy was built to remove.
const DOMAIN_ENUMS: &[(&str, &str)] = &[
    ("Status", "rule/status.rs"),
    ("Emittability", "rule/status.rs"),
    ("ProseCoverage", "rule/status.rs"),
    ("Home", "rule/home.rs"),
    ("Federation", "rule/home.rs"),
    ("Authority", "rule/authority.rs"),
    ("Provenance", "rule/authority.rs"),
    ("Origin", "rule/origin.rs"),
    ("RecurrenceRole", "rule/origin.rs"),
    ("ControlKind", "rule/control.rs"),
];

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("a readable source directory") {
        let path = entry.expect("a readable directory entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// The file's production half: everything before its `#[cfg(test)]` module.
///
/// The precondition — exactly one such marker per file, running to the end — is
/// **asserted rather than assumed**, because if a file ever grows a second test
/// module mid-file this truncation would silently stop scanning real code and
/// the gate would go quietly green over it.
fn production_half(path: &Path, code: &str) -> String {
    let markers = code.matches("#[cfg(test)]").count();
    assert!(
        markers <= 1,
        "{}: {markers} `#[cfg(test)]` markers. This gate truncates at the first \
         one and assumes the rest of the file is test code; with two, it would \
         stop scanning production code and pass without saying so. Split the \
         file, or teach this function about module boundaries",
        path.display()
    );
    let production = match code.find("#[cfg(test)]") {
        Some(at) => &code[..at],
        None => code,
    };
    strip_line_comments(production)
}

/// Blank out `//` line comments.
///
/// A doc comment quoting the banned pattern is not a violation — this repository
/// has several, including the ones explaining *why* the pattern is banned. Over-
/// stripping (a `//` inside a string literal, say) can only hide a violation,
/// never invent one, so the failure direction is a false negative and the gate
/// never cries wolf.
fn strip_line_comments(code: &str) -> String {
    code.lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether `haystack` names `ident` as a whole identifier rather than as the
/// tail of a longer one.
///
/// `VerifyStatus::Ok` contains `Status::`, and the first draft of this gate
/// reported `fsio.rs` for it — a false positive against a genuine identity test,
/// which is the failure that gets a gate muted. A preceding character that could
/// belong to an identifier means this is a different type.
fn names_ident(haystack: &str, ident: &str) -> bool {
    let mut from = 0;
    while let Some(at) = haystack[from..].find(ident) {
        let start = from + at;
        let preceded_by_ident = haystack[..start]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
        if !preceded_by_ident {
            return true;
        }
        from = start + ident.len();
    }
    false
}

/// Whether `code` decides something about `enum_name` by pattern-matching.
///
/// `matches!` is found by locating the macro and looking ahead a short window,
/// because the real ones wrap across lines. `if let` and `let … else` are direct.
fn decisions_on(code: &str, enum_name: &str) -> Vec<String> {
    let variant = format!("{enum_name}::");
    let mut found = Vec::new();

    let mut from = 0;
    while let Some(at) = code[from..].find("matches!") {
        let start = from + at;
        let end = (start + 240).min(code.len());
        let window = &code[start..end];
        if names_ident(window, &variant) {
            found.push(format!("matches!(.., {variant}..)"));
        }
        from = start + "matches!".len();
    }

    if names_ident(code, &format!("if let {variant}")) {
        found.push(format!("if let {variant}.."));
    }
    if names_ident(code, &format!("let {variant}")) && code.contains("else {") {
        found.push(format!("let {variant}.. = .. else"));
    }
    found
}

#[test]
fn no_call_site_decides_a_domain_enum_by_pattern_matching() {
    let root = src_root();
    let mut files = Vec::new();
    rust_files(&root, &mut files);
    assert!(
        files.len() > 10,
        "expected to find the source tree, found {} file(s) — a gate that \
         scans nothing passes",
        files.len()
    );

    let mut violations = Vec::new();
    for path in &files {
        let code = fs::read_to_string(path).expect("a readable source file");
        let production = production_half(path, &code);
        let relative = path
            .strip_prefix(&root)
            .expect("every file is under src/")
            .to_string_lossy()
            .replace('\\', "/");

        for (enum_name, home) in DOMAIN_ENUMS {
            if relative == *home {
                continue; // the enum's own module may look at its own variants
            }
            for shape in decisions_on(&production, enum_name) {
                violations.push(format!("  src/{relative}: {shape}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "a domain enum is being decided by pattern-matching at a call site, \
         which answers for a NEW VARIANT SILENTLY:\n{}\n\nMove the decision into \
         an exhaustive policy method on the enum — `Status::prose_coverage`, \
         `Authority::provenance` and `Origin::recurrence_role` are the shape — \
         so the next variant is a compile error at the one place that must \
         decide. If this is a genuine identity test rather than a policy, it \
         belongs in the enum's own module beside the variant list.",
        violations.join("\n")
    );
}
