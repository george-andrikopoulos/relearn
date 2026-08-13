//! `emit::claude` — emits `<out>/skills/<name>/SKILL.md` with YAML
//! front-matter (`name`, `description`), the description built from the rule's
//! title and error class so the skill's trigger coverage matches the rule.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.
