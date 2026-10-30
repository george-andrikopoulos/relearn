//! The reads `relearn controls` needs: the map, a settings file, and what is at
//! each path the map names under each supplied root. Read-only, and it moves
//! bytes only — every decision about what they mean is made in `resolve`.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::resolve::{
    ControlMap, Found, HookRegistry, MapError, RelPath, RootName, SettingsError, Snapshot,
};

/// Directories never searched for a test function: build output and VCS
/// metadata are not where a test is defined, and `target/` holds copies.
const SKIPPED_DIRS: &[&str] = &["target", ".git", "node_modules"];

/// Why a read for `relearn controls` failed. Each names the path at fault.
#[derive(Debug, thiserror::Error)]
pub enum ControlReadError {
    /// A file or directory could not be read.
    #[error("reading {path:?}: {source}")]
    Io {
        /// What could not be read.
        path: PathBuf,
        /// The underlying error.
        #[source]
        source: std::io::Error,
    },
    /// The control map did not parse.
    #[error("{path:?}: {source}")]
    Map {
        /// The map file.
        path: PathBuf,
        /// Why.
        #[source]
        source: MapError,
    },
    /// The settings file did not parse as hook registrations.
    #[error("{path:?}: {source}")]
    Settings {
        /// The settings file.
        path: PathBuf,
        /// Why.
        #[source]
        source: SettingsError,
    },
    /// A supplied root is not a directory on this machine. Refused rather than
    /// resolved, because every control under it would read `MISSING` — a
    /// verdict about the artefacts when the fault is the path that was typed.
    #[error("root `{root}`: {path:?} is not a directory")]
    RootNotADirectory {
        /// The root's name.
        root: RootName,
        /// The path supplied for it.
        path: PathBuf,
    },
}

fn io(path: &Path) -> impl FnOnce(std::io::Error) -> ControlReadError + '_ {
    move |source| ControlReadError::Io {
        path: path.to_path_buf(),
        source,
    }
}

/// Read and parse the control map.
pub fn read_control_map(path: &Path) -> Result<ControlMap, ControlReadError> {
    let text = fs::read_to_string(path).map_err(io(path))?;
    ControlMap::parse(&text).map_err(|source| ControlReadError::Map {
        path: path.to_path_buf(),
        source,
    })
}

/// Read a settings file's hook registrations.
pub fn read_settings(path: &Path) -> Result<HookRegistry, ControlReadError> {
    let text = fs::read_to_string(path).map_err(io(path))?;
    HookRegistry::parse(&text).map_err(|source| ControlReadError::Settings {
        path: path.to_path_buf(),
        source,
    })
}

/// Read everything the map names under the supplied roots: `here` (the
/// directory holding the map) and each `(name, path)` from the command line.
/// A root the map never names is the caller's to refuse; this reads only what
/// the map asks for.
pub fn snapshot(
    map: &ControlMap,
    here: &Path,
    roots: &[(RootName, PathBuf)],
) -> Result<Snapshot, ControlReadError> {
    let mut located: Vec<(&RootName, &Path)> = vec![(map.here(), here)];
    located.extend(roots.iter().map(|(name, path)| (name, path.as_path())));

    let mut snapshot = Snapshot::new();
    for (name, dir) in &located {
        if !dir.is_dir() {
            return Err(ControlReadError::RootNotADirectory {
                root: (*name).clone(), // allow:clone: the error owns the name it reports, outliving the map borrow
                path: dir.to_path_buf(),
            });
        }
        snapshot.supply((*name).clone()); // allow:clone: the snapshot owns its keys; the map is borrowed only for this call
    }
    for (root, path) in map.reads() {
        let Some((_, dir)) = located.iter().find(|(name, _)| *name == root) else {
            continue;
        };
        snapshot.record(root, path.clone(), read_found(dir, path)?); // allow:clone: as above, the snapshot owns its keys
    }
    Ok(snapshot)
}

fn read_found(root: &Path, path: &RelPath) -> Result<Found, ControlReadError> {
    let full = if path.is_root() {
        root.to_path_buf()
    } else {
        root.join(path.as_str())
    };
    if full.is_dir() {
        let mut files = Vec::new();
        collect_rust(&full, root, &mut files)?;
        files.sort();
        return Ok(Found::Tree(files));
    }
    match fs::read(&full) {
        Ok(bytes) => Ok(Found::File(String::from_utf8_lossy(&bytes).into_owned())),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(Found::Absent),
        Err(source) => Err(ControlReadError::Io { path: full, source }),
    }
}

fn collect_rust(
    dir: &Path,
    root: &Path,
    out: &mut Vec<(String, String)>,
) -> Result<(), ControlReadError> {
    for entry in fs::read_dir(dir).map_err(io(dir))? {
        let path = entry.map_err(io(dir))?.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            if !SKIPPED_DIRS.contains(&name) {
                collect_rust(&path, root, out)?;
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            let bytes = fs::read(&path).map_err(io(&path))?;
            let below = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((below, String::from_utf8_lossy(&bytes).into_owned()));
        }
    }
    Ok(())
}
