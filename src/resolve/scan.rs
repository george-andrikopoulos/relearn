//! Reading source text for the facts resolution needs: does a function exist,
//! is it a test the harness runs, does an invoker call something.
//!
//! **Scope, stated because it is narrower than "parses Rust"**
//! (`[R:measure-the-claim-not-a-subset]`). These are line readers, not
//! parsers. Every Rust comment, line and block (nested, as Rust nests them),
//! is removed by [`RustCode::strip`] before anything is matched: a test named
//! only in a comment must not satisfy a claim that it exists, and a definition
//! or call that has been commented out is not one. The stripper tracks string,
//! raw-string and char literals so that `"**/*.rs"` or `"http://"` is never
//! read as the start of a comment — the failure that would turn a real test
//! into a missing one. A Rust test file is not checked for being reachable
//! through a `mod` declaration; a file nothing declares is never compiled, and
//! this module will still call it wired. That gap is carried in `TODO.md`.

/// Rust source with every comment removed — the text the compiler reads as
/// code. The only constructor is [`RustCode::strip`], so a line reader that
/// takes [`RustCode::lines`] cannot be handed source that still has comments.
pub(super) struct RustCode(String);

/// Where the stripper is in the source.
#[derive(Clone, Copy)]
enum Lex {
    Code,
    LineComment,
    /// Inside `/* */`, at this nesting depth (at least 1).
    Block(u32),
    /// Inside `"…"`.
    Str,
    /// Inside `r#…"…"#…`, closed by `"` and this many `#`.
    RawStr(usize),
}

impl RustCode {
    /// Remove every comment from Rust source. Each comment becomes one space,
    /// and every newline inside it is kept, so line indices still mean lines.
    #[must_use]
    pub(super) fn strip(text: &str) -> Self {
        let chars: Vec<char> = text.chars().collect();
        let mut out = String::with_capacity(text.len());
        let mut state = Lex::Code;
        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];
            let next = chars.get(i + 1).copied();
            state = match state {
                Lex::Code => match (c, next) {
                    ('/', Some('/')) => {
                        out.push(' ');
                        i += 1;
                        Lex::LineComment
                    }
                    ('/', Some('*')) => {
                        out.push(' ');
                        i += 1;
                        Lex::Block(1)
                    }
                    ('"', _) => {
                        out.push(c);
                        Lex::Str
                    }
                    ('r', _) => match raw_string_open(&chars, i) {
                        Some(hashes) => {
                            out.extend(&chars[i..=i + hashes + 1]);
                            i += hashes + 1;
                            Lex::RawStr(hashes)
                        }
                        None => {
                            out.push(c);
                            Lex::Code
                        }
                    },
                    ('\'', _) => {
                        let end = char_literal_end(&chars, i).unwrap_or(i);
                        out.extend(&chars[i..=end]);
                        i = end;
                        Lex::Code
                    }
                    _ => {
                        out.push(c);
                        Lex::Code
                    }
                },
                Lex::LineComment => {
                    if c == '\n' {
                        out.push('\n');
                        Lex::Code
                    } else {
                        Lex::LineComment
                    }
                }
                Lex::Block(depth) => match (c, next) {
                    ('/', Some('*')) => {
                        i += 1;
                        Lex::Block(depth + 1)
                    }
                    ('*', Some('/')) => {
                        i += 1;
                        if depth == 1 {
                            Lex::Code
                        } else {
                            Lex::Block(depth - 1)
                        }
                    }
                    ('\n' | '\r', _) => {
                        out.push(c);
                        Lex::Block(depth)
                    }
                    _ => Lex::Block(depth),
                },
                Lex::Str => {
                    out.push(c);
                    match (c, next) {
                        ('\\', Some(escaped)) => {
                            out.push(escaped);
                            i += 1;
                            Lex::Str
                        }
                        ('"', _) => Lex::Code,
                        _ => Lex::Str,
                    }
                }
                Lex::RawStr(hashes) => {
                    out.push(c);
                    let closes = c == '"'
                        && chars.len() > i + hashes
                        && chars[i + 1..=i + hashes].iter().all(|&h| h == '#');
                    if closes {
                        out.extend(&chars[i + 1..=i + hashes]);
                        i += hashes;
                        Lex::Code
                    } else {
                        Lex::RawStr(hashes)
                    }
                }
            };
            i += 1;
        }
        Self(out)
    }

    /// The comment-free source, one entry per line.
    #[must_use]
    pub(super) fn lines(&self) -> Vec<&str> {
        self.0.lines().collect()
    }
}

/// At an `r`, whether a raw string opens here (`r"`, `r#"`, `br##"`, …), and
/// with how many `#`. An `r` that ends an identifier (`for"` is not Rust, but
/// `bar#` must not read as `r#`) opens nothing; `br` is the one prefix allowed.
fn raw_string_open(chars: &[char], at: usize) -> Option<usize> {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    let prefixed = match at.checked_sub(1).map(|p| chars[p]) {
        None => false,
        Some('b') => at.checked_sub(2).is_some_and(|p| ident(chars[p])),
        Some(p) => ident(p),
    };
    if prefixed {
        return None;
    }
    let hashes = chars[at + 1..].iter().take_while(|&&c| c == '#').count();
    (chars.get(at + 1 + hashes) == Some(&'"')).then_some(hashes)
}

/// At a `'`, the index of the closing `'` when a char literal opens here
/// (`'x'`, `'\n'`, `'\u{1F600}'`, `'"'`). `None` for a lifetime or label
/// (`'a`, `'static`, `'outer:`), which is code.
fn char_literal_end(chars: &[char], at: usize) -> Option<usize> {
    match chars.get(at + 1)? {
        '\\' => chars[at + 2..]
            .iter()
            .take(12)
            .position(|&c| c == '\'')
            .map(|p| at + 2 + p),
        _ => (chars.get(at + 2) == Some(&'\'')).then_some(at + 2),
    }
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
    use proptest::prelude::*;

    fn mark(src: &str, name: &str) -> Option<TestMark> {
        rust_fn(&RustCode::strip(src).lines(), name)
    }

    fn runs_a_test(src: &str) -> bool {
        has_running_test(&RustCode::strip(src).lines())
    }

    #[test]
    fn a_test_attribute_on_the_definition_line_is_read() {
        assert_eq!(
            mark("#[test] fn a_case() {}\n", "a_case"),
            Some(TestMark::Runs)
        );
    }

    #[test]
    fn a_longer_name_sharing_the_prefix_is_a_different_function() {
        assert_eq!(mark("#[test]\nfn a_case_two() {}\n", "a_case"), None);
    }

    #[test]
    fn an_ignored_test_does_not_make_its_file_run_a_test() {
        assert!(!runs_a_test("#[test]\n#[ignore]\nfn a() {}\n"));
        assert!(!runs_a_test("#[ignore]\n#[test]\nfn a() {}\n"));
        assert!(runs_a_test("#[test]\nfn a() {}\n"));
    }

    // TODO.md, "Block comments are not stripped": a definition inside `/* */`
    // read as present.
    #[test]
    fn a_test_inside_a_block_comment_is_not_defined() {
        assert_eq!(mark("/*\n#[test]\nfn a() {}\n*/\n", "a"), None);
        assert!(!runs_a_test("/* #[test] */ fn a() {}\n"));
    }

    #[test]
    fn block_comments_nest_as_rust_nests_them() {
        let src = "/* outer /* inner */ #[test] fn a() {} */\n#[test]\nfn b() {}\n";
        assert_eq!(mark(src, "a"), None);
        assert_eq!(mark(src, "b"), Some(TestMark::Runs));
    }

    // The failure the stripper must not introduce: a glob in a string opening
    // a comment that swallows every real test after it.
    #[test]
    fn a_comment_opener_inside_a_literal_opens_nothing() {
        for opener in [
            "let g = \"src/**/*.rs\";",
            "let u = \"http://x\";",
            "let r = r#\"/* \"quoted\" \"#;",
            "let b = br\"/*\";",
            "let e = \"\\\" /* \";",
            "let q = '\"'; let s = \"/*\";",
        ] {
            let src = format!("{opener}\n#[test]\nfn a() {{}}\n");
            assert_eq!(mark(&src, "a"), Some(TestMark::Runs), "{opener}");
        }
    }

    #[test]
    fn a_lifetime_is_code_and_not_a_char_literal() {
        let src = "fn f<'a>(x: &'a str) {} /* fn g() {} */\n#[test] fn h() {}\n";
        assert_eq!(mark(src, "f"), Some(TestMark::NotATest));
        assert_eq!(mark(src, "g"), None);
        assert_eq!(mark(src, "h"), Some(TestMark::Runs));
    }

    #[test]
    fn a_definition_after_a_closed_comment_on_the_same_line_is_read() {
        assert_eq!(
            mark("/* note */ #[test] fn a() {}\n", "a"),
            Some(TestMark::Runs)
        );
        assert_eq!(
            mark("let u = \"a//b\"; fn a() {}\n", "a"),
            Some(TestMark::NotATest)
        );
    }

    proptest! {
        // Line indices must still mean lines, whatever the input, including
        // unterminated comments, strings and raw strings.
        #[test]
        fn stripping_keeps_every_newline(src in "[ -~\n]{0,200}") {
            let stripped = RustCode::strip(&src);
            prop_assert_eq!(
                stripped.0.matches('\n').count(),
                src.matches('\n').count()
            );
        }

        // Source with no comment, string or char syntax is code, untouched.
        #[test]
        fn source_without_comment_or_literal_syntax_is_unchanged(
            src in "[a-zA-Z0-9_#\\[\\](){}<>:;,.=&!*+ \n-]{0,200}"
        ) {
            prop_assert_eq!(RustCode::strip(&src).0, src);
        }

        // Nothing inside a closed block comment reaches the code.
        #[test]
        fn a_closed_block_comment_leaves_only_whitespace(
            body in "[ -~\n]{0,200}"
                // A trailing `/` would meet the closing `*/` and open a nested
                // comment, exactly as rustc reads `/* a/*/`.
                .prop_filter("no comment delimiters", |b| {
                    !b.contains("/*") && !b.contains("*/") && !b.ends_with('/')
                })
        ) {
            let src = format!("/*{body}*/");
            prop_assert!(RustCode::strip(&src).0.trim().is_empty());
        }
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
