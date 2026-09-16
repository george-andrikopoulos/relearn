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
    Approval, Approver, Authority, Body, ControlError, ControlRef, Date, DateError, EmptyText,
    ErrorClass, Home, Incident, Origin, OriginError, PublishedIncident, Recurrence, Rule, RuleTag,
    RuleTagError, ScopeTag, ScopeTagError, SourceId, Status, StatusError, Title, Version,
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

    #[error("field `status`: {0}")]
    Control(#[from] ControlError),

    #[error("field `status`: {0}")]
    Status(#[from] StatusError),
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
    /// The `authority` table carried an unrecognised `kind`.
    #[error("`authority` has unknown kind `{0}` (expected local | adopted | cached)")]
    UnknownAuthorityKind(String),
    /// `authority.version` was outside the range a revision number can hold.
    #[error("`authority.version` is {0}, outside the range of a revision number")]
    VersionOutOfRange(i64),
    /// A field was present that the authority's `kind` has no meaning for —
    /// a copy's provenance (`from`, `pulled`, `adopted`) under `kind =
    /// "local"`.
    ///
    /// Refused rather than ignored. A field the parser drops reads back, to
    /// the next person, as a fact it accepted: a file saying it is this
    /// install's own rule *and* naming where it was pulled from has said two
    /// contradictory things, and silence picks one of them without saying so.
    #[error("`{context}` has no field `{field}` — that belongs to a cached or adopted rule")]
    UnexpectedField {
        /// Which authority kind was being read.
        context: &'static str,
        /// The field that has no meaning there.
        field: &'static str,
    },
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
    /// Whether this install is the rule's home, holds a cache of one whose home
    /// is elsewhere, or holds a deliberate fork.
    ///
    /// `default` and absent means [`Authority::Local`] — what every rule
    /// written before the field existed is, and what a corpus that has never
    /// cached anything stays. The safe reading in both directions: a rule file
    /// that says nothing is this install's own, and a cache must **say** it is
    /// a cache before anything treats it as one.
    #[serde(default)]
    authority: Option<RawAuthority>,
    /// The publishable account of the incident — a **separate authored field**,
    /// never a transformation of `incident`.
    ///
    /// Absent for every rule nobody has prepared for contribution, which is
    /// almost all of them. Its absence is why `contribute` refuses: there is
    /// nothing to publish in place of the quotation, and the quotation itself
    /// is what may not travel.
    #[serde(default)]
    published_incident: Option<String>,
}

/// The `authority` table: `kind`, plus the provenance the kind requires.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawAuthority {
    kind: String,
    from: Option<String>,
    /// Parsed **wide** as `i64` and range-checked into `u32`, so a negative or
    /// oversized revision reports as out-of-range rather than collapsing into
    /// "not a number" (`[R:parse-wide-then-range-check]`).
    version: Option<i64>,
    pulled: Option<String>,
    adopted: Option<String>,
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
    by: Option<String>,
    uncovered: Option<String>,
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
        // Absent means Local: a rule file that says nothing about authority is
        // this install's own, and a cache must say it is one.
        let authority = match self.authority {
            Some(raw) => raw.into_authority()?,
            None => Authority::local(),
        };
        // Authored only when a rule is being prepared for contribution, which is
        // almost never — and never derived from `incident`, which is the whole
        // design: see `PublishedIncident`.
        let published_incident = self
            .published_incident
            .map(PublishedIncident::parse)
            .transpose()?;
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
            authority,
            published_incident,
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

/// The range check the wide parse exists for: a revision is a `u32`, and a file
/// holding `-1` or `5_000_000_000` must say so rather than read as a syntax
/// error (`[R:parse-wide-then-range-check]`).
///
/// One function, read by every authority kind, so a revision cannot be
/// range-checked in one arm and taken on trust in another.
fn revision(version: i64) -> Result<Version, ParseError> {
    u32::try_from(version)
        .map(Version::new)
        .map_err(|_| ParseError::VersionOutOfRange(version))
}

impl RawAuthority {
    fn into_authority(self) -> Result<Authority, ParseError> {
        match self.kind.as_str() {
            "local" => {
                // A home may state which revision it is, and nothing else: a
                // source, a pull date and an adoption date are a *copy's*
                // provenance. Refused rather than ignored, for the reason
                // `UnexpectedField` records.
                for (field, present) in [
                    ("from", self.from.is_some()),
                    ("pulled", self.pulled.is_some()),
                    ("adopted", self.adopted.is_some()),
                ] {
                    if present {
                        return Err(ParseError::UnexpectedField {
                            context: "authority local",
                            field,
                        });
                    }
                }
                match self.version {
                    None => Ok(Authority::local()),
                    Some(version) => Ok(Authority::local_at(revision(version)?)),
                }
            }
            "cached" => {
                let (from, version, pulled) = self.upstream("authority cached")?;
                Ok(Authority::cached(from, version, pulled))
            }
            "adopted" => {
                let (from, version, pulled) = self.upstream_for("authority adopted")?;
                let adopted = self.adopted.ok_or(ParseError::MissingField {
                    context: "authority adopted",
                    field: "adopted",
                })?;
                let adopted = Date::parse(&adopted).map_err(|source| ParseError::Date {
                    field: "authority.adopted",
                    source,
                })?;
                Ok(Authority::adopted(from, version, pulled, adopted))
            }
            other => Err(ParseError::UnknownAuthorityKind(other.to_owned())),
        }
    }

    /// The three fields every non-local authority carries. Taken by value, so
    /// the `adopted` kind reads them before adding its own field.
    fn upstream(self, context: &'static str) -> Result<(SourceId, Version, Date), ParseError> {
        self.upstream_for(context)
    }

    fn upstream_for(&self, context: &'static str) -> Result<(SourceId, Version, Date), ParseError> {
        let from = self
            .from
            .as_ref()
            .ok_or(ParseError::MissingField {
                context,
                field: "from",
            })
            .and_then(|s| Ok(SourceId::parse(s.as_str())?))?;
        let version = self.version.ok_or(ParseError::MissingField {
            context,
            field: "version",
        })?;
        let version = revision(version)?;
        let pulled = self.pulled.as_ref().ok_or(ParseError::MissingField {
            context,
            field: "pulled",
        })?;
        let pulled = Date::parse(pulled.as_str()).map_err(|source| ParseError::Date {
            field: "authority.pulled",
            source,
        })?;
        Ok((from, version, pulled))
    }
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
            "partial" => {
                let by = self.by.ok_or(ParseError::MissingField {
                    context: "status partial",
                    field: "by",
                })?;
                // Required, not defaulted. A partial graduation that does not say
                // which part is still uncovered reads exactly like a full one.
                let uncovered = self.uncovered.ok_or(ParseError::MissingField {
                    context: "status partial",
                    field: "uncovered",
                })?;
                let date_str = self.date.ok_or(ParseError::MissingField {
                    context: "status partial",
                    field: "date",
                })?;
                let date = Date::parse(&date_str).map_err(|source| ParseError::Date {
                    field: "status.date",
                    source,
                })?;
                Ok(Status::partial(by, uncovered, date)?)
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

    /// Absent means local — the reason none of the fifty-two committed rules
    /// needed editing, and the safe direction: a cache must *say* it is one.
    #[test]
    fn a_rule_without_an_authority_is_local_and_editable() {
        let rule = parse_document(DOC).expect("valid document parses");
        assert_eq!(rule.authority(), &Authority::local());
        assert!(rule.is_editable());
    }

    fn with_authority(table: &str) -> String {
        with_field("tag", "\"R:x\"").replace(
            "incident = \"i\"\n",
            &format!("incident = \"i\"\nauthority = {table}\n"),
        )
    }

    /// A home may state which revision it is, and the parser **keeps** it. It
    /// used to read the field and throw it away, which is the shape of defect
    /// `UnexpectedField` exists for: a number written down, accepted, and
    /// silently absent from everything downstream.
    #[test]
    fn a_local_authority_keeps_its_revision() {
        let doc = with_authority("{ kind = \"local\", version = 4 }");
        let rule = parse_document(&doc).expect("a numbered local authority parses");
        assert_eq!(rule.authority(), &Authority::local_at(Version::new(4)));
        assert!(rule.is_editable(), "a home is still this install's own");
    }

    /// A copy's provenance under `kind = "local"` is two contradictory
    /// statements in one table. Refused rather than ignored.
    #[test]
    fn a_local_authority_carrying_a_copys_provenance_is_refused() {
        for field in [
            "from = \"upstream\"",
            "pulled = \"2026-09-10\"",
            "adopted = \"2026-09-14\"",
        ] {
            let doc = with_authority(&format!("{{ kind = \"local\", {field} }}"));
            assert!(
                matches!(
                    parse_document(&doc),
                    Err(ParseError::UnexpectedField {
                        context: "authority local",
                        ..
                    })
                ),
                "a local authority accepted {field}"
            );
        }
    }

    /// The wide parse reaches the local arm too, through the one range check
    /// every kind reads — never a second copy that could be forgotten.
    #[test]
    fn a_local_revision_out_of_range_says_so() {
        let doc = with_authority("{ kind = \"local\", version = -1 }");
        assert!(matches!(
            parse_document(&doc),
            Err(ParseError::VersionOutOfRange(-1))
        ));
    }

    #[test]
    fn a_cached_authority_parses_and_is_not_editable() {
        let doc = with_authority(
            "{ kind = \"cached\", from = \"relearn-upstream\", version = 3, pulled = \"2026-09-10\" }",
        );
        let rule = parse_document(&doc).expect("a cached authority parses");
        assert!(!rule.is_editable());
        assert_eq!(rule.authority().version(), Some(Version::new(3)));
    }

    #[test]
    fn an_adopted_authority_keeps_both_dates() {
        let doc = with_authority(
            "{ kind = \"adopted\", from = \"up\", version = 1, pulled = \"2026-09-10\", \
             adopted = \"2026-09-13\" }",
        );
        let rule = parse_document(&doc).expect("an adopted authority parses");
        assert!(rule.is_editable());
        match rule.authority() {
            Authority::Adopted {
                pulled, adopted, ..
            } => {
                assert_eq!(pulled.to_string(), "2026-09-10");
                assert_eq!(adopted.to_string(), "2026-09-13");
            }
            other => panic!("expected Adopted, got {other:?}"),
        }
    }

    #[test]
    fn a_cached_authority_without_a_version_is_missing_field() {
        let doc = with_authority("{ kind = \"cached\", from = \"up\", pulled = \"2026-09-10\" }");
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::MissingField {
                context: "authority cached",
                field: "version",
            })
        );
    }

    // The dogfood, one field over: a version outside `u32` reports as
    // out-of-range rather than collapsing into "not a number"
    // (`[R:parse-wide-then-range-check]`).
    #[test]
    fn a_negative_version_is_a_range_error_carrying_the_value() {
        let doc = with_authority(
            "{ kind = \"cached\", from = \"up\", version = -1, pulled = \"2026-09-10\" }",
        );
        assert_eq!(parse_document(&doc), Err(ParseError::VersionOutOfRange(-1)));
    }

    #[test]
    fn an_unknown_authority_kind_is_rejected() {
        let doc = with_authority("{ kind = \"borrowed\" }");
        assert_eq!(
            parse_document(&doc),
            Err(ParseError::UnknownAuthorityKind("borrowed".to_owned()))
        );
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
