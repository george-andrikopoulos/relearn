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
