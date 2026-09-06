+++
tag = "R:repair-the-lying-artefact"
title = "Repair the artefact that made the false claim, not only the doc about it"
error_class = "Closing an incident by writing or correcting prose while the executable artefact that actually misled — a script's printed path, a status line, a generated header, a success message — goes on emitting the same false claim, so the defect stays fully operational behind a note that makes it look handled"
home = { kind = "global" }
created = "2026-08-24"
origin = "mined"
status = { kind = "active" }
incident = "Design-Architecture-Tool, twice from one cause. 2026-08-16: a release build was smoke-tested against ./target/release/dat-server.exe and reported an old version, a 404 on / and a 404 on the wasm bundle -- read as three regressions in the fresh build, all in fact properties of an April binary the build had not touched (this machine sets CARGO_TARGET_DIR globally, so cargo writes elsewhere). The fix was a CLAUDE.md paragraph naming the incident and printing the resolve-it-properly command. 2026-08-24: the same thing, with a five-month-old pre-rename designer binary -- missing tag fixes and tofu buttons read as 'my fixes were not applied'. Nobody had been misled by reading CLAUDE.md: they were misled by build.sh, which ended every run printing `Binary: $ROOT/target/release/dat-designer`, a path it hardcoded and never resolved, and which the 2026-08-16 fix left untouched."
+++

When an incident traces to a false claim, find the artefact the person was actually
reading at the moment they were misled, and repair **that** first. Usually it is not the
document — it is something that printed during the work: a script's final "Binary:" line,
a deploy script's success message, a generated file's header, a status endpoint, a test
name. A document is consulted; an artefact is *emitted at you* while your attention is on
the task. That asymmetry decides who gets believed.

A doc and a script disagreeing is not a tie. People run the script. Correcting the doc and
stopping there leaves the defect fully operational and adds a paragraph that makes it look
handled — the worst of both, because the next occurrence now has a written warning standing
over it as evidence that someone already dealt with this.

Apply the test before closing: **could the same person be misled again in exactly the same
way without ever opening the document I just fixed?** If yes, the fix has not landed. Ask
also what else in the repo asserts this same fact — a README snippet, a CI summary, a
printed usage line — because a claim usually has more than one mouth.

Then push it down a layer rather than restating it: make the artefact *derive* what it
reports instead of asserting it (resolve the path, stat the file, read the version it
actually built), and add the mechanical check that fails any future artefact making the
unresolved claim. Deriving beats asserting for the same reason
`[R:prefer-by-construction]` prefers designs to guards — a derived claim cannot drift from
what it describes, so it cannot go stale the way `[R:doc-currency]` describes.

This is the reporting half of `[R:verify-through-production-path]`. That rule says to
exercise the real channel; this one says the real channel must also tell the truth about
what it produced — verifying through a production path that reports a path it never
resolved proves nothing.
