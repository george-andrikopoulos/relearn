//! A match on `Home` that does not decide about every variant must not compile.
//!
//! This is the mechanism behind the federation exclusion, pinned as a negative.
//! `Home::federation` answers "may a rule in this home leave the machine?" by
//! matching every variant with no catch-all arm, so **adding a home variant is a
//! compile error at every site that has to decide about it** — including that
//! one. A rule cannot inherit a federation answer nobody chose for it.
//!
//! The code below is what the failure looks like from outside the crate: a match
//! written before `Org` existed, which stopped compiling the moment it did. That
//! is the whole guarantee, and it is the reason the exclusion is a match rather
//! than a filter — a filter added to one publishing path compiles perfectly
//! while a second path publishes everything.

use relearn::rule::Home;

fn decide(home: &Home) -> bool {
    match home {
        Home::Global => true,
        Home::Domain { .. } => true,
        Home::Project { .. } => false,
    }
}

fn main() {
    let _ = decide(&Home::global());
}
