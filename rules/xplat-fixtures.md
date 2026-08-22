+++
tag = "R:xplat-fixtures"
title = "A test fixture must work on every OS the repository runs on"
error_class = "Writing a test fixture or verifier that passes only on the authoring OS -- a hardcoded target path, a shell-script stub, a raw-vs-canonical path assertion, or an unstripped CR in another tool's output -- so a green run certifies one platform while claiming to certify the repository"
home = { kind = "domain", name = "rust" }
created = "2026-07-20"
status = { kind = "active" }
incident = "Ferridis (2026-07-20): three tests green on Linux and broken on Windows -- a hardcoded `target/release/mcp_consumer` path that spawned a stale Linux ELF (os error 193), a bash stub for `claude-cli` on a machine with no bash, and an `AllowedCwd` assertion comparing a raw path against a canonicalized one, which on Windows returns the extended-length `\\\\?\\C:\\...` form. Recurred at the tooling layer 2026-08-16: `verify-wiring.sh` reported the same hook wiring as both 'declared but NOT live' and 'live-only' -- a self-contradicting permanent FAIL -- because Windows jq terminates lines with CRLF and command substitution strips the trailing newline but leaves the final line's CR."
+++

A test that passes only on the machine that wrote it is a latent lie, and it is a
particularly expensive one: it does not fail, it certifies. Where a repository is worked
on from more than one operating system, every fixture that spawns a process, locates a
binary, compares filesystem paths, or parses another tool's output must be written for
both, because the one that is not will report success on the authoring OS indefinitely.

Locate a sibling binary from the running test, never from a constructed path.
`CARGO_BIN_EXE_*` exists only for the bins of the crate under test; for anything else,
start at `current_exe()`, pop `deps`, pop the profile directory, and join the name with
`std::env::consts::EXE_SUFFIX`. A literal `target/<profile>/<name>` ignores both
`CARGO_TARGET_DIR` and the platform's executable suffix, and the failure it produces is
an exec error rather than a missing-file error, which reads as a broken binary rather
than a broken path.

A fixture that must run as a child process is a small program in the language of the
repository, compiled once per test run into `CARGO_TARGET_TMPDIR` behind a `OnceLock`.
The toolchain is guaranteed present wherever the tests run; an interpreter is not. When
generating such a program's source, write the payload through a byte-level write rather
than a formatting macro, or braces in the payload are parsed as format placeholders.

Canonicalize both sides before any path comparison. On Windows `canonicalize` returns the
extended-length form, so a prefix or equality assertion against a raw path fails for a
path that is in fact correct.

Strip carriage returns from another tool's output before comparing it. Many ports
terminate lines with CRLF; capturing a command's output removes the trailing newline but
leaves the final line's CR, so exactly one record per stream carries a stray byte and
never matches its twin. The result looks like real drift, is invisible on the other OS,
and an always-red check is a muted check.

Treat a green run as evidence for the operating system it ran on and no other. A fixture
recorded as an enforcing artefact on the strength of a single-platform run is a claim
about a guarantee that was never tested where it was most likely to break.

This is the in-flight half of a pair. Its sibling governs bytes at rest -- what a checkout
puts on disk, fixed once and structurally in `.gitattributes`. This rule governs bytes in
flight, what a tool emits into a pipe at runtime, which no file attribute can reach, so
the fix belongs at the consuming end. A repository can satisfy either and fail the other.
