//! One headless session's `stream-json` transcript, parsed at the perimeter.
//!
//! What is read, and from where (Claude Code `--output-format stream-json
//! --verbose`, one JSON object per line):
//!
//! * `assistant` messages: `text` blocks become [`EventKind::Text`]; `tool_use`
//!   blocks become shell (`Bash`, `PowerShell`: `input.command`), write
//!   (`Write`, `Edit`, `MultiEdit`: `input.file_path`; `NotebookEdit`:
//!   `input.notebook_path`) or read (`Read`: `input.file_path`) events. A
//!   `Skill` call is recorded separately as a skill the model opened. Every
//!   other tool is ignored.
//! * the `system`/`init` message: the skills, plugins and MCP servers the
//!   session started with, which the runner compares with what its arm put
//!   there. A user layer leaking into a session shows up here.
//! * the `result` message: `total_cost_usd` and `usage`.
//!
//! A line that is not JSON makes the whole transcript malformed. Skipping it
//! would let a truncated session read as a short one.

use serde_json::Value;

use crate::rule::{EventKind, SessionEvent};

/// Why a transcript could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TranscriptError {
    #[error("line {line}: not a JSON object: {reason}")]
    NotJson { line: usize, reason: String },
    #[error("line {line}: `total_cost_usd` {value} is not a non-negative decimal")]
    Cost { line: usize, value: String },
}

/// An amount in millionths of a US dollar. Money is not a float
/// (`[R:money-is-not-a-float]`), so the transcript's decimal is parsed from
/// its digits, never through `f64`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct MicroUsd(u64);

impl MicroUsd {
    /// Parse a non-negative decimal as JSON writes one: `0.25`, `3`, or the
    /// exponent form a very small amount takes (`1e-5`, `2.5E-7`). Digits
    /// beyond the sixth place are truncated.
    #[must_use]
    pub fn parse_decimal(s: &str) -> Option<Self> {
        let (mantissa, exponent) = match s.split_once(['e', 'E']) {
            Some((m, e)) => (m, e.parse::<i32>().ok()?),
            None => (s, 0),
        };
        let (whole, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        let digits = |t: &str| !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit());
        if !digits(whole) || !(frac.is_empty() || digits(frac)) {
            return None;
        }
        // All the digits, with the decimal point `point` places from the left,
        // moved by the exponent and then by six more places into micro-units.
        let all: String = [whole, frac].concat();
        let point = i64::try_from(whole.len()).ok()? + i64::from(exponent) + 6;
        if point <= 0 {
            return Some(MicroUsd(0));
        }
        let point = usize::try_from(point).ok()?;
        let mut integral: String = all.chars().take(point).collect();
        while integral.len() < point {
            integral.push('0');
        }
        integral.parse().ok().map(MicroUsd)
    }

    #[must_use]
    pub fn micros(self) -> u64 {
        self.0
    }

    #[must_use]
    pub fn from_micros(micros: u64) -> Self {
        MicroUsd(micros)
    }
}

impl std::fmt::Display for MicroUsd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{:06}", self.0 / 1_000_000, self.0 % 1_000_000)
    }
}

/// What one session did, and what it started with.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Session {
    events: Vec<SessionEvent>,
    loaded: Vec<String>,
    skills_invoked: Vec<String>,
    cost: Option<MicroUsd>,
    tokens: Option<u64>,
}

impl Session {
    /// The session's events, in order.
    #[must_use]
    pub fn events(&self) -> &[SessionEvent] {
        &self.events
    }

    /// What the `init` message listed, as `skill:<n>`, `plugin:<n>` and
    /// `mcp:<n>`, in that order.
    #[must_use]
    pub fn loaded(&self) -> &[String] {
        &self.loaded
    }

    /// The skills the model opened with the `Skill` tool, in order.
    #[must_use]
    pub fn skills_invoked(&self) -> &[String] {
        &self.skills_invoked
    }

    /// The client-side cost estimate the CLI reported, if it reported one.
    #[must_use]
    pub fn cost(&self) -> Option<MicroUsd> {
        self.cost
    }

    /// Input, output, cache-read and cache-write tokens, summed.
    #[must_use]
    pub fn tokens(&self) -> Option<u64> {
        self.tokens
    }
}

/// Parse a whole transcript. Blank lines are skipped; anything else that is
/// not a JSON object is an error.
///
/// # Errors
/// [`TranscriptError`] naming the first line that could not be read.
pub fn session_from_stream_json(stream: &str) -> Result<Session, TranscriptError> {
    let mut session = Session::default();
    for (i, line) in stream.lines().enumerate() {
        let line_no = i + 1;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line).map_err(|e| TranscriptError::NotJson {
            line: line_no,
            reason: e.to_string(),
        })?;
        if !value.is_object() {
            return Err(TranscriptError::NotJson {
                line: line_no,
                reason: "not an object".into(),
            });
        }
        match value["type"].as_str() {
            Some("assistant") => read_assistant(&value, &mut session),
            Some("system") if value["subtype"].as_str() == Some("init") => {
                read_init(&value, &mut session);
            }
            Some("result") => read_result(&value, line_no, &mut session)?,
            _ => {}
        }
    }
    Ok(session)
}

fn read_assistant(value: &Value, session: &mut Session) {
    let Some(blocks) = value["message"]["content"].as_array() else {
        return;
    };
    for block in blocks {
        match block["type"].as_str() {
            Some("text") => {
                if let Some(text) = block["text"].as_str() {
                    session
                        .events
                        .push(SessionEvent::new(EventKind::Text, text));
                }
            }
            Some("tool_use") => read_tool(block, session),
            _ => {}
        }
    }
}

fn read_tool(block: &Value, session: &mut Session) {
    let input = &block["input"];
    let (kind, field) = match block["name"].as_str() {
        Some("Bash" | "PowerShell") => (EventKind::Shell, "command"),
        Some("Write" | "Edit" | "MultiEdit") => (EventKind::Write, "file_path"),
        Some("NotebookEdit") => (EventKind::Write, "notebook_path"),
        Some("Read") => (EventKind::Read, "file_path"),
        Some("Skill") => {
            if let Some(name) = skill_named(input) {
                session.skills_invoked.push(name.to_owned());
            }
            return;
        }
        _ => return,
    };
    if let Some(subject) = input[field].as_str() {
        let subject = match kind {
            EventKind::Write | EventKind::Read => subject.replace('\\', "/"),
            EventKind::Shell | EventKind::Text => subject.to_owned(),
        };
        session.events.push(SessionEvent::new(kind, subject));
    }
}

/// The skill a `Skill` call opened, as invoked: `input.skill`, or
/// `input.command` where an older CLI wrote that. One definition for every
/// reader of a Skill call — a stream-json `tool_use.input` here, and a hook's
/// `tool_input`, which the hooks reference describes as the same arguments.
/// The load-log pack's fixture runner is generated from this function, so its
/// `jq` and this code cannot read different names from the same payload.
#[must_use]
pub fn skill_named(input: &Value) -> Option<&str> {
    input["skill"].as_str().or(input["command"].as_str())
}

/// A list field of the `init` message, as names: a string entry is its own
/// name, an object entry contributes its `name`.
fn names(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().or(item["name"].as_str()))
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn read_init(value: &Value, session: &mut Session) {
    for (field, prefix) in [
        ("skills", "skill"),
        ("plugins", "plugin"),
        ("mcp_servers", "mcp"),
    ] {
        for name in names(&value[field]) {
            session.loaded.push(format!("{prefix}:{name}"));
        }
    }
}

fn read_result(value: &Value, line: usize, session: &mut Session) -> Result<(), TranscriptError> {
    if let Some(number) = value.get("total_cost_usd").filter(|v| !v.is_null()) {
        let text = number.to_string();
        session.cost = Some(
            MicroUsd::parse_decimal(&text).ok_or(TranscriptError::Cost { line, value: text })?,
        );
    }
    let usage = &value["usage"];
    if usage.is_object() {
        let total = [
            "input_tokens",
            "output_tokens",
            "cache_read_input_tokens",
            "cache_creation_input_tokens",
        ]
        .iter()
        .filter_map(|k| usage[*k].as_u64())
        .sum();
        session.tokens = Some(total);
    }
    Ok(())
}
