+++
tag = "R:anchor-outside-the-checked-partys-reach"
title = "Keep a check's trust anchor out of reach of the party it checks"
error_class = "A check compares the checked party's input with an anchor (a key, an allow-list, a permission, a chain head) that the same party can write by some path, so the check verifies the adversary against the adversary's own data and a guarantee of unforgeability or confinement is claimed that does not hold"
home = { kind = "global" }
created = "2026-10-17"
origin = "mined"
status = { kind = "active" }
incident = "trade-platform, 2026-10-17. The decision core required a HumanSignature to accept a proposal: an RSA signature over the proposal digest, verified against \"the enrolled key\". The enrolled key was a row in the same SQLite ledger that the MCP process and any same-user process (a Claude Code shell among them) could write, enrol_key was callable by any holder of a Desk<Human>, and replay trusted any key ever enrolled. So anything that could write the file could enrol its own key, sign with it, and record an acceptance that replayed and chain-verified cleanly. The signature check itself was correct; what it checked against was in the adversary's hands. A doc comment claimed \"a process that can call accept still cannot forge one\", and two tests used the hole as their setup without anyone noticing. An independent review caught it as CRITICAL before any push. Fix: the anchor is read from the device at start-up (the Windows Hello public key; the private half never leaves the TPM), never from the writable store, and a test reproduces the forgery and shows the real desk refuse to load it. The audit log's own head-anchor gap (tail truncation undetected because the chain's anchor lives in the same file) is the same class, recorded as an open TODO the same day."
+++

Every check that protects something against a party compares that party's input with an
anchor: a public key, an allow-list, a permission, a policy, a hash-chain head. The check
is only as strong as the anchor's position. If the party being checked can write the
anchor, the check verifies the adversary against the adversary's own data, and it passes
for them exactly as it passes for you.

So before trusting any such check, ask: *who can write the thing it compares against, and
is the checked party among them?* Trace write access by every path, not by the one the API
offers. A table the surface has no endpoint for is still writable by any process that can
open the file; a config file is writable by anything running as the same user; a value a
plugin stores in its own namespace is the plugin's to change. If the checked party can
reach the anchor by any path, the guarantee is not held, however correct the cryptography
or the comparison.

Put the anchor where the checked party cannot reach it: in hardware (a TPM-held key
read from the device), in a separate trust domain, in a signed or out-of-band source, or
compiled into the binary the checked party does not control. Where no such place exists
yet, say in the threat model which parties the check does not stop, rather than letting
the check imply it stops them.

The tell is a guarantee stated in the form "X cannot do Y because Z is verified": check
where Z's reference value lives before writing the sentence. A claim of unforgeability is a
claim about write access to the anchor, and it is false whenever that write access was not
traced.

This is the security-design sibling of `[R:wired-artifact]`: that rule catches a check
accepting evidence anything can produce; this one catches a check whose reference value
anything the check is aimed at can produce. Same family as `[R:guarantee-needs-a-reader]`
and `[R:check-the-claim-you-inherit]`: a stated guarantee with no traced basis.

Failure-mode check: **can the party this check constrains write, by any path, the value
the check compares against?** If yes, or if you have not traced it, the check is not yet a
control.
