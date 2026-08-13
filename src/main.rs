//! `relearn` binary — a thin shell over [`relearn::cli`]. All logic lives in
//! the library so it is testable without spawning a process.
//!
//! **Must NOT:** contain business logic — dispatch only.

fn main() {
    // Scaffold: no commands are wired yet. The first implementation session
    // starts with the types (TODO.md Phase A: `RuleTag`, `Home`, `Status`,
    // `Incident`, `ErrorClass`, and the `Library<Unvalidated>/<Validated>`
    // typestate), then property tests and pins — never an emitter first.
}
