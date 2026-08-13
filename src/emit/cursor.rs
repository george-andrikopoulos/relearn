//! `emit::cursor` — emits `<out>/.cursor/rules/<tag>.mdc` with YAML
//! front-matter (`description`, `globs`, `alwaysApply`). See TODO.md open
//! question: whether `globs`/`alwaysApply` are target-specific optional fields
//! on the rule or derived from `Home` — resolve before implementing.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.
