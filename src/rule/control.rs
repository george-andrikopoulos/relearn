//! `Controls` — what holds a rule, as a typed list rather than a sentence.
//!
//! A graduated rule names the control that superseded its prose, and a partly
//! held one names the controls covering part of its class. Three rules in this
//! corpus name **two** controls, written as one string joined by `" + "`, some
//! with a parenthesised note saying which half each covers. That convention was
//! load-bearing and enforced by nothing: `to = "garbage"` parsed happily,
//! `report` re-split the string on whitespace to guess at control kinds, and a
//! mistyped prefix produced a rule whose control kind the federation could not
//! see — silently, because there was nothing to disagree with.
//!
//! So the sentence is parsed **once, at the perimeter**, into this module's
//! types, and a malformed one stops the build like any other bad field
//! (`[R:parse-dont-validate]`). The on-disk format is unchanged: the neutral
//! format stays hand-editable and diff-friendly, and what travels between
//! installs is the same text it always was. [`Controls`]'s `Display` is the
//! exact inverse of [`Controls::parse`], which is what makes that true and is
//! pinned over the real corpus in `tests/control.rs`.

use std::fmt;

/// The separator between two controls in a written destination.
const SEPARATOR: &str = " + ";

/// Why a destination could not be read as a list of controls.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ControlError {
    /// No controls at all — a graduation that names nothing is not a graduation.
    #[error("a destination must name at least one control")]
    Empty,

    /// An entry with no `kind:` prefix.
    #[error(
        "control `{0}` has no `kind:` prefix (expected one of type | property | test | gate | hook)"
    )]
    NoKind(String),

    /// A prefix outside the closed vocabulary.
    #[error("control kind `{0}` is not one of type | property | test | gate | hook")]
    UnknownKind(String),

    /// A prefix with nothing after it.
    #[error("control `{0}:` names no artefact")]
    NoName(String),

    /// `name ()` — a coverage note that says nothing.
    #[error("control `{0}` has an empty coverage note")]
    EmptyCoverage(String),
}

/// The kind of control that holds a rule — a closed vocabulary, because the
/// federation's recurrence report publishes it and a free-text kind would make
/// the cross-install numbers unaddable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ControlKind {
    /// A type makes the mistake unrepresentable.
    Type,
    /// A property test states a law over generated input.
    PropertyTest,
    /// A unit test pins a past bug.
    UnitTest,
    /// A gate in a build or CI run.
    Gate,
    /// A hook at write time.
    Hook,
}

impl ControlKind {
    /// The prefix as a rule file writes it, before the colon.
    ///
    /// Deliberately different from [`Self::published`] for the two test kinds:
    /// rule files have always written `test:`, and every existing file has to
    /// stay byte-identical, while a report needs to tell a unit test from a
    /// property test. One concept, two spellings, both exhaustive.
    #[must_use]
    pub fn prefix(self) -> &'static str {
        match self {
            ControlKind::Type => "type",
            ControlKind::PropertyTest => "property",
            ControlKind::UnitTest => "test",
            ControlKind::Gate => "gate",
            ControlKind::Hook => "hook",
        }
    }

    /// The spelling the recurrence report publishes.
    #[must_use]
    pub fn published(self) -> &'static str {
        match self {
            ControlKind::Type => "type",
            ControlKind::PropertyTest => "property-test",
            ControlKind::UnitTest => "unit-test",
            ControlKind::Gate => "gate",
            ControlKind::Hook => "hook",
        }
    }

    fn from_prefix(s: &str) -> Option<Self> {
        match s {
            "type" => Some(ControlKind::Type),
            "property" => Some(ControlKind::PropertyTest),
            "test" => Some(ControlKind::UnitTest),
            "gate" => Some(ControlKind::Gate),
            "hook" => Some(ControlKind::Hook),
            _ => None,
        }
    }
}

/// One control: its kind, the artefact's name, and optionally which part of the
/// rule's class it covers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Control {
    kind: ControlKind,
    name: String,
    covers: Option<String>,
}

impl Control {
    /// The kind of control.
    #[must_use]
    pub fn kind(&self) -> ControlKind {
        self.kind
    }

    /// The artefact's name, as written — a hook name, a test path, a gate
    /// function. Names contain spaces (`verify.sh emoji_ban`) and paths
    /// (`tests/ledger.rs::a_test`), so this is text and not a further grammar.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Which part of the rule's class this control covers, if the author said.
    #[must_use]
    pub fn covers(&self) -> Option<&str> {
        self.covers.as_deref()
    }

    /// Parse one written control: `kind:name`, or `kind:name (covers)`.
    fn parse_one(entry: &str) -> Result<Self, ControlError> {
        let entry = entry.trim();
        let (prefix, rest) = entry
            .split_once(':')
            .ok_or_else(|| ControlError::NoKind(entry.to_owned()))?;
        let prefix = prefix.trim();
        let kind = ControlKind::from_prefix(prefix)
            .ok_or_else(|| ControlError::UnknownKind(prefix.to_owned()))?;

        // A trailing parenthesised group is the coverage note, and it is found
        // from the END. Names legitimately contain spaces, dots, `::` and even
        // parentheses, so splitting on the first `(` or on whitespace would eat
        // part of the name — which is exactly what the ad-hoc reader in `report`
        // used to do to every name that had a space in it.
        let rest = rest.trim();
        let split = rest.strip_suffix(')').and_then(|head| {
            head.rfind('(')
                .map(|at| (head[..at].trim(), head[at + 1..].trim()))
        });
        let (name, covers) = match split {
            Some((name, note)) => (name, Some(note)),
            None => (rest, None),
        };

        if name.is_empty() {
            return Err(ControlError::NoName(prefix.to_owned()));
        }
        if covers.is_some_and(str::is_empty) {
            return Err(ControlError::EmptyCoverage(name.to_owned()));
        }
        Ok(Control {
            kind,
            name: name.to_owned(),
            covers: covers.map(ToOwned::to_owned),
        })
    }
}

impl fmt::Display for Control {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.kind.prefix(), self.name)?;
        match &self.covers {
            Some(covers) => write!(f, " ({covers})"),
            None => Ok(()),
        }
    }
}

/// A non-empty list of the controls that hold a rule.
///
/// Non-empty by construction: "graduated to nothing" and "partly held by
/// nothing" are both just `Active`, and letting them be spelled a second way
/// would put two names on one state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Controls(Vec<Control>);

impl Controls {
    /// Parse a written destination: one or more controls joined by `" + "`.
    ///
    /// All-or-nothing. A half-read destination would emit a note naming one
    /// control while the rule believes it has two, which is the shape of false
    /// claim this repository exists to refuse.
    pub fn parse(s: impl AsRef<str>) -> Result<Self, ControlError> {
        let s = s.as_ref().trim();
        if s.is_empty() {
            return Err(ControlError::Empty);
        }
        let controls = s
            .split(SEPARATOR)
            .map(Control::parse_one)
            .collect::<Result<Vec<_>, _>>()?;
        if controls.is_empty() {
            return Err(ControlError::Empty);
        }
        Ok(Controls(controls))
    }

    /// The controls, in the order the author wrote them.
    #[must_use]
    pub fn as_slice(&self) -> &[Control] {
        &self.0
    }

    /// Every distinct control kind named here, sorted and deduplicated.
    ///
    /// The federation's report asks this. It used to be answered by splitting
    /// the raw string on whitespace and reading the text before each colon,
    /// which silently dropped any name containing a space.
    #[must_use]
    pub fn kinds(&self) -> Vec<ControlKind> {
        let mut kinds: Vec<ControlKind> = self.0.iter().map(Control::kind).collect();
        kinds.sort_unstable();
        kinds.dedup();
        kinds
    }
}

impl fmt::Display for Controls {
    /// Renders exactly the text it was parsed from, which is what keeps every
    /// rule file and every emitted instruction file byte-identical across the
    /// change that introduced this type.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (at, control) in self.0.iter().enumerate() {
            if at > 0 {
                write!(f, "{SEPARATOR}")?;
            }
            write!(f, "{control}")?;
        }
        Ok(())
    }
}

impl Controls {
    /// The one kind of control here, if every control is the same kind.
    ///
    /// The federation's report publishes this and nothing else about a
    /// graduation: `gate:internal-payments-lint` publishes `gate`, and the rest
    /// of that string never leaves the machine. `None` when two kinds are
    /// named, because choosing one of them would be inventing a fact.
    #[must_use]
    pub fn sole_kind(&self) -> Option<ControlKind> {
        match self.kinds().as_slice() {
            [only] => Some(*only),
            _ => None,
        }
    }
}
