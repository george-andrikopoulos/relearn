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

use super::{Approval, Authority, Home, Rule, ScopeTag, Status};

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
    // Beside `origin`, because it *is* the origin's payload: a mandate's
    // provenance is its sign-off, where a mined rule's is its incident. Emitted
    // by matching the variant rather than by testing an option, so the two can
    // never disagree about whether a rule was approved.
    if let Some(approval) = rule.origin().approval() {
        out.push_str(&approval_line(approval));
    }
    out.push_str(&status_line(rule.status()));
    // Emitted **only** when the rule is not local, so every rule written before
    // the field existed renders exactly as it did — and, more importantly, a
    // file that says nothing about authority reads back as `Local`, which is the
    // safe direction: a cache must declare itself before anything treats it as
    // one.
    if let Some(line) = authority_line(rule.authority()) {
        out.push_str(&line);
    }
    out.push_str(&kv("incident", rule.incident().as_str()));
    // Beside `incident`, because a reader comparing the two is exactly the
    // review this field exists for: the quotation that stays, and the account
    // that may travel. Emitted only when authored, so no committed rule file
    // changed when the field arrived.
    if let Some(published) = rule.published_incident() {
        out.push_str(&kv("published_incident", published.as_str()));
    }
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
        Home::Org { name } => format!(
            "home = {{ kind = \"org\", name = {} }}\n",
            toml_basic_string(name.as_str())
        ),
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

/// The `approval = { ... }` line of a mandated rule — the inline table the
/// parser requires when, and only when, `origin = "mandated"`.
fn approval_line(approval: &Approval) -> String {
    format!(
        "approval = {{ by = {}, date = {}, control = {} }}\n",
        toml_basic_string(approval.by().as_str()),
        toml_basic_string(&approval.date().to_string()),
        toml_basic_string(approval.control().as_str())
    )
}

/// The `authority = { ... }` line, or `None` for a local rule with no revision.
///
/// An unnumbered `Local` renders nothing rather than `kind = "local"`: the
/// overwhelmingly common case is a rule this install owns, and a line every
/// file carries is a line no reader reads. The parser's default closes the
/// loop — absent is `Local` with no revision, and the round trip is exact in
/// both directions. A **numbered** local rule does render, because the number
/// is the only thing a cache of it can be compared against and dropping it
/// would make every such cache permanently un-stale.
fn authority_line(authority: &Authority) -> Option<String> {
    match authority {
        Authority::Local { version: None } => None,
        Authority::Local {
            version: Some(version),
        } => Some(format!(
            "authority = {{ kind = \"local\", version = {version} }}\n"
        )),
        Authority::Cached {
            from,
            version,
            pulled,
        } => Some(format!(
            "authority = {{ kind = \"cached\", from = {}, version = {version}, pulled = {} }}\n",
            toml_basic_string(from.as_str()),
            toml_basic_string(&pulled.to_string())
        )),
        Authority::Adopted {
            from,
            version,
            pulled,
            adopted,
        } => Some(format!(
            "authority = {{ kind = \"adopted\", from = {}, version = {version}, \
             pulled = {}, adopted = {} }}\n",
            toml_basic_string(from.as_str()),
            toml_basic_string(&pulled.to_string()),
            toml_basic_string(&adopted.to_string())
        )),
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
            toml_basic_string(&to.to_string()),
            toml_basic_string(&date.to_string())
        ),
        Status::Partial {
            by,
            uncovered,
            date,
        } => format!(
            "status = {{ kind = \"partial\", by = {}, uncovered = {}, date = {} }}\n",
            toml_basic_string(&by.to_string()),
            toml_basic_string(uncovered.as_str()),
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
        Approval, Approver, Authority, Body, ControlRef, Date, ErrorClass, Home, Incident, Origin,
        Recurrence, Rule, RuleTag, ScopeTag, SourceId, Status, Title, Version, parse_document,
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
            Authority::local(),
            None,
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
            Authority::local(),
            None,
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
            Authority::local(),
            None,
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
    fn round_trips_an_org_home() {
        let r = rule_with(
            "an org principle",
            Home::org("acme").expect("non-empty org"),
            Status::active(),
            "Do the thing.",
        );
        let doc = to_document(&r);
        assert!(
            doc.contains("home = { kind = \"org\", name = \"acme\" }"),
            "{doc}"
        );
        assert_eq!(parse_document(&doc), Ok(r));
    }

    fn mandated_rule() -> Rule {
        Rule::new(
            RuleTag::parse("R:x").expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::org("acme").expect("non-empty org"),
            Date::parse("2026-09-13").expect("valid date"),
            Origin::Mandated(Approval::new(
                Approver::parse("the change board").expect("non-empty approver"),
                Date::parse("2026-07-11").expect("valid date"),
                ControlRef::parse("AC-6(9)").expect("non-empty control"),
            )),
            Status::active(),
            Incident::parse("the mandate was accepted").expect("non-empty incident"),
            Body::parse("Do the mandated thing.").expect("non-empty body"),
            Vec::new(),
            Vec::new(),
            Authority::local(),
            None,
        )
    }

    #[test]
    fn round_trips_a_mandate_with_its_approval() {
        let r = mandated_rule();
        let doc = to_document(&r);
        assert!(doc.contains("origin = \"mandated\""), "{doc}");
        assert!(
            doc.contains(
                "approval = { by = \"the change board\", date = \"2026-07-11\", \
                 control = \"AC-6(9)\" }"
            ),
            "{doc}"
        );
        assert_eq!(parse_document(&doc), Ok(r));
    }

    /// The approval line comes from matching the variant, so a rule that is not
    /// mandated cannot render one — there is no option to leave set by mistake.
    #[test]
    fn a_rule_that_is_not_mandated_renders_no_approval_line() {
        let doc = to_document(&rule_with(
            "x",
            Home::global(),
            Status::active(),
            "Do the thing.",
        ));
        assert!(!doc.contains("approval"), "{doc}");
    }

    /// Beside `origin`, because it is the origin's payload: a reader asking
    /// "where did this rule come from?" finds the answer and its signer
    /// together.
    #[test]
    fn the_approval_line_follows_origin() {
        let doc = to_document(&mandated_rule());
        let origin_at = doc.find("origin = ").expect("origin is emitted");
        let approval_at = doc.find("approval = ").expect("approval is emitted");
        let status_at = doc.find("status = ").expect("status is emitted");
        assert!(origin_at < approval_at && approval_at < status_at, "{doc}");
    }

    fn rule_under(authority: Authority) -> Rule {
        Rule::new(
            RuleTag::parse("R:x").expect("valid tag"),
            Title::parse("A title").expect("non-empty title"),
            ErrorClass::parse("an error class").expect("non-empty error class"),
            Home::global(),
            Date::parse("2026-09-13").expect("valid date"),
            Origin::Mined,
            Status::active(),
            Incident::parse("the triggering incident").expect("non-empty incident"),
            Body::parse("Do the thing.").expect("non-empty body"),
            Vec::new(),
            Vec::new(),
            authority,
            None,
        )
    }

    /// A local rule renders no authority line at all — the reason no committed
    /// rule file needed editing, and the reason absent can safely mean local.
    #[test]
    fn a_local_rule_renders_no_authority_line() {
        let doc = to_document(&rule_under(Authority::local()));
        assert!(!doc.contains("authority"), "{doc}");
    }

    #[test]
    fn round_trips_a_cached_authority() {
        let r = rule_under(Authority::cached(
            SourceId::parse("relearn-upstream").expect("valid source"),
            Version::new(3),
            Date::parse("2026-09-10").expect("valid date"),
        ));
        let doc = to_document(&r);
        assert!(doc.contains("version = 3"), "{doc}");
        assert_eq!(parse_document(&doc), Ok(r));
    }

    #[test]
    fn round_trips_an_adopted_authority() {
        let r = rule_under(Authority::adopted(
            SourceId::parse("relearn-upstream").expect("valid source"),
            Version::new(12),
            Date::parse("2026-09-10").expect("valid date"),
            Date::parse("2026-09-13").expect("valid date"),
        ));
        assert_eq!(parse_document(&to_document(&r)), Ok(r));
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
