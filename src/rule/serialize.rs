//! Serializing a [`Rule`] back to the neutral `+++`-delimited document form —
//! the inverse of [`super::parse_document`]. `parse_document(&to_document(&r))`
//! reproduces `r` exactly, which is what makes the pipeline lossless: a rule can
//! be read, transformed, and rewritten without decay.
//!
//! This is the rule's **own** codec, not a vendor output format (that is
//! `emit`'s job) — so it does not breach the module's "know nothing about output
//! formats" rule; it round-trips the same neutral format the parser reads.
//!
//! Dates are emitted as quoted strings (matching the parser and the
//! `[R:parse-wide-then-range-check]` dogfood), and every text value is escaped
//! as a TOML basic string so quotes, backslashes, and control characters
//! survive the round trip.

use super::{Home, Rule, ScopeTag, Status};

/// Serialize a rule to its neutral document form (`+++` front-matter, then the
/// markdown body). [`super::parse_document`] parses the result back to an equal
/// [`Rule`].
#[must_use]
pub fn to_document(rule: &Rule) -> String {
    let mut out = String::from("+++\n");
    out.push_str(&kv("tag", rule.tag().as_str()));
    out.push_str(&kv("title", rule.title().as_str()));
    out.push_str(&kv("error_class", rule.error_class().as_str()));
    out.push_str(&home_line(rule.home()));
    // Written beside `home` because that is where a human reads it: home says
    // who owns the rule, `applies_to` says who loads it, and the two questions
    // belong next to each other in the file even though the constructor takes
    // them apart. Emitted **only** when non-empty, so a rule that declares no
    // audience renders exactly as it did before the field existed — the reason
    // none of the fifty-two committed rules needed editing, asserted over the
    // real corpus by `tests/corpus.rs`.
    if rule.is_scoped() {
        out.push_str(&applies_to_line(rule.applies_to()));
    }
    out.push_str(&kv("created", &rule.created().to_string()));
    out.push_str(&kv("origin", rule.origin().as_str()));
    out.push_str(&status_line(rule.status()));
    out.push_str(&kv("incident", rule.incident().as_str()));
    // Emitted **only** when there is at least one, and last, because a TOML
    // array of tables captures every key that follows it. A rule that has not
    // recurred therefore renders exactly as it did before the field existed,
    // which is what makes the format change need no migration — asserted over
    // the real corpus by `tests/corpus.rs`.
    for recurrence in rule.recurrences() {
        out.push_str("\n[[recurrence]]\n");
        out.push_str(&kv("date", &recurrence.date().to_string()));
        out.push_str(&kv("incident", recurrence.incident().as_str()));
    }
    out.push_str("+++\n\n");
    out.push_str(rule.body().as_str());
    out.push('\n');
    out
}

/// One `key = "value"` front-matter line, the value a TOML basic string.
fn kv(key: &str, value: &str) -> String {
    format!("{key} = {}\n", toml_basic_string(value))
}

/// The `home = { ... }` line, an inline table keyed by the home's kind.
fn home_line(home: &Home) -> String {
    match home {
        Home::Global => "home = { kind = \"global\" }\n".to_owned(),
        Home::Domain { name } => format!(
            "home = {{ kind = \"domain\", name = {} }}\n",
            toml_basic_string(name.as_str())
        ),
        Home::Project { path } => format!(
            "home = {{ kind = \"project\", path = {} }}\n",
            toml_basic_string(path.as_str())
        ),
    }
}

/// The `applies_to = [...]` line, a TOML array of basic strings in file order.
///
/// Order is preserved rather than sorted, for the same reason recurrences are:
/// the serializer must reproduce the file it parsed, and sorting here would
/// rewrite every scoped rule the first time a build ran.
fn applies_to_line(scopes: &[ScopeTag]) -> String {
    let list: Vec<String> = scopes
        .iter()
        .map(|s| toml_basic_string(s.as_str()))
        .collect();
    format!("applies_to = [{}]\n", list.join(", "))
}

/// The `status = { ... }` line; a graduated/atticked status carries its payload.
fn status_line(status: &Status) -> String {
    match status {
        Status::Active => "status = { kind = \"active\" }\n".to_owned(),
        Status::Graduated { to, date } => format!(
            "status = {{ kind = \"graduated\", to = {}, date = {} }}\n",
            toml_basic_string(to.as_str()),
            toml_basic_string(&date.to_string())
        ),
        Status::Attic { reason, date } => format!(
            "status = {{ kind = \"attic\", reason = {}, date = {} }}\n",
            toml_basic_string(reason.as_str()),
            toml_basic_string(&date.to_string())
        ),
    }
}

/// Render `s` as a TOML basic string (double-quoted, with the escapes TOML
/// requires) so it parses back to exactly `s`.
fn toml_basic_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::to_document;
    use crate::rule::{
        Body, Date, ErrorClass, Home, Incident, Origin, Recurrence, Rule, RuleTag, ScopeTag,
        Status, Title, parse_document,
    };

    fn rule_with_recurrences(recurrences: Vec<Recurrence>) -> Rule {
        Rule::new(
            RuleTag::parse("R:x").expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::global(),
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            Status::active(),
            Incident::parse("the triggering incident").expect("non-empty incident"),
            Body::parse("Do the thing.").expect("non-empty body"),
            recurrences,
            Vec::new(),
        )
    }

    fn recurrence(date: &str, incident: &str) -> Recurrence {
        Recurrence::new(
            Date::parse(date).expect("valid date"),
            Incident::parse(incident).expect("non-empty incident"),
        )
    }

    fn rule_with(error_class: &str, home: Home, status: Status, body: &str) -> Rule {
        Rule::new(
            RuleTag::parse("R:x").expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse(error_class).expect("non-empty error class"),
            home,
            Date::parse("2026-08-13").expect("valid date"),
            Origin::Mined,
            status,
            Incident::parse("an incident").expect("non-empty incident"),
            Body::parse(body).expect("non-empty body"),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn round_trips_a_plain_rule() {
        let r = rule_with(
            "range collapse at parse",
            Home::global(),
            Status::active(),
            "Do the thing.",
        );
        assert_eq!(parse_document(&to_document(&r)), Ok(r));
    }

    #[test]
    fn round_trips_quotes_and_backslashes_in_text() {
        // The exact hazards the escaper exists for: embedded quotes and a
        // Windows path with backslashes must survive the TOML round trip.
        let r = rule_with(
            "encoding a state as 0, -1, \"\", or T::zero()",
            Home::domain("rust").expect("non-empty domain"),
            Status::active(),
            "Prefer an enum variant over a sentinel like \"\" or a path C:\\repo.",
        );
        assert_eq!(parse_document(&to_document(&r)), Ok(r));
    }

    #[test]
    fn round_trips_graduated_status() {
        let g = rule_with(
            "superseded by a hook",
            Home::global(),
            Status::graduated(
                "hook:no-narrow-parse",
                Date::parse("2026-07-21").expect("valid date"),
            )
            .expect("non-empty destination"),
            "Body.",
        );
        assert_eq!(parse_document(&to_document(&g)), Ok(g));
    }

    #[test]
    fn round_trips_attic_status_and_project_home() {
        let date = Date::parse("2026-09-01").expect("valid date");
        let a = rule_with(
            "cold surface",
            Home::project("relearn").expect("non-empty project"),
            Status::attic("challenge-tested, no recurrence", date).expect("non-empty reason"),
            "Body.",
        );
        assert_eq!(parse_document(&to_document(&a)), Ok(a));
    }

    // The assertion that makes "no rule file needs editing" true at the unit
    // level; `tests/corpus.rs` makes it true over the real forty-six.
    #[test]
    fn a_rule_with_no_recurrences_renders_no_recurrence_table() {
        let doc = to_document(&rule_with_recurrences(Vec::new()));
        assert!(
            !doc.contains("[[recurrence]]"),
            "an unrecurred rule must render exactly as it did before the field existed:\n{doc}"
        );
    }

    #[test]
    fn round_trips_recurrences() {
        let r = rule_with_recurrences(vec![
            recurrence("2026-08-24", "Fired again; the script was never repaired."),
            recurrence("2026-08-30", "And again, with a \"quoted\" phrase."),
        ]);
        assert_eq!(parse_document(&to_document(&r)), Ok(r));
    }

    // The tables must come **after** every scalar key: a TOML array of tables
    // captures everything that follows it, so a `[[recurrence]]` emitted before
    // `incident` would swallow `incident` into the table and change the rule.
    #[test]
    fn recurrence_tables_are_emitted_after_the_scalar_fields() {
        let doc = to_document(&rule_with_recurrences(vec![recurrence(
            "2026-08-24",
            "again",
        )]));
        let table_at = doc.find("[[recurrence]]").expect("the table is emitted");
        let incident_at = doc
            .find("incident = ")
            .expect("the incident key is emitted");
        assert!(
            incident_at < table_at,
            "the scalar keys must precede the array of tables:\n{doc}"
        );
    }

    fn scoped_rule(scopes: &[&str]) -> Rule {
        Rule::new(
            RuleTag::parse("R:x").expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::domain("low-latency").expect("non-empty domain"),
            Date::parse("2026-09-13").expect("valid date"),
            Origin::Mined,
            Status::active(),
            Incident::parse("the triggering incident").expect("non-empty incident"),
            Body::parse("Do the thing.").expect("non-empty body"),
            Vec::new(),
            scopes
                .iter()
                .map(|s| ScopeTag::parse(*s).expect("valid scope"))
                .collect(),
        )
    }

    // The assertion that makes "no rule file needs editing" true for this field
    // at the unit level; `tests/corpus.rs` makes it true over the real fifty-two.
    #[test]
    fn an_unscoped_rule_renders_no_applies_to_line() {
        let doc = to_document(&scoped_rule(&[]));
        assert!(
            !doc.contains("applies_to"),
            "a rule declaring no audience must render exactly as it did before \
             the field existed:\n{doc}"
        );
    }

    #[test]
    fn round_trips_applies_to_in_file_order() {
        let r = scoped_rule(&["rust", "java"]);
        let doc = to_document(&r);
        assert!(doc.contains("applies_to = [\"rust\", \"java\"]"), "{doc}");
        assert_eq!(parse_document(&doc), Ok(r));
    }

    // Beside `home`, because that is where a reader asks the question: home says
    // who owns the rule, `applies_to` says who loads it. The ordering is pinned
    // rather than left to the next edit of `to_document`.
    #[test]
    fn the_applies_to_line_follows_home() {
        let doc = to_document(&scoped_rule(&["rust"]));
        let home_at = doc.find("home = ").expect("home is emitted");
        let scope_at = doc.find("applies_to = ").expect("applies_to is emitted");
        let created_at = doc.find("created = ").expect("created is emitted");
        assert!(home_at < scope_at && scope_at < created_at, "{doc}");
    }

    #[test]
    fn round_trips_a_multi_line_body() {
        let r = rule_with(
            "x",
            Home::global(),
            Status::active(),
            "First line.\nSecond line.\n\nFourth after a blank.",
        );
        assert_eq!(parse_document(&to_document(&r)), Ok(r));
    }
}
