+++
tag = "R:price-every-dependency"
title = "Every dependency is priced in the decisions log, and so is every dependency refused"
error_class = "A dependency enters the tree without being recorded as a choice, and the alternative of taking none leaves no artefact at all, so a refused crate is indistinguishable from one nobody considered and is added by the next session"
home = { kind = "global" }
created = "2026-09-12"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-12 out of a design conversation, not out of a failure that cost something. The anchoring observation was measured in the relearn repository the same day. Nine dependencies in the manifest -- six under [dependencies] (toml, serde, clap, thiserror, anyhow, sha2) and three under [dev-dependencies] (proptest, tempfile, trybuild) -- against the ARCHITECTURE decisions log, in which three of the nine names appear at all and exactly one appears as a crate choice: `toml`, and that entry prices a format decision (TOML front-matter, `+++` delimiters) for which the crate's maintenance is the stated reason. `serde` occurs only as the `#[serde(default)]` attribute in a later entry and as the rejected `serde_yaml` in the first; `sha2` occurs only inside another decision's rejected alternative. No dev-dependency appears anywhere in the log, and no entry lists writing the thing by hand among the alternatives refused. The sibling repository Design-Architecture-Tool, whose `printpdf` entry is the worked example of feature-flag pricing, was not reachable from this machine, so the pair is measured from this repository alone."
published_incident = "Codified out of a design conversation rather than a failure that cost something. The anchoring measurement was taken before the rule was written: of nine dependencies in one project's manifests, exactly one was named in the decisions log, and that one only incidentally as the reason for a format choice. The practice: a dependency is priced in the log before it is used — what it buys, what was rejected — because the alternative is a manifest nobody can account for and a refusal nobody recorded. The refusing half has no artefact and therefore no gate; it stays with the person who refused."
+++

Every dependency gets an entry in the decisions log: what it is for, why this one, and
what was refused. The refused list includes **writing it yourself**, with a rough size,
whenever that was a live option -- and it usually was. A crate arrives as one line in a
manifest: no design discussion, no diff worth reading, nothing that looks like a choice
was made. The entry is what makes it one.

A decision *not* to take a dependency gets an entry too, and that is the half which
disappears. Forty lines written by hand in place of a crate leave no manifest line, no
lockfile churn, nothing a later reader can trip over. The refusal exists only in the head
of whoever made it, so the next session proposes the same crate, nobody can tell it was
already priced and declined, and in it goes. A refused dependency and one nobody ever
considered look identical from the outside; only a deliberate entry tells them apart.

This is sharper with an assistant than without one. Reaching for a crate is what the
training data does: asked to parse a date, hash a string, or retry a request, an
assistant will propose a dependency before it proposes twenty lines, because published
code overwhelmingly takes the dependency. The refusal is therefore the option that has to
be written down on purpose, precisely because it is the option nothing else records.

Feature flags are part of the price. `default-features = false` belongs in the entry
together with its reason: what the default set drags in, and why that tree is not wanted.
A crate taken with its defaults off is a different dependency from the same crate taken
whole, and an entry that does not say which one was taken has not priced anything.

Dev-only is part of the price, and it lowers it. An entry that can say *nothing shipped
depends on this* is materially different from one that cannot: the blast radius is a
build machine rather than every user, and the bar the alternatives had to clear was lower
because of it. Say which it is.

The manifest comment points at the entry; it does not restate it. A one-line purpose
beside the dependency is where a reader actually meets it and is worth having -- what
this is for, and where the decision lives. Standalone reasoning in a manifest comment is
a second home for the rationale, and two homes drift: the comment is trimmed, the log is
appended to, and neither reader can tell which one is current.

Record the entry when the dependency is added, while the alternatives are still in mind.
Reconstructed later, the rejected list is always the flattering one -- the same
observation `[R:decisions-log-records-rejected-alternatives]` already makes, and the
reason this rule is that one's sibling rather than its replacement. That rule requires a
decision already recognised as a decision to carry its why and its refusals. This one
says a dependency is such a decision despite arriving as a manifest line, and that
choosing to take none is another.

The asymmetry is real and must not be papered over. A gate can parse every manifest and
fail when a dependency is not named in the log, so the half about dependencies **taken**
is mechanisable and cheap. Nothing mechanical can check the half about dependencies
**refused**: there is no artefact to compare the log against, which is exactly why that
half is the one that goes missing. There the human reader is the only detector, and a
gate covering the first half must never be read as covering the second.
