//! What a Claude Code settings file registers as a hook — and nothing else.
//!
//! A settings file holds a permission allowlist and other machine-local,
//! sometimes secret-bearing configuration (`[R:no-secrets-in-config-repo]`).
//! This reads the `hooks` table and keeps only the command strings under it;
//! nothing else in the file is retained, and nothing from it is ever printed
//! but the file name of a hook script the corpus already names.
//!
//! The shape is read strictly. A `hooks` table that is not the documented
//! `event -> [ { matcher, hooks: [ { type, command } ] } ]` is refused rather
//! than read as empty, because "empty" would turn every hook into
//! `RESOLVED_NOT_WIRED` — a confident answer built on a file the tool did not
//! understand. Reading the JSON as JSON, rather than searching its text, is
//! what keeps a script named in `permissions` or `statusLine` from counting as
//! registered.

use serde_json::Value;

use super::scan::names;

/// Why a settings file could not be read as hook registrations.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SettingsError {
    /// Not JSON.
    #[error("the settings file is not valid JSON: {0}")]
    Json(String),
    /// JSON, but `hooks` is not the shape Claude Code reads.
    #[error("the settings file's `{at}` is not {expected}")]
    Shape {
        /// Where in the document, as a path.
        at: String,
        /// What was expected there.
        expected: &'static str,
    },
}

/// Every hook command a settings file registers, for any event and matcher.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HookRegistry {
    commands: Vec<String>,
}

fn shape(at: impl Into<String>, expected: &'static str) -> SettingsError {
    SettingsError::Shape {
        at: at.into(),
        expected,
    }
}

impl HookRegistry {
    /// Read the hook registrations out of a settings document. A document with
    /// no `hooks` key registers nothing, which is a true statement about it.
    pub fn parse(json: &str) -> Result<Self, SettingsError> {
        let doc: Value =
            serde_json::from_str(json).map_err(|e| SettingsError::Json(e.to_string()))?;
        let Some(hooks) = doc.get("hooks") else {
            return Ok(HookRegistry::default());
        };
        let events = hooks
            .as_object()
            .ok_or_else(|| shape("hooks", "an object of events"))?;
        let mut commands = Vec::new();
        for (event, groups) in events {
            let groups = groups
                .as_array()
                .ok_or_else(|| shape(format!("hooks.{event}"), "an array of matcher groups"))?;
            for (g, group) in groups.iter().enumerate() {
                let entries = group
                    .get("hooks")
                    .and_then(Value::as_array)
                    .ok_or_else(|| shape(format!("hooks.{event}[{g}].hooks"), "an array"))?;
                for (h, entry) in entries.iter().enumerate() {
                    // A hook of another type (a prompt, say) runs no script, so
                    // it registers nothing this module could be asked about.
                    if entry
                        .get("type")
                        .and_then(Value::as_str)
                        .is_some_and(|t| t != "command")
                    {
                        continue;
                    }
                    let command =
                        entry
                            .get("command")
                            .and_then(Value::as_str)
                            .ok_or_else(|| {
                                shape(format!("hooks.{event}[{g}].hooks[{h}].command"), "a string")
                            })?;
                    commands.push(command.to_owned());
                }
            }
        }
        Ok(HookRegistry { commands })
    }

    /// Whether some registered command runs the script with this file name.
    /// The match is on the file name as a whole path component — `~` and
    /// `$HOME` are spelled differently on every machine, and expanding them
    /// would be reading the machine rather than the file.
    #[must_use]
    pub fn registers(&self, file_name: &str) -> bool {
        // A Windows command line may spell the path with backslashes; the token
        // is always forward-slash, as every path in the map is.
        self.commands
            .iter()
            .any(|c| names(&c.replace('\\', "/"), file_name))
    }
}
