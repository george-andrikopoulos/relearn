//! `relearn` binary — a thin shell over [`relearn::cli`]. All logic lives in
//! the library so it is testable without spawning a process.
//!
//! **Must NOT:** contain business logic — dispatch only.

use std::process::ExitCode;

fn main() -> ExitCode {
    relearn::cli::run()
}
