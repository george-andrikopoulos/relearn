//! The pack READMEs count their own contents, and this reads the counts.
//!
//! `copilot-pack/README.md` says *"52 rules, 746 lines"*; `claude-pack/README.md`
//! says *"27 rules"*, *"18 rules"*, and three more. Those numbers were written
//! by hand and **nothing checked them** — the federation programme names it as
//! the standing exposure, twice, and it is `[R:guarantee-needs-a-reader]` in the
//! repository whose own corpus carries that rule: a claim in prose with nothing
//! reading the state it asserts.
//!
//! **Each check owns its own denominator, and the two directions are separate:**
//!
//! * every **claim** resolves to a file in its pack and matches what that file
//!   actually contains, so a number cannot go stale while the tree moves;
//! * every **file** in a pack is the subject of at least one claim, so a sixth
//!   skill layer cannot arrive with the README silently unchanged — which is the
//!   drift that actually happens, and the direction a claims-only check misses.
//!
//! What this does **not** check is that the packs match the rules: `relearn
//! verify --targets copilot --out copilot-pack` holds that, and CI runs it as a
//! second output root. The two compose — README ↔ pack ↔ rules — and neither
//! needs to know how the other measures.
//!
//! Scope, stated because `[R:measure-the-claim-not-a-subset]` is what goes wrong
//! here: a **rule** is a `## ` heading ending in its `[R:tag]`, which is the form
//! every emitter writes and which no prose heading can accidentally match; a
//! **line** is `str::lines()`, which strips a trailing `\r`, so the count is the
//! same on a CRLF checkout as on an LF one (`[R:xplat-fixtures]`).

use std::fs;
use std::path::{Path, PathBuf};

/// The packs that carry a hand-written inventory of themselves.
const PACKS: [&str; 2] = ["copilot-pack", "claude-pack"];

/// What a number in a README is counting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unit {
    Rules,
    Lines,
}

impl Unit {
    fn label(self) -> &'static str {
        match self {
            Unit::Rules => "rules",
            Unit::Lines => "lines",
        }
    }

    /// What the file actually holds.
    fn measure(self, text: &str) -> usize {
        match self {
            // The form every emitter writes: `## Title [R:tag]`. Matching the
            // tag rather than the bare `## ` means a prose heading in a
            // hand-authored README section can never inflate a rule count.
            Unit::Rules => text
                .lines()
                .filter(|line| line.starts_with("## ") && ends_with_tag(line))
                .count(),
            // `lines()` strips a trailing `\r`, so a CRLF checkout counts the
            // same as an LF one.
            Unit::Lines => text.lines().count(),
        }
    }
}

/// One count a README claims about a file in its own pack.
#[derive(Debug)]
struct Claim {
    readme: PathBuf,
    source_line: usize,
    /// The path token as the README writes it, e.g. `global/SKILL.md`.
    token: String,
    unit: Unit,
    claimed: usize,
}

/// `## Something [R:some-tag]`, allowing for a trailing `\r` already stripped by
/// `lines()`.
fn ends_with_tag(line: &str) -> bool {
    let trimmed = line.trim_end();
    let Some(rest) = trimmed.strip_suffix(']') else {
        return false;
    };
    let Some((_, tag)) = rest.rsplit_once("[R:") else {
        return false;
    };
    !tag.is_empty()
        && tag
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Every `<number> rules` / `<number> lines` phrase on a line.
fn counts_on(line: &str) -> Vec<(Unit, usize)> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    tokens
        .windows(2)
        .filter_map(|pair| {
            let n: usize = pair[0].parse().ok()?;
            // `rules,` and `lines —` both appear: keep the letters only.
            let word: String = pair[1].chars().filter(char::is_ascii_alphabetic).collect();
            match word.as_str() {
                "rules" | "rule" => Some((Unit::Rules, n)),
                "lines" | "line" => Some((Unit::Lines, n)),
                _ => None,
            }
        })
        .collect()
}

/// The first `*.md` path token on a line, if there is one.
fn file_token(line: &str) -> Option<&str> {
    line.split_whitespace().find(|t| t.ends_with(".md"))
}

/// Every claim a README makes: a line naming a `.md` file **and** a count.
fn claims_in(readme: &Path) -> Vec<Claim> {
    let text = fs::read_to_string(readme).expect("a readable README");
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let token = file_token(line)?;
            let counts = counts_on(line);
            (!counts.is_empty()).then(|| {
                counts.into_iter().map(move |(unit, claimed)| Claim {
                    readme: readme.to_path_buf(),
                    source_line: i + 1,
                    token: token.to_owned(),
                    unit,
                    claimed,
                })
            })
        })
        .flatten()
        .collect()
}

/// Every `*.md` file under `dir`, recursively.
fn markdown_under(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(next) = stack.pop() {
        for entry in fs::read_dir(&next).expect("a readable pack directory") {
            let path = entry.expect("a readable entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// The one file in `files` whose path ends with the README's token.
///
/// Resolved by **suffix match rather than by reading the ASCII tree**: the tree
/// drawing is decoration, and a check that parsed it would break on a redrawn
/// box character while the numbers it exists to guard stayed wrong. Ambiguity is
/// an error rather than a first match — two files ending in `SKILL.md` under one
/// token would mean the claim names neither.
fn resolve<'a>(token: &str, files: &'a [PathBuf]) -> Result<&'a PathBuf, String> {
    let token = token.replace('\\', "/");
    let matches: Vec<&PathBuf> = files
        .iter()
        .filter(|path| {
            let as_posix = path.to_string_lossy().replace('\\', "/");
            as_posix.ends_with(&format!("/{token}")) || as_posix == token
        })
        .collect();
    match matches.as_slice() {
        [one] => Ok(one),
        [] => Err(format!("no file in the pack is named {token}")),
        several => Err(format!("{} files match {token}", several.len())),
    }
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every number a pack README states about its own contents is the number that
/// file actually holds.
#[test]
fn every_count_a_pack_readme_claims_is_the_one_its_file_holds() {
    let root = root();
    let mut checked = 0;

    for pack in PACKS {
        let dir = root.join(pack);
        let files = markdown_under(&dir);
        let readme = dir.join("README.md");
        let claims = claims_in(&readme);

        assert!(
            !claims.is_empty(),
            "{pack}/README.md states no counts at all — either the inventory was \
             removed, or this parser has stopped recognising it, and a check that \
             measures nothing passes"
        );

        for claim in claims {
            let path = resolve(&claim.token, &files).unwrap_or_else(|why| {
                panic!(
                    "{}:{} claims {} {} about {}, but {why}",
                    claim.readme.display(),
                    claim.source_line,
                    claim.claimed,
                    claim.unit.label(),
                    claim.token
                )
            });
            let text = fs::read_to_string(path).expect("a readable pack file");
            let actual = claim.unit.measure(&text);
            assert_eq!(
                actual,
                claim.claimed,
                "{}:{} says {} has {} {}, and it has {actual}. \
                 The number is hand-written; the file is generated — fix the README",
                claim.readme.display(),
                claim.source_line,
                claim.token,
                claim.claimed,
                claim.unit.label(),
            );
            checked += 1;
        }
    }

    // Six claims exist today: the copilot file's rules and lines, and one rule
    // count per skill layer. The floor is deliberately below that — it asserts
    // the parser found a real inventory rather than pinning a number that would
    // have to be edited every time a layer is added, which is the maintenance
    // burden that makes a check get deleted.
    assert!(
        checked >= 5,
        "only {checked} claim(s) checked across {} pack(s) — too few to mean anything",
        PACKS.len()
    );
}

/// **The other direction, and the one that catches what actually drifts.** A
/// sixth home layer arrives, `relearn build --out claude-pack` writes its
/// `SKILL.md`, `verify` is green because the pack matches the rules — and the
/// README still lists five. Every claim would still be correct; the inventory
/// would be a lie by omission.
#[test]
fn every_file_in_a_pack_is_counted_by_its_readme() {
    let root = root();

    for pack in PACKS {
        let dir = root.join(pack);
        let readme = dir.join("README.md");
        let claimed: Vec<String> = claims_in(&readme)
            .into_iter()
            .map(|claim| claim.token)
            .collect();

        for path in markdown_under(&dir) {
            if path == readme {
                continue; // the README does not count itself
            }
            let as_posix = path.to_string_lossy().replace('\\', "/");
            assert!(
                claimed
                    .iter()
                    .any(|token| as_posix.ends_with(&format!("/{token}"))),
                "{} is in {pack} and {}/README.md counts nothing about it — the \
                 inventory is missing a line",
                path.display(),
                pack
            );
        }
    }
}
