+++
tag = "R:close-what-you-open"
title = "A resource is closed by the construct that opened it, on every path"
error_class = "Releasing a resource on the happy path only -- a close after the work rather than in a finally, a try/finally that loses the original exception to a failure in close, an AutoCloseable handed to a method that may throw before it is registered -- so a leak appears under exactly the conditions that produced the error, and the exception that explains it is replaced by one from the cleanup"
home = { kind = "domain", name = "java" }
created = "2026-09-19"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-19 with the rest of the Java set. The reason it is a rule and not a style note is the second half of the error class: a hand-written finally block does not merely risk a leak, it can *replace the diagnosis*, so the incident that leaked is also the incident nobody can read."
+++

Open a resource in a `try`-with-resources header and nowhere else. The construct closes on
every exit, in reverse order, and — the part a hand-written `finally` cannot do — it keeps
the original exception when the close also fails.

```java
try (var conn = pool.take(); var stmt = conn.prepare(sql)) {
    return stmt.execute();
}
```

The failure that a `finally` block introduces is worth stating plainly because it looks
correct. If the body throws and `close()` also throws, the `finally`'s exception replaces
the body's: the program reports a failure to close a connection and says nothing about the
query that failed first. Try-with-resources instead **suppresses** the close failure and
attaches it to the original, so both survive and the cause is the one you wanted. A leak
costs you a handle; a lost cause costs you the investigation.

Three cases where the construct does not apply and the discipline still does:

* **A resource whose lifetime is a field**, not a block — a pool, a client, an executor.
  The owning object becomes `AutoCloseable` and the discipline moves up a level; the thing
  to refuse is a field that is opened and never closed by anyone in particular.
* **An `ExecutorService`** is not closed by `shutdown()` alone. `shutdown()` then
  `awaitTermination()` then `shutdownNow()` is the sequence, and skipping it leaves
  non-daemon threads holding the JVM up.
* **A `Stream` over a file** (`Files.lines`, `Files.walk`) holds a handle and must be
  closed. It is the one stream that leaks, and it looks exactly like the ones that do not.

Do not reach for a finalizer or `Cleaner` as the primary mechanism. They run at an
unspecified time or never, and they exist to catch the case where the discipline already
failed.

Failure-mode check, for every resource: **which construct closes this if the next line
throws?** If the answer is a `close()` call further down the method, nothing does.
