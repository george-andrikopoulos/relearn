//! `emit::copilot` — emits a single `<out>/.github/copilot-instructions.md`,
//! rules ordered by home then tag.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.
