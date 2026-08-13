//! `emit::claude_md` — emits `<out>/CLAUDE.md`, project-layer rules only
//! (`Home::Project`), so a project's `CLAUDE.md` never absorbs global or
//! domain rules that belong in their own home.
//!
//! **Must NOT:** read the filesystem, or read anything not carried by the rule.
