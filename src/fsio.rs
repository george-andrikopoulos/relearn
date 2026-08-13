//! `fsio` — all filesystem reads (loading `rules/*.md`) and writes (emitting
//! target files), including the generated-file guard that refuses to overwrite
//! a target lacking the generated-by header and matching hash
//! (`[R:generate-guards-unversioned]`).
//!
//! **Must NOT:** contain business logic. It moves bytes and prepends file names
//! to diagnostics; every decision about *what* a document means is made in
//! `rule` (parsing) and every decision about *what* to write is made in `emit`.

use std::fs;
use std::path::{Path, PathBuf};

use crate::library::{Library, Unvalidated};
use crate::rule::{ParseError, parse_document};

/// Why loading a rules directory failed. Every variant names the file (or
/// directory) at fault, so a diagnostic points the operator straight at it.
#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    /// The rules directory could not be read.
    #[error("reading directory {path:?}: {source}")]
    ReadDir {
        /// The directory that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A rule file could not be read.
    #[error("reading {path:?}: {source}")]
    Io {
        /// The file that could not be read.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// A rule file was read but did not parse. Carries the field-level
    /// `ParseError` and the file it came from — the "file + field" diagnostic.
    #[error("{path:?}: {source}")]
    Parse {
        /// The file that failed to parse.
        path: PathBuf,
        /// The field-level parse error.
        #[source]
        source: ParseError,
    },
}

/// Load every `*.md` file in `dir` into an unvalidated library, in sorted file
/// order for determinism. A file that cannot be read or parsed stops the load
/// with a diagnostic naming it — a bad file is never silently skipped.
///
/// The result is `Library<Unvalidated>`; the caller runs `validate` to check
/// cross-file invariants (tag uniqueness) and reach `Library<Validated>`.
pub fn load_rules(dir: &Path) -> Result<Library<Unvalidated>, LoadError> {
    let read = fs::read_dir(dir).map_err(|source| LoadError::ReadDir {
        path: dir.to_path_buf(),
        source,
    })?;

    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in read {
        let entry = entry.map_err(|source| LoadError::ReadDir {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            paths.push(path);
        }
    }
    paths.sort();

    let mut library = Library::new();
    for path in paths {
        // `path` is moved into the error on either failure path and dropped on
        // success — so it is never cloned.
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(source) => return Err(LoadError::Io { path, source }),
        };
        let rule = match parse_document(&content) {
            Ok(rule) => rule,
            Err(source) => return Err(LoadError::Parse { path, source }),
        };
        library.push(rule);
    }
    Ok(library)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule_doc(tag: &str) -> String {
        format!(
            "+++\n\
             tag = \"{tag}\"\n\
             title = \"t\"\n\
             error_class = \"e\"\n\
             home = {{ kind = \"global\" }}\n\
             created = \"2026-01-01\"\n\
             status = {{ kind = \"active\" }}\n\
             incident = \"i\"\n\
             +++\n\n\
             Body.\n"
        )
    }

    #[test]
    fn loads_md_files_in_sorted_order_ignoring_others() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("b.md"), rule_doc("R:beta")).expect("write b");
        fs::write(dir.path().join("a.md"), rule_doc("R:alpha")).expect("write a");
        fs::write(dir.path().join("notes.txt"), "not a rule").expect("write notes");

        let library = load_rules(dir.path()).expect("load succeeds");
        let validated = library.validate().expect("distinct tags validate");
        assert_eq!(validated.len(), 2);
        // Sorted by file name: a.md (R:alpha) before b.md (R:beta).
        assert_eq!(validated.rules()[0].tag().as_str(), "R:alpha");
        assert_eq!(validated.rules()[1].tag().as_str(), "R:beta");
    }

    #[test]
    fn a_bad_file_stops_the_load_and_names_it() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("ok.md"), rule_doc("R:ok")).expect("write ok");
        fs::write(dir.path().join("broken.md"), "not a rule at all").expect("write broken");

        match load_rules(dir.path()) {
            Err(LoadError::Parse { path, .. }) => assert!(path.ends_with("broken.md")),
            other => panic!("expected a Parse error naming broken.md, got {other:?}"),
        }
    }

    #[test]
    fn missing_directory_is_a_readdir_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("does-not-exist");
        assert!(matches!(
            load_rules(&missing),
            Err(LoadError::ReadDir { .. })
        ));
    }

    #[test]
    fn empty_directory_loads_an_empty_library() {
        let dir = tempfile::tempdir().expect("tempdir");
        let library = load_rules(dir.path()).expect("load succeeds");
        assert!(library.validate().expect("validate").is_empty());
    }
}
