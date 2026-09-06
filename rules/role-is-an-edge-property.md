+++
tag = "R:role-is-an-edge-property"
title = "A role belongs to an edge endpoint, never to the thing at the end of it"
error_class = "Deriving a relationship role from an attribute of one participant -- producer or consumer from how a service waits -- so every renderer asserts a mechanism the design does not contain. It is sound only for a pure source or a pure sink, and it survives a correction in whatever document was not part of the fix"
home = { kind = "project", path = "design-architecture-tool" }
created = "2026-08-28"
origin = "mined"
status = { kind = "graduated", to = "test:the_repository_derives_no_queue_role_from_a_spin_mode (documents) + test:spin_mode_does_not_change_a_single_queue_end_letter (renderers)", date = "2026-09-06" }
incident = "M23-E1, 2026-08-28. The UI and `SpinMode`'s own doc comments derived a queue role from the spin mode -- busy meant producer, lazy meant consumer. George: 'this doesnt mean that Busy spinning cant read from a Queue, it can be a consumer also.' It is the classic low-latency read path: a busy-spinning service spins on its INBOUND queue to avoid the scheduler wake-up, so the derivation was backwards in the commonest case. A mid-chain service is both roles at once and a fan-out is a producer many times over, so one role per service can never be right."

[[recurrence]]
date = "2026-08-28"
incident = "Fired again the same day, in the feature that was supposed to state the role correctly: the P and C letters shipped on EVERY edge, HTTP included. George caught it immediately -- 'I have seen http and P -> C, thats not right'. Producer and consumer are a QUEUE relationship; a call has no queue and therefore neither role, so the diagram asserted a mechanism that was not there. Fixed by making absence a type: `queue_ends()` returns `Option<(QueueEnd, QueueEnd)>` and is None for a call, rather than a flag every renderer has to remember to check."

[[recurrence]]
date = "2026-08-30"
incident = "Fired a third time, in the document a reader consults for the concepts themselves. CLAUDE.md's domain-knowledge section still read 'Queue Producers = busy-spinning services' and 'Queue Consumers = lazy-spinning services' two days after the derivation had been removed from the UI (M23-E1) and from QueueEnd's doc comments (M24-E6). It survived two corrections of the same error and then misled a session into asserting it back to George as though it were the contract. Third home of a correction made in two."
+++

Read a role from the DIRECTION of an edge, never from a property of a service.

A service that writes into a queue is that queue's producer; one that reads from it is a
consumer. Any service is routinely both at once -- a mid-chain hop consumes from its
inbound queue and produces into its outbound one -- so a role assigned to a whole service
is only ever correct for a pure source or a pure sink, and the diagram is wrong
everywhere else.

Spin mode says how a service WAITS, not what it does with a queue. What it legitimately
says is how often an endpoint pays a socket crossing: a cost, not an identity. That is
`SpinCost` and `HandoffSeverity` in `shared/src/model/handoff.rs`, and neither type can
express a role, so neither can drift back into asserting one. Making the wrong statement
unrepresentable is what ended this, after prose had failed three times.

The recurrences are the lesson, not the original error. A derivation removed from the
code lives on in the doc comments, and removed from the doc comments lives on in the
domain-knowledge section -- each fix landing where the last reader complained rather than
everywhere the claim is made. When a correction is to a *concept*, grep the whole
repository for the claim before calling it fixed, and pin it with a test that reads the
concept rather than the surface: `spin_mode_does_not_change_a_single_queue_end_letter`
would have failed on day one.
