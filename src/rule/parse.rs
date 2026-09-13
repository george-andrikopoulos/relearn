//! Parsing the neutral rule format — `+++`-delimited TOML front-matter plus a
//! markdown body — into a [`Rule`]. Every failure is an error that names the
//! offending field; nothing is ever silently skipped, because a skipped rule is
//! a lost correction.
//!
//! Dates (`created`, and an atticked status's `date`) are carried as **quoted
//! strings** and parsed by [`Date::parse`], not by TOML's native date literal.
//! That is deliberate: an impossible date must report as a range error
//! (`MonthOutOfRange`) rather than a TOML syntax error, which is the
//! `[R:parse-wide-then-range-check]` dogfood — see the ARCHITECTURE decisions
//! log, 2026-08-13.

use serde::Deserialize;

use super::{
    Approval, Approver, Body, ControlRef, Date, DateError, EmptyText, ErrorClass, Home, Incident,
    Origin, OriginError, Recurrence, Rule, RuleTag, RuleTagError, ScopeTag, ScopeTagError, Status,
    Title,
};

/// Why a rule document failed to parse.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    /// The document did not open with a `+++` line.
    #[error("missing `+++` front-matter delimiter")]
    MissingFrontMatter,
    /// The front-matter opened with `+++` but there was no closing `+++`.
    #[error("front-matter opened with `+++` but was never closed")]
    UnterminatedFrontMatter,
    /// The front-matter was not valid TOML.
    #[error("front-matter is not valid TOML: {0}")]
    Toml(String),
    /// The `tag` field was malformed.
    #[error("field `tag`: {0}")]
    Tag(#[from] RuleTagError),
    /// A non-empty text field was empty (the message names which).
    #[error("{0}")]
    Text(#[from] EmptyText),
    /// A date field held an out-of-range or malformed value.
    #[error("field `{field}`: {source}")]
    Date {
        /// Which date field (`created` or `status.date`).
        field: &'static str,
        /// The underlying date error.
        source: DateError,
    },
    /// The `home` table carried an unrecognised `kind`.
    #[error("`home` has unknown kind `{0}` (expected global | org | domain | project)")]
    UnknownHomeKind(String),
    /// The `status` table carried an unrecognised `kind`.
    #[error("`status` has unknown kind `{0}` (expected active | graduated | attic)")]
    UnknownStatusKind(String),
    /// The `origin` field, or its `approval` table, did not describe a valid
    /// origin — including the two halves of "an approval is required when and
    /// only when the origin is mandated".
    #[error("field `origin`: {0}")]
    Origin(#[from] OriginError),
    /// An `applies_to` entry was not a well-formed scope.
    #[error("field `applies_to`: {0}")]
    Scope(#[from] ScopeTagError),
    /// The same scope appeared twice in one rule's `applies_to`.
    ///
    /// A parse error for the same reason a duplicate rule tag is: it is a
    /// mistake, and accepting it would mean one written intent with two
    /// behaviours — the list would no longer say what it appears to say, and
    /// the second entry would be silently inert.
    #[error("field `applies_to`: scope `{0}` is listed twice")]
    DuplicateScope(String),
    /// A tagged table was missing a field its kind requires.
    #[error("`{context}` requires field `{field}`")]
    MissingField {
        /// The table and kind, e.g. `home domain`.
        context: &'static str,
        /// The missing field name.
        field: &'static str,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRule {
    tag: String,
    title: String,
    error_class: String,
    home: RawHome,
    created: String,
    origin: String,
    status: RawStatus,
    incident: String,
    /// Later occurrences of the same error class, as `[[recurrence]]` tables.
    ///
    /// `default` rather than required: forty-six rule files predate the field
    /// and none of them needs editing. Absent means "has not recurred", which
    /// is the honest reading of a corpus in which recurrence was not
    /// recordable — not a claim that anyone checked.
    #[serde(default)]
    recurrence: Vec<RawRecurrence>,
    /// The audiences this rule serves, as a TOML array of strings.
    ///
    /// `default` rather than required, and the absent case must stay the
    /// overwhelmingly common one: fifty-two rule files predate the field and
    /// none of them needs editing. Absent means "every audience", which is the
    /// only safe reading — a rule that has declared no audience has not opted
    /// out of anyone's, and a rule dropped from a build is a correction lost.
    #[serde(default)]
    applies_to: Vec<String>,
    /// The sign-off behind a **mandated** rule.
    ///
    /// Optional in the raw shape and mandatory in the parsed one: the table is
    /// absent for every mined or codified rule and required for every mandate,
    /// and `Origin::parse` sees the pair and refuses either mismatch. Modelling
    /// it as `Option` here rather than on `Rule` is what keeps the illegal
    /// states out of the domain type — the raw shape may hold anything a file
    /// contains; the parsed shape may not.
    #[serde(default)]
    approval: Option<RawApproval>,
}

/// The `approval` table of a mandated rule.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawApproval {
    by: String,
    date: String,
    control: String,
}

/// One `[[recurrence]]` table. An array of tables rather than a list of
/// strings so the pair stays legible and diffable in the file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRecurrence {
    date: String,
    incident: String,
}

#[derive(Deserialize)]
struct RawHome {
    kind: String,
    name: Option<String>,
    path: Option<String>,
}

#[derive(Deserialize)]
struct RawStatus {
    kind: String,
    to: Option<String>,
    reason: Option<String>,
    date: Option<String>,
}

/// Parse a rule document (`+++` front-matter, then markdown body) into a
/// [`Rule`]. The caller (`fsio`) supplies the file name for diagnostics; this
/// function is pure and names only fields.
pub fn parse_document(doc: &str) -> Result<Rule, ParseError> {
    let (front, body) = split_frontmatter(doc)?;
    let raw: RawRule = toml::from_str(&front).map_err(|e| ParseError::Toml(e.to_string()))?;
    raw.into_rule(body)
}

/// Split a document into its front-matter (between the first two `+++` lines)
/// and the body (everything after).
fn split_frontmatter(doc: &str) -> Result<(String, String), ParseError> {
    let trimmed = doc.trim_start_matches('\u{feff}').trim_start();
    let mut lines = trimmed.lines();
    match lines.next() {
        Some(first) if first.trim() == "+++" => {}
        _ => return Err(ParseError::MissingFrontMatter),
    }
    let mut front = String::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line.trim() == "+++" {
            closed = true;
            break;
        }
        front.push_str(line);
        front.push('\n');
    }
    if !closed {
        return Err(ParseError::UnterminatedFrontMatter);
    }
    let body = lines.collect::<Vec<_>>().join("\n");
    Ok((front, body))
}

impl RawRule {
    fn into_rule(self, body: String) -> Result<Rule, ParseError> {
        let tag = RuleTag::parse(self.tag)?;
        let title = Title::parse(self.title)?;
        let error_class = ErrorClass::parse(self.error_class)?;
        let incident = Incident::parse(self.incident)?;
        let home = self.home.into_home()?;
        let created = Date::parse(&self.created).map_err(|source| ParseError::Date {
            field: "created",
            source,
        })?;
        // The approval is parsed first so its own fields report their own
        // errors (`approval.by must not be empty`), and handed to `Origin::parse`
        // as the other half of the pair it has to judge.
        let approval = self.approval.map(RawApproval::into_approval).transpose()?;
        let origin = Origin::parse(&self.origin, approval)?;
        let status = self.status.into_status()?;
        let body = Body::parse(body)?;
        // Collected with `?`, not filtered: a malformed recurrence stops the
        // build like any other field. A recurrence silently dropped is the
        // evidence that the rule failed, dropped.
        let recurrences = self
            .recurrence
            .into_iter()
            .map(RawRecurrence::into_recurrence)
            .collect::<Result<Vec<_>, _>>()?;
        let applies_to = parse_scopes(self.applies_to)?;
        Ok(Rule::new(
            tag,
            title,
            error_class,
            home,
            created,
            origin,
            status,
            incident,
            body,
            recurrences,
            applies_to,
        ))
    }
}

/// Parse an `applies_to` array into scope witnesses, rejecting a duplicate.
///
/// Collected with `?` rather than filtered, like every other field: a malformed
/// scope stops the build. The duplicate check is linear over a list that is
/// realistically two or three entries long, and it preserves file order — the
/// serializer must reproduce the file it parsed, so sorting here would rewrite
/// every scoped rule on the next build.
fn parse_scopes(raw: Vec<String>) -> Result<Vec<ScopeTag>, ParseError> {
    let mut scopes: Vec<ScopeTag> = Vec::with_capacity(raw.len());
    for entry in raw {
        let scope = ScopeTag::parse(entry)?;
        if scopes.contains(&scope) {
            return Err(ParseError::DuplicateScope(scope.as_str().to_owned()));
        }
        scopes.push(scope);
    }
    Ok(scopes)
}

impl RawApproval {
    fn into_approval(self) -> Result<Approval, ParseError> {
        let by = Approver::parse(self.by)?;
        let date = Date::parse(&self.date).map_err(|source| ParseError::Date {
            field: "approval.date",
            source,
        })?;
        let control = ControlRef::parse(self.control)?;
        Ok(Approval::new(by, date, control))
    }
}

impl RawRecurrence {
    fn into_recurrence(self) -> Result<Recurrence, ParseError> {
        let date = Date::parse(&self.date).map_err(|source| ParseError::Date {
            field: "recurrence.date",
            source,
        })?;
        let incident = Incident::parse(self.incident)?;
        Ok(Recurrence::new(date, incident))
    }
}

impl RawHome {
    fn into_home(self) -> Result<Home, ParseError> {
        match self.kind.as_str() {
            "global" => Ok(Home::global()),
            "domain" => {
                let name = self.name.ok_or(ParseError::MissingField {
                    context: "home domain",
                    field: "name",
                })?;
                Ok(Home::domain(name)?)
            }
            "org" => {
                let name = self.name.ok_or(ParseError::MissingField {
                    context: "home org",
                    field: "name",
                })?;
                Ok(Home::org(name)?)
            }
            "project" => {
                let path = self.path.ok_or(ParseError::MissingField {
                    context: "home project",
                    field: "path",
                })?;
                Ok(Home::project(path)?)
            }
            other => Err(ParseError::UnknownHomeKind(other.to_owned())),
        }
    }
}

impl RawStatus {
    fn into_status(self) -> Result<Status, ParseError> {
        match self.kind.as_str() {
            "active" => Ok(Status::active()),
            "graduated" => {
                let to = self.to.ok_or(ParseError::MissingField {
                    context: "status graduated",
                    field: "to",
                })?;
                let date_str = self.date.ok_or(ParseError::MissingField {
                    context: "status graduated",
                    field: "date",
                })?;
                let date = Date::parse(&date_str).map_err(|source| ParseError::Date {
                    field: "status.date",
                    source,
                })?;
                Ok(Status::graduated(to, date)?)
            }
            "attic" => {
                let reason = self.reason.ok_or(ParseError::MissingField {
                    context: "status attic",
                    field: "reason",
                })?;
                let date_str = self.date.ok_or(ParseError::MissingField {
                    context: "status attic",
                    field: "date",
                })?;
                let date = Date::parse(&date_str).map_err(|source| ParseError::Date {
                    field: "status.date",
                    source,
                })?;
                Ok(Status::attic(reason, date)?)
            }
            other => Err(ParseError::UnknownStatusKind(other.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"+++
tag = "R:parse-wide-then-range-check"
title = "Parse wide, then range-check"
error_class = "narrowing at parse makes OutOfRange unreachable"
home = { kind = "domain", name = "rust" }
created = "2026-07-23"
origin = "mined"
status = { kind = "active" }
incident = "grouping task 01"
+++

Parse into a type wide enough to represent the out-of-range value.
"#;

    // Build a valid document, overriding one field's value line.
    fn with_field(field: &str, value: &str) -> String {
        let base = [
            ("tag", "\"R:x\""),
            ("title", "\"t\""),
            ("error_class", "\"e\""),
            ("home", "{ kind = \"global\" }"),
            ("created", "\"2026-01-01\""),
            ("origin", "\"mined\""),
            ("status", "{ kind = \"active\" }"),
            ("incident", "\"i\""),
        ];
        let mut front = String::from("+++\n");
        for (k, v) in base {
            let vv = if k == field { value } else { v };
            front.push_str(&format!("{k} = {vv}\n"));
        }
        front.push_str("+++\n\nBody.\n");
        front
    }

    #[test]
    fn parses_a_full_document() {
        let rule = parse_document(DOC).expect("valid document parses");
        assert_eq!(rule.tag().as_str(), "R:parse-wide-then-range-check");
        assert_eq!(rule.title().as_str(), "Parse wide, then range-check");
        assert_eq!(rule.created().to_string(), "2026-07-23");
        assert!(matches!(rule.home(), Home::Domain { .. }));
        assert!(matches!(rule.status(), Status::Active));
        assert!(rule.body().as_str().starts_with("Parse into a type"));
    }

    #[test]
    fn missing_frontmatter_is_an_error() {
        assert_eq!(
            parse_document("no frontmatter here"),
            Err(ParseError::MissingFrontMatter)
        );
    }

    #[test]
    fn unterminated_frontmatter_is_an_error() {
        assert_eq!(
            parse_document("+++\ntag = \"R:x\"\n"),
            Err(ParseError::UnterminatedFrontMatter)
        );
    }

    #[test]
    fn invalid_toml_is_named() {
        assert!(matches!(
            parse_document("+++\ntag = = =\n+++\nbody"),
            Err(ParseError::Toml(_))
        ));
    }

    #[test]
    fn bad_tag_is_a_field_error() {
        let doc = with_field("tag", "\"not-a-tag\"");
        assert!(matches!(parse_document(&doc), Err(ParseError::Tag(_))));
    }

    // THE end-to-end dogfood: an impossible date in `created` surfaces as a
    // range error naming the value, through the whole parser — not a TOML
    // syntax error, not a silent skip.
    #[test]
    fn out_of_range_created_date_is_a_range_error_end_to_end() {
        let doc = with_field("created", "\"2026-13-01\"");
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::Date {
                field: "created",
                source: DateError::MonthOutOfRange(13),
            })
        );
    }

    #[test]
    fn empty_body_is_rejected() {
        let mut doc = with_field("tag", "\"R:x\"");
        // Replace the "Body." trailer with whitespace only.
        doc = doc.replace("\n\nBody.\n", "\n\n   \n");
        assert!(matches!(parse_document(&doc), Err(ParseError::Text(_))));
    }

    #[test]
    fn domain_home_without_name_is_missing_field() {
        let doc = with_field("home", "{ kind = \"domain\" }");
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::MissingField {
                context: "home domain",
                field: "name",
            })
        );
    }

    #[test]
    fn unknown_home_kind_is_rejected() {
        let doc = with_field("home", "{ kind = \"universe\" }");
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::UnknownHomeKind("universe".to_owned()))
        );
    }

    #[test]
    fn unknown_top_level_field_is_rejected() {
        let mut doc = with_field("tag", "\"R:x\"");
        doc = doc.replace("incident = \"i\"\n", "incident = \"i\"\nbogus = 1\n");
        assert!(matches!(parse_document(&doc), Err(ParseError::Toml(_))));
    }

    #[test]
    fn attic_status_parses_with_reason_and_date() {
        let doc = with_field(
            "status",
            "{ kind = \"attic\", reason = \"cold surface\", date = \"2026-09-01\" }",
        );
        let rule = parse_document(&doc).expect("attic parses");
        assert!(matches!(rule.status(), Status::Attic { .. }));
    }

    // A rule file written before the field existed parses to a rule with no
    // recurrences — the reason none of the committed rules needed editing.
    #[test]
    fn a_rule_without_recurrences_parses_to_none() {
        let rule = parse_document(DOC).expect("valid document parses");
        assert!(!rule.has_recurred());
        assert!(rule.recurrences().is_empty());
    }

    #[test]
    fn recurrences_parse_as_an_array_of_tables() {
        let doc = DOC.replace(
            "+++\n\nParse into",
            concat!(
                "\n[[recurrence]]\n",
                "date = \"2026-08-24\"\n",
                "incident = \"Fired again: the build-path claim was never repaired.\"\n",
                "\n[[recurrence]]\n",
                "date = \"2026-08-30\"\n",
                "incident = \"And again, in the domain-knowledge section.\"\n",
                "+++\n\nParse into"
            ),
        );
        let rule = parse_document(&doc).expect("recurrences parse");
        assert!(rule.has_recurred());
        assert_eq!(rule.recurrences().len(), 2);
        assert_eq!(rule.recurrences()[0].date().to_string(), "2026-08-24");
        assert!(
            rule.recurrences()[1]
                .incident()
                .as_str()
                .starts_with("And again")
        );
        assert_eq!(
            rule.latest_recurrence().map(|d| d.to_string()),
            Some("2026-08-30".to_owned())
        );
    }

    // A malformed recurrence stops the build like any other field, and the
    // error names which date failed — `recurrence.date`, not `created`.
    #[test]
    fn a_recurrence_with_a_bad_date_is_a_range_error_naming_the_field() {
        let doc = DOC.replace(
            "+++\n\nParse into",
            "\n[[recurrence]]\ndate = \"2026-02-30\"\nincident = \"again\"\n+++\n\nParse into",
        );
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::Date {
                field: "recurrence.date",
                source: DateError::DayOutOfRange(30),
            })
        );
    }

    #[test]
    fn a_recurrence_with_an_empty_incident_is_rejected() {
        let doc = DOC.replace(
            "+++\n\nParse into",
            "\n[[recurrence]]\ndate = \"2026-08-24\"\nincident = \"  \"\n+++\n\nParse into",
        );
        assert!(matches!(parse_document(&doc), Err(ParseError::Text(_))));
    }

    #[test]
    fn an_unknown_field_in_a_recurrence_is_rejected() {
        let doc = DOC.replace(
            "+++\n\nParse into",
            "\n[[recurrence]]\ndate = \"2026-08-24\"\nincident = \"again\"\nbogus = 1\n+++\n\nParse into",
        );
        assert!(matches!(parse_document(&doc), Err(ParseError::Toml(_))));
    }

    // A rule file written before `applies_to` existed parses to a rule that
    // serves every audience — the reason none of the fifty-two committed rules
    // needed editing, and the reason adding the field to one rule cannot remove
    // a different rule from anyone's build.
    #[test]
    fn a_rule_without_applies_to_parses_as_unscoped() {
        let rule = parse_document(DOC).expect("valid document parses");
        assert!(!rule.is_scoped());
        assert!(rule.applies_to().is_empty());
        assert!(rule.serves(&[ScopeTag::parse("java").expect("valid scope")]));
    }

    #[test]
    fn applies_to_parses_as_an_array_of_scopes_in_file_order() {
        let doc = with_field("tag", "\"R:x\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napplies_to = [\"rust\", \"java\"]\n",
        );
        let rule = parse_document(&doc).expect("scopes parse");
        let scopes: Vec<&str> = rule.applies_to().iter().map(ScopeTag::as_str).collect();
        assert_eq!(scopes, vec!["rust", "java"]);
        assert!(rule.is_scoped());
    }

    #[test]
    fn a_malformed_scope_stops_the_build_naming_the_field() {
        let doc = with_field("tag", "\"R:x\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napplies_to = [\"Rust\"]\n",
        );
        assert!(matches!(parse_document(&doc), Err(ParseError::Scope(_))));
    }

    #[test]
    fn an_empty_scope_entry_is_rejected() {
        let doc = with_field("tag", "\"R:x\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napplies_to = [\"rust\", \"  \"]\n",
        );
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::Scope(ScopeTagError::Empty))
        );
    }

    // A duplicate is a parse error for the same reason a duplicate rule tag is:
    // one written intent with two behaviours, the second of them inert.
    #[test]
    fn a_repeated_scope_is_a_parse_error_naming_it() {
        let doc = with_field("tag", "\"R:x\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napplies_to = [\"rust\", \"java\", \"rust\"]\n",
        );
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::DuplicateScope("rust".to_owned()))
        );
    }

    // Trimming happens at the perimeter, so a stray space in the array is a
    // formatting accident rather than a failure — and the *trimmed* value is
    // what a duplicate is judged against.
    #[test]
    fn scopes_are_trimmed_before_the_duplicate_check() {
        let doc = with_field("tag", "\"R:x\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napplies_to = [\"rust\", \" rust \"]\n",
        );
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::DuplicateScope("rust".to_owned()))
        );
    }

    #[test]
    fn an_empty_applies_to_array_is_the_same_as_absent() {
        let doc = with_field("tag", "\"R:x\"")
            .replace("incident = \"i\"\n", "incident = \"i\"\napplies_to = []\n");
        let rule = parse_document(&doc).expect("an empty array parses");
        assert!(!rule.is_scoped());
    }

    #[test]
    fn an_org_home_parses_and_names_its_organisation() {
        let doc = with_field("home", "{ kind = \"org\", name = \"acme\" }");
        let rule = parse_document(&doc).expect("an org home parses");
        match rule.home() {
            Home::Org { name } => assert_eq!(name.as_str(), "acme"),
            other => panic!("expected an org home, got {other:?}"),
        }
        assert!(
            !rule.is_publishable(),
            "an org-homed rule must never be publishable"
        );
    }

    #[test]
    fn an_org_home_without_a_name_is_missing_field() {
        let doc = with_field("home", "{ kind = \"org\" }");
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::MissingField {
                context: "home org",
                field: "name",
            })
        );
    }

    #[test]
    fn a_mandate_parses_with_its_approval_table() {
        let doc = with_field("origin", "\"mandated\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napproval = { by = \"the change board\", \
             date = \"2026-07-11\", control = \"AC-6(9)\" }\n",
        );
        let rule = parse_document(&doc).expect("a mandate parses");
        let approval = rule.origin().approval().expect("a mandate has an approval");
        assert_eq!(approval.by().as_str(), "the change board");
        assert_eq!(approval.date().to_string(), "2026-07-11");
        assert_eq!(approval.control().as_str(), "AC-6(9)");
        assert!(!rule.counts_toward_recurrence_statistics());
    }

    // The two halves of "required when and only when", end to end through the
    // whole parser rather than only at `Origin::parse`.
    #[test]
    fn a_mandate_without_an_approval_stops_the_build() {
        let doc = with_field("origin", "\"mandated\"");
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::Origin(OriginError::MandateWithoutApproval))
        );
    }

    #[test]
    fn an_approval_without_a_mandate_stops_the_build() {
        let doc = with_field("tag", "\"R:x\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napproval = { by = \"b\", date = \"2026-07-11\", control = \"c\" }\n",
        );
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::Origin(OriginError::ApprovalWithoutMandate(
                "mined".to_owned()
            )))
        );
    }

    #[test]
    fn an_approval_with_a_bad_date_names_its_own_field() {
        let doc = with_field("origin", "\"mandated\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napproval = { by = \"b\", date = \"2026-02-30\", control = \"c\" }\n",
        );
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::Date {
                field: "approval.date",
                source: DateError::DayOutOfRange(30),
            })
        );
    }

    #[test]
    fn an_empty_approver_is_rejected() {
        let doc = with_field("origin", "\"mandated\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napproval = { by = \"  \", date = \"2026-07-11\", control = \"c\" }\n",
        );
        assert!(matches!(parse_document(&doc), Err(ParseError::Text(_))));
    }

    #[test]
    fn an_unknown_field_in_an_approval_is_rejected() {
        let doc = with_field("origin", "\"mandated\"").replace(
            "incident = \"i\"\n",
            "incident = \"i\"\napproval = { by = \"b\", date = \"2026-07-11\", \
             control = \"c\", bogus = 1 }\n",
        );
        assert!(matches!(parse_document(&doc), Err(ParseError::Toml(_))));
    }

    #[test]
    fn attic_status_with_bad_date_names_status_date() {
        let doc = with_field(
            "status",
            "{ kind = \"attic\", reason = \"cold\", date = \"2026-02-30\" }",
        );
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::Date {
                field: "status.date",
                source: DateError::DayOutOfRange(30),
            })
        );
    }
}
