+++
tag = "R:exceptions-name-what-failed"
title = "Catch what you can answer; never catch Exception"
error_class = "Catching a supertype -- `Exception`, `Throwable`, `RuntimeException` -- so failures the handler was never written for are absorbed by it, or wrapping every cause in one opaque unchecked type, so a caller that could have retried one condition and surfaced another can distinguish neither"
home = { kind = "domain", name = "java" }
created = "2026-09-19"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-19 with the rest of the Java set, as the Java half of a discipline the corpus already states for Rust. `[R:errors-name-what-failed]` and `[R:no-anyhow-in-libraries]` are homed `domain-rust` and reach no Java reader; the vocabulary differs enough -- checked exceptions, a catch hierarchy, suppressed causes -- that a port rather than a shared rule is the honest form."
+++

A `catch` clause is a claim that you can do something about what you caught. Catch the
type you can answer and let the rest go up.

`catch (Exception e)` claims you can answer everything, which is never true: it absorbs
the `IllegalStateException` from a bug three frames down and the
`InterruptedException` that was asking the thread to stop, and it treats both the same as
the `IOException` the author was thinking about. The narrower the caught type, the more
the code says. Multi-catch (`catch (IOException | TimeoutException e)`) is how you handle
two without widening to their common supertype.

Three refusals that carry most of the value:

* **Never swallow.** An empty catch block, or one that logs and continues, converts a
  failure into a wrong answer computed quickly. If there is genuinely nothing to do,
  rethrow; if the method cannot throw, that is the design problem to fix rather than to
  hide.
* **Never lose the cause.** `throw new ServiceException("failed")` discards the stack that
  says why. Wrapping is fine and often right — `throw new ServiceException("loading " + id, e)`
  — as long as the cause travels. A chain printed once at the edge is worth more than
  every layer restating what the layer beneath it already said.
* **Never catch `InterruptedException` without restoring the flag.** Either propagate it or
  `Thread.currentThread().interrupt()`; absorbing it silently is how a shutdown request
  disappears and a thread pool refuses to stop.

On checked exceptions, the choice is about the caller, not about taste: a condition the
caller can plausibly recover from is checked, and a programming error — a broken
precondition, an impossible state — is unchecked. The failure mode to avoid is declaring
`throws Exception`, which is checked in form and unanswerable in practice, and the one to
avoid next is making everything unchecked so the signature stops mentioning failure at
all. That is the Rust library's ban on erasing a typed error contract, arriving in a
language where the erasure is free and needs no crate. (Named in prose rather than by
tag: that rule has graduated to a hook, and citing a retired rule is a `lint` finding —
the same precedent `[R:doc-currency]` set.)

Exception types are types. One per condition a caller could treat differently, each
carrying the values that identify the instance — the id, the path, the elapsed time — so
one log line is enough to act on rather than the start of an investigation. A message
built by string concatenation with no fields is prose wearing a class name.

Failure-mode check, at every `catch`: **what will this clause do with a failure I have not
thought of?** If the answer is "the same thing", the caught type is too wide.
