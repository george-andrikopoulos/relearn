//! Compile-fail pins (rust-typedd tier 1 — the negative): code that MUST NOT
//! compile. This proves the `Library<Validated>` typestate gate is load-bearing
//! — an emitter cannot be called with an unvalidated library. The type already
//! enforces this; trybuild pins the negative so a regression (e.g. relaxing an
//! emitter to accept any `Library<S>`) is caught as a failing test, and it
//! checks *why* the code is rejected (the type mismatch), not merely that it is.
//!
//! The expected-error `.stderr` files are toolchain-sensitive. After a rustc
//! bump, regenerate with:
//!   TRYBUILD=overwrite cargo test --test compile_fail

#[test]
fn typestate_gate_rejects_unvalidated_libraries() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/emit_rejects_unvalidated_library.rs");
}

/// The federation exclusion's mechanism, pinned as a negative: a match on `Home`
/// that does not decide about every variant does not compile. That is what makes
/// "an `Org`-homed rule can never be contributed" a property of the type system
/// rather than a filter somebody has to remember to add to the second publishing
/// path.
///
/// It pins the mechanism, not the future: it cannot prove a variant added in
/// 2027 is classified *correctly*, only that it must be classified at all. The
/// other half — that `Home::federation` itself has no catch-all arm, which would
/// silently classify that variant — is `tests/federation_exclusion.rs`, because
/// a wildcard compiles fine and no compiler can object to it.
#[test]
fn a_match_on_home_must_decide_about_every_variant() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/home_match_must_be_exhaustive.rs");
}
