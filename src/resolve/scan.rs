//! Reading source text for the facts resolution needs: does a function exist,
//! is it a test the harness runs, does an invoker call something.
//!
//! **Scope, stated because it is narrower than "parses Rust"**
//! (`[R:measure-the-claim-not-a-subset]`). These are line readers, not
//! parsers. Line comments are stripped before anything is matched — a test
//! named only in a comment must not satisfy a claim that it exists, and a call
//! that has been commented out is not a call. Block comments (`/* */`) are not
//! stripped, so a definition inside one still reads as present. A Rust test
//! file is not checked for being reachable through a `mod` declaration; a file
//! nothing declares is never compiled, and this module will still call it
//! wired. Both gaps are carried in `TODO.md`.

/// Rust source with `//` comments removed, one entry per line.
pub(super) fn rust_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(|line| match line.find("//") {
            Some(at) => &line[..at],
            None => line,
        })
        .collect()
}

/// Shell source with whole-line `#` comments blanked (kept as empty lines so
/// indices still mean lines). A trailing `# ...` is left in place: `#` inside
/// a word or a quoted string is not a comment, and telling those apart needs a
/// shell parser.
pub(super) fn shell_lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(|line| {
            if line.trim_start().starts_with('#') {
                ""
            } else {
                line
            }
        })
        .collect()
}

/// A character that can be part of a file or function name, for boundary
/// tests: `gate.sh` must not match inside `gate.sh.bak` or `no-gate.sh`.
fn is_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.'
}

/// Whether `token` appears in `text` as a whole name.
pub(super) fn names(text: &str, token: &str) -> bool {
    text.match_indices(token).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + token.len()..].chars().next();
        !before.is_some_and(is_name_char) && !after.is_some_and(is_name_char)
    })
}

/// What the attributes on a Rust function say about whether the test harness
/// runs it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TestMark {
    /// `#[test]` (or a `…::test]` such as `#[tokio::test]`), and not ignored.
    Runs,
    /// A test, but `#[ignore]`d: the default run skips it.
    Ignored,
    /// No test attribute: an ordinary function.
    NotATest,
}

fn is_test_attribute(line: &str) -> bool {
    line.contains("#[test]") || line.contains("::test]")
}

/// Find `fn <name>` in Rust lines and read its attributes. `None` when the
/// function is not defined there.
pub(super) fn rust_fn(lines: &[&str], name: &str) -> Option<TestMark> {
    let at = lines.iter().position(|line| defines_rust_fn(line, name))?;
    // The definition line itself may carry the attribute (`#[test] fn x()`),
    // and so may every attribute or blank line directly above it.
    let above = lines[..at].iter().rev().take_while(|line| {
        let t = line.trim();
        t.is_empty() || t.starts_with("#[")
    });
    let attributes: Vec<&str> = std::iter::once(&lines[at]).chain(above).copied().collect();
    let is_test = attributes.iter().any(|a| is_test_attribute(a));
    let ignored = attributes.iter().any(|a| a.contains("#[ignore"));
    Some(match (is_test, ignored) {
        (true, false) => TestMark::Runs,
        (true, true) => TestMark::Ignored,
        (false, _) => TestMark::NotATest,
    })
}

fn defines_rust_fn(line: &str, name: &str) -> bool {
    line.match_indices("fn ").any(|(at, _)| {
        let keyword_bounded = !line[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_ascii_alphanumeric() || c == '_');
        keyword_bounded
            && line[at + 3..]
                .trim_start()
                .strip_prefix(name)
                .and_then(|after| after.chars().next())
                .is_some_and(|c| c == '(' || c == '<' || c.is_whitespace())
    })
}

/// Whether Rust lines hold at least one test the harness runs by default:
/// a test attribute whose attribute group carries no `#[ignore]`.
pub(super) fn has_running_test(lines: &[&str]) -> bool {
    lines.iter().enumerate().any(|(at, line)| {
        if !is_test_attribute(line) {
            return false;
        }
        let group_above = lines[..at]
            .iter()
            .rev()
            .take_while(|l| l.trim().starts_with("#["));
        let group_below = lines[at + 1..]
            .iter()
            .take_while(|l| l.trim().starts_with("#["));
        !std::iter::once(line)
            .chain(group_above)
            .chain(group_below)
            .any(|l| l.contains("#[ignore"))
    })
}

/// Whether a shell line defines function `name`: `name()`, `name ()`, or
/// `function name`.
fn defines_shell_fn(line: &str, name: &str) -> bool {
    let t = line.trim_start();
    if let Some(rest) = t.strip_prefix("function ") {
        return rest
            .trim_start()
            .strip_prefix(name)
            .is_some_and(|after| after.is_empty() || after.starts_with(['(', ' ', '{', '\t']));
    }
    t.strip_prefix(name)
        .is_some_and(|after| after.trim_start().starts_with("()"))
}

/// Whether shell lines define function `name`.
pub(super) fn shell_fn(lines: &[&str], name: &str) -> bool {
    lines.iter().any(|line| defines_shell_fn(line, name))
}

/// Whether shell lines reference `token` as a whole name on some line that is
/// not the definition of a function called `token` — a call, not the thing
/// being called.
pub(super) fn shell_invokes(lines: &[&str], token: &str) -> bool {
    lines
        .iter()
        .any(|line| !defines_shell_fn(line, token) && names(line, token))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_test_attribute_on_the_definition_line_is_read() {
        let src = "#[test] fn a_case() {}\n";
        assert_eq!(rust_fn(&rust_lines(src), "a_case"), Some(TestMark::Runs));
    }

    #[test]
    fn a_longer_name_sharing_the_prefix_is_a_different_function() {
        let src = "#[test]\nfn a_case_two() {}\n";
        assert_eq!(rust_fn(&rust_lines(src), "a_case"), None);
    }

    #[test]
    fn an_ignored_test_does_not_make_its_file_run_a_test() {
        assert!(!has_running_test(&rust_lines(
            "#[test]\n#[ignore]\nfn a() {}\n"
        )));
        assert!(!has_running_test(&rust_lines(
            "#[ignore]\n#[test]\nfn a() {}\n"
        )));
        assert!(has_running_test(&rust_lines("#[test]\nfn a() {}\n")));
    }

    #[test]
    fn every_shell_definition_form_is_a_definition_and_not_a_call() {
        for src in [
            "f() {\n}\n",
            "f () {\n}\n",
            "function f {\n}\n",
            "function f() {\n}\n",
        ] {
            let lines = shell_lines(src);
            assert!(shell_fn(&lines, "f"), "{src}");
            assert!(!shell_invokes(&lines, "f"), "{src}");
        }
    }

    #[test]
    fn a_file_name_is_matched_whole() {
        assert!(names("bash scripts/gate.sh --x", "gate.sh"));
        assert!(names("\"$root/gate.sh\"", "gate.sh"));
        assert!(!names("bash scripts/gate.sh.bak", "gate.sh"));
        assert!(!names("bash scripts/no-gate.sh", "gate.sh"));
    }
}
