//! The reads `relearn budget` needs: find every instruction file a Claude Code
//! session started in `project` would load, the way the loader finds them, and
//! read each one. Read-only, and it moves bytes only — what they cost and what
//! is wrong with them is decided in [`crate::loadset`].
//!
//! The loader's order, from its documentation and its code (decisions log,
//! 2026-10-31): the user's `CLAUDE.md` and `rules/**`, then each directory from
//! the filesystem root down to the project — `CLAUDE.md`, `.claude/CLAUDE.md`,
//! `.claude/rules/**`, `CLAUDE.local.md` — and every `@import` of each, to four
//! hops, resolved against the importing file. A file reached twice is the same
//! file; [`crate::loadset::LoadSet`] keeps the first.

use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::loadset::{Authorship, LayerSlug, LoadedFile, Residency, Share, read_layer_text};

/// The loader's documented import depth: "a maximum depth of four hops".
const MAX_IMPORT_HOPS: usize = 4;

/// Why the load set could not be read.
#[derive(Debug, thiserror::Error)]
pub enum LoadSetReadError {
    /// A file or directory that exists could not be read.
    #[error("reading {path:?}: {source}")]
    Io {
        /// What could not be read.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// A root given on the command line is not a directory.
    #[error("{path:?} is not a directory")]
    NotADirectory {
        /// The path given.
        path: PathBuf,
    },
}

/// Which directories above the project to walk.
#[derive(Debug, Clone, Copy)]
pub enum Ancestors<'a> {
    /// None: the project tree only.
    Skip,
    /// Every directory up to the filesystem root, as the loader does.
    All,
    /// Directories strictly below `stop` — a bounded walk, for a tree built
    /// somewhere whose real ancestors are not part of the question.
    Below(&'a Path),
}

/// Find and read every file a session in `project` loads.
///
/// `user` is the user configuration directory (`~/.claude`); `None` means the
/// user share is not measured, and [`crate::loadset::Coverage`] carries that.
pub fn discover_load_set(
    project: &Path,
    user: Option<&Path>,
    ancestors: Ancestors<'_>,
) -> Result<Vec<LoadedFile>, LoadSetReadError> {
    let project = canonical_dir(project)?;
    let user = user.map(canonical_dir).transpose()?;
    let mut walk = Walk {
        files: Vec::new(),
        user: user.as_deref(),
        project: &project,
    };

    if let Some(user) = &user {
        walk.file(&user.join("CLAUDE.md"), Share::User, 0)?;
        walk.rules_dir(&user.join("rules"), Share::User)?;
    }
    for dir in ancestor_dirs(&project, ancestors)? {
        walk.directory(&dir, Share::Ancestor)?;
    }
    walk.directory(&project, Share::Project)?;
    Ok(walk.files)
}

struct Walk<'a> {
    files: Vec<LoadedFile>,
    user: Option<&'a Path>,
    project: &'a Path,
}

impl Walk<'_> {
    /// One directory's four locations, in the loader's order.
    fn directory(&mut self, dir: &Path, share: Share) -> Result<(), LoadSetReadError> {
        self.file(&dir.join("CLAUDE.md"), share, 0)?;
        self.file(&dir.join(".claude").join("CLAUDE.md"), share, 0)?;
        self.rules_dir(&dir.join(".claude").join("rules"), share)?;
        self.file(&dir.join("CLAUDE.local.md"), share, 0)
    }

    /// Every `.md` under a rules directory, recursively, in path order.
    fn rules_dir(&mut self, dir: &Path, share: Share) -> Result<(), LoadSetReadError> {
        let mut found = Vec::new();
        collect_md(dir, &mut found)?;
        found.sort();
        for path in found {
            self.file(&path, share, 0)?;
        }
        Ok(())
    }

    /// Read one file if it exists, record it, and follow its imports.
    fn file(&mut self, path: &Path, share: Share, hops: usize) -> Result<(), LoadSetReadError> {
        self.file_scoped(path, share, hops, None)
    }

    fn file_scoped(
        &mut self,
        path: &Path,
        share: Share,
        hops: usize,
        inherited: Option<&Residency>,
    ) -> Result<(), LoadSetReadError> {
        // The loader ignores a location that holds no file; so does the walk.
        if !path.is_file() {
            return Ok(());
        }
        let raw = fs::read_to_string(path).map_err(|source| LoadSetReadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let key = fs::canonicalize(path).map_err(|source| LoadSetReadError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if self.files.iter().any(|f| f.key() == key) {
            return Ok(());
        }
        let (own, chars, generated, imports) = read_layer_text(&raw).into_parts();
        // An import of a path-scoped file attaches when its importer does.
        let residency = match (inherited, own) {
            (Some(scoped @ Residency::PathScoped(_)), _) => scoped.clone(), // allow:clone: each imported file owns its residency
            (_, own) => own,
        };
        let origin = if generated {
            Authorship::Generated(
                key.file_stem()
                    .and_then(|s| s.to_str())
                    .and_then(LayerSlug::parse),
            )
        } else {
            Authorship::HandAuthored
        };
        let shown = self.shown(&key);
        self.files.push(LoadedFile::new(
            key.clone(), // allow:clone: the record owns its key; the walk still needs it to resolve imports
            shown,
            share,
            residency.clone(), // allow:clone: the record owns its residency; imports inherit it below
            chars,
            origin,
        ));

        if hops >= MAX_IMPORT_HOPS {
            return Ok(());
        }
        let base = key.parent().unwrap_or(&key).to_path_buf();
        for import in imports {
            if let Some(target) = self.resolve_import(&base, &import) {
                self.file_scoped(&target, share, hops + 1, Some(&residency))?;
            }
        }
        Ok(())
    }

    /// Where an `@import` points: relative to the importing file, absolute as
    /// written, or `~/…` under the user directory's parent. A `~` import with
    /// no user directory given is not resolved, because resolving it would
    /// mean reading the machine's home directory, which nobody named.
    fn resolve_import(&self, base: &Path, import: &str) -> Option<PathBuf> {
        if let Some(rest) = import.strip_prefix("~/") {
            return self.user.and_then(Path::parent).map(|home| home.join(rest));
        }
        let path = Path::new(import);
        Some(if path.is_absolute() {
            path.to_path_buf()
        } else {
            base.join(path)
        })
    }

    /// How a report names a file: relative to the user or project directory
    /// when it is under one, otherwise the full path.
    fn shown(&self, key: &Path) -> String {
        let under = |root: &Path, label: &str| {
            key.strip_prefix(root)
                .ok()
                .map(|rel| format!("{label}:{}", slashed(rel)))
        };
        self.user
            .and_then(|u| under(u, "user"))
            .or_else(|| under(self.project, "project"))
            .unwrap_or_else(|| slashed(&display_path(key)))
    }
}

fn collect_md(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), LoadSetReadError> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::NotADirectory) => {
            return Ok(());
        }
        Err(source) => {
            return Err(LoadSetReadError::Io {
                path: dir.to_path_buf(),
                source,
            });
        }
    };
    for entry in entries {
        let path = entry
            .map_err(|source| LoadSetReadError::Io {
                path: dir.to_path_buf(),
                source,
            })?
            .path();
        if path.is_dir() {
            collect_md(&path, out)?;
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
    Ok(())
}

fn canonical_dir(path: &Path) -> Result<PathBuf, LoadSetReadError> {
    let canonical = fs::canonicalize(path).map_err(|source| LoadSetReadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if canonical.is_dir() {
        Ok(canonical)
    } else {
        Err(LoadSetReadError::NotADirectory {
            path: path.to_path_buf(),
        })
    }
}

/// The project's ancestors, from the root down, excluding the project.
fn ancestor_dirs(
    project: &Path,
    ancestors: Ancestors<'_>,
) -> Result<Vec<PathBuf>, LoadSetReadError> {
    let stop = match ancestors {
        Ancestors::Skip => return Ok(Vec::new()),
        Ancestors::All => None,
        Ancestors::Below(stop) => Some(canonical_dir(stop)?),
    };
    let mut dirs: Vec<PathBuf> = project
        .ancestors()
        .skip(1)
        .take_while(|d| stop.as_deref() != Some(*d))
        .map(Path::to_path_buf)
        .collect();
    dirs.reverse();
    Ok(dirs)
}

/// A canonical Windows path without its `\\?\` verbatim prefix, for reading.
fn display_path(path: &Path) -> PathBuf {
    let mut components = path.components();
    match components.next() {
        Some(Component::Prefix(prefix)) => {
            let text = prefix.as_os_str().to_string_lossy();
            let plain = text.strip_prefix(r"\\?\").unwrap_or(&text).to_owned();
            let mut out = PathBuf::from(plain);
            out.push(std::path::MAIN_SEPARATOR_STR);
            out.extend(components.filter(|c| !matches!(c, Component::RootDir)));
            out
        }
        _ => path.to_path_buf(),
    }
}

fn slashed(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
