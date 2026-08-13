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
    Body, Date, DateError, EmptyText, ErrorClass, Home, Incident, Rule, RuleTag, RuleTagError,
    Status, Title,
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
    #[error("`home` has unknown kind `{0}` (expected global | domain | project)")]
    UnknownHomeKind(String),
    /// The `status` table carried an unrecognised `kind`.
    #[error("`status` has unknown kind `{0}` (expected active | graduated | attic)")]
    UnknownStatusKind(String),
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
    status: RawStatus,
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
        let status = self.status.into_status()?;
        let body = Body::parse(body)?;
        Ok(Rule::new(
            tag,
            title,
            error_class,
            home,
            created,
            status,
            incident,
            body,
        ))
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
                Ok(Status::graduated(to)?)
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
