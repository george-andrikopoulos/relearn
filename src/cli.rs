//! `cli` — argument parsing (`clap`, derive), command dispatch, and
//! human-readable diagnostics. The planned commands (TODO.md Phase A):
//! `check` (validate, write nothing), `build --targets <list>` (emit named
//! targets; unknown target rejected before any write), `list --home <layer>`.
//!
//! **Must NOT:** contain business logic. It translates arguments into calls on
//! `library`/`emit`/`fsio` and formats their results; the rules of the domain
//! live in those modules. `anyhow` may surface here at the binary edge; library
//! code returns typed errors.
