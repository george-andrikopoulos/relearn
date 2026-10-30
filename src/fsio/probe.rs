//! The reads `relearn probe` needs: a transcript, a results file, and a final
//! workspace compared against the fixture it started from.
//!
//! A probe run writes into temporary directories outside this repository; these
//! functions only read what the caller names on the command line.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::rule::WorkspaceChange;

/// Directories a workspace comparison never descends into: the arm's own
/// emitted layer (which the fixture never holds, so it would read as a change
/// in every WITH session), build output, and version control.
const NOT_THE_WORK: &[&str] = &[".claude", "target", ".git"];

/// Why a probe input could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ProbeReadError {
    #[error("cannot read {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{path} is not a directory")]
    NotADirectory { path: PathBuf },
}

/// Read a whole text file.
///
/// # Errors
/// [`ProbeReadError::Read`] naming the path.
pub fn read_text(path: &Path) -> Result<String, ProbeReadError> {
    fs::read_to_string(path).map_err(|source| ProbeReadError::Read {
        path: path.to_owned(),
        source,
    })
}

/// Every file under `root`, by relative `/` path, with its bytes.
fn tree(root: &Path) -> Result<BTreeMap<String, Vec<u8>>, ProbeReadError> {
    if !root.is_dir() {
        return Err(ProbeReadError::NotADirectory {
            path: root.to_owned(),
        });
    }
    let mut out = BTreeMap::new();
    let mut stack = vec![(root.to_owned(), String::new())];
    while let Some((dir, prefix)) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|source| ProbeReadError::Read {
            path: dir.clone(), // allow:clone: the error owns the path it names, and `dir` is dropped with the frame
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| ProbeReadError::Read {
                path: dir.clone(), // allow:clone: as above, the error outlives the frame
                source,
            })?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let rel = if prefix.is_empty() {
                name.clone() // allow:clone: `name` is still needed for the skip test below
            } else {
                format!("{prefix}/{name}")
            };
            let path = entry.path();
            if path.is_dir() {
                if !(prefix.is_empty() && NOT_THE_WORK.contains(&name.as_str())) {
                    stack.push((path, rel));
                }
            } else {
                let bytes = fs::read(&path).map_err(|source| ProbeReadError::Read {
                    path: path.clone(), // allow:clone: the error owns the path; `path` dies with the iteration
                    source,
                })?;
                out.insert(rel, bytes);
            }
        }
    }
    Ok(out)
}

/// Compare a session's final workspace with the fixture it started from.
/// `None` for the fixture is an empty start, so every file is new.
///
/// # Errors
/// [`ProbeReadError`] if either tree cannot be read.
pub fn workspace_change(
    fixture: Option<&Path>,
    workspace: &Path,
) -> Result<WorkspaceChange, ProbeReadError> {
    let before = match fixture {
        Some(f) => tree(f)?,
        None => BTreeMap::new(),
    };
    let after = tree(workspace)?;
    let mut changed: Vec<String> = after
        .iter()
        .filter(|(path, bytes)| before.get(*path) != Some(*bytes))
        .map(|(path, _)| path.clone()) // allow:clone: the change owns its paths, outliving both trees
        .collect();
    changed.extend(before.keys().filter(|p| !after.contains_key(*p)).cloned());
    changed.sort();
    let files = after
        .into_iter()
        .map(|(path, bytes)| (path, String::from_utf8_lossy(&bytes).into_owned()))
        .collect();
    Ok(WorkspaceChange::new(changed, files))
}
