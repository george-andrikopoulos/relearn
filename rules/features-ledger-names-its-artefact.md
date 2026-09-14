+++
tag = "R:features-ledger-names-its-artefact"
title = "Every feature-ledger entry names the artefact that enforces it, or declares itself exposed"
error_class = "A feature recorded in the behaviour contract with no enforcing type, property or test named, so a documented wish is indistinguishable from a documented guarantee and the ledger certifies coverage it does not have"
home = { kind = "global" }
created = "2026-08-25"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: ported 2026-08-25 from ~/.claude/skills/project-discipline/SKILL.md, the artefact that defines the practice, and specified in *Aiming the Stochastic Machine* (Zenodo, doi:10.5281/zenodo.22083202), whose section 9 names the format's sharpest limit -- the column records enforcement, not invocation. The anchoring failure is real and dated: a security IP allowlist shipped in ha-mcp v0.7.0 (2026-07-16) passing every smoke gate through a dev override while the production wiring produced non-JSON, so the headline security feature would have shipped silently disabled, failing open. Surfaced for porting on 2026-08-25, when the emitted Copilot pack was found to carry no rule about the repository discipline at all."
published_incident = "Codified from a standing repository discipline rather than mined from a failure. It surfaced in an audit of an emitted instruction pack, which carried no rule about that discipline at all while the type-design half was fully present — so one of the two disciplines the author works by reached every assistant and the other reached none. The practice: every entry in the behaviour ledger names the artefact that enforces it, or declares itself unenforced in as many words. The format's sharpest limit is that the column records enforcement and never invocation — an artefact can be named, be real, and never run — so the invocation belongs beside it."
+++

Every entry in the behaviour contract carries the artefact that enforces it: the type that makes violation unrepresentable, the property test that covers the space, or the named unit test that pins the case. No entry without one.

Where a feature genuinely cannot be enforced yet, the entry says so in those words -- **exposed, nothing yet** -- and the gap goes to the open-work list until it closes. That is not a failure of the ledger; it is the ledger working. What must never happen is a row that reads like a guarantee because it is written in the same voice as the rows beside it.

Prefer the artefact highest in the hierarchy that can hold the guarantee: a type beats a property beats a unit test. And the artefact must exercise the **wired** behaviour, not the component alone -- R:wired-artifact is the sharp edge here, because a type's own unit tests prove the type works and say nothing about whether anything produces or consumes it.

Record the **invocation**, not only the artefact. A test that exists and is never run is enforcement on paper: name the command, and require that the command named is the one the pipeline actually takes. A gate whose green invocation needs a flag teaches every reader that the default is noisy, and that is how a control decays into decoration.

The ledger is only load-bearing if it is read before every change, which is what makes R:definition-of-done-every-change its second check rather than a courtesy.
