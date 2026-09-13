//! Writing a rule file without first asking whether it may be edited must not
//! compile.
//!
//! This is the refusal's mechanism, pinned as a negative. `fsio::write_rule`
//! takes an `EditableRule`, whose only constructor refuses a cached rule — so a
//! write path added later cannot reach the filesystem by passing the `&Rule` it
//! happens to be holding. It has to mint the witness, and minting it is where
//! the question gets asked.
//!
//! A check *inside* `write_rule` would compile this file happily and pass its
//! own test. It is the second write path — the one nobody has written yet — that
//! the witness is for.

use relearn::fsio;
use relearn::rule::{
    Authority, Body, Date, ErrorClass, Home, Incident, Origin, Rule, RuleTag, Status, Title,
};

fn main() {
    let rule = Rule::new(
        RuleTag::parse("R:x").expect("valid tag"),
        Title::parse("A title").expect("non-empty title"),
        ErrorClass::parse("ec").expect("non-empty error class"),
        Home::global(),
        Date::parse("2026-09-13").expect("valid date"),
        Origin::Mined,
        Status::active(),
        Incident::parse("an incident").expect("non-empty incident"),
        Body::parse("The body.").expect("non-empty body"),
        Vec::new(),
        Vec::new(),
        Authority::local(),
        None,
    );

    // Even a *local* rule cannot take this shortcut: the witness is the only
    // way in, so the question is asked for every rule rather than for the ones
    // somebody remembered to check.
    let _ = fsio::write_rule(std::path::Path::new("rules"), &rule);
}
