+++
tag = "R:a-view-is-not-a-copy"
title = "A collection view is a window onto someone else's data"
error_class = "Treating a view as an independent collection -- `Arrays.asList`, `subList`, `keySet`, `Map.values`, `Collections.unmodifiable*` -- so a write through one side appears through the other, a structural change to the backing collection invalidates the view, and an `add` on a fixed-size wrapper throws at a call site that reads like ordinary list code"
home = { kind = "domain", name = "java" }
created = "2026-09-19"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-19 with the rest of the Java set. It is in the set because the type system actively hides the distinction: a view and a copy are both `List<T>`, the difference is in a factory method's documentation, and the failure arrives as an exception from a line that looks like every other line."
+++

Know whether you are holding data or a window onto data. The static type is `List<T>`
either way and will never tell you.

The library returns views far more often than most code assumes, and each has its own way
of surprising:

* **`Arrays.asList(a)`** is a fixed-size wrapper *over the array*. `set` writes through to
  the array; `add` and `remove` throw `UnsupportedOperationException`. `List.of(...)` is
  immutable and independent, and is usually what was meant.
* **`list.subList(from, to)`** is a live window. Writes go through, and any structural
  change to the parent makes the sublist throw `ConcurrentModificationException` on its
  next use — including the common idiom of taking a sublist and then clearing the parent.
* **`map.keySet()`, `values()`, `entrySet()`** are views. `keySet().remove(k)` removes the
  entry from the map. That is occasionally what you want and always worth being deliberate
  about.
* **`Collections.unmodifiableList(l)`** wraps without copying: the wrapper refuses writes
  and the original still works, so it defends against the caller and not against you. It
  is not a defensive copy, which is why `[R:no-reference-to-internals-escapes]` reaches for
  `List.copyOf` instead.
* **`Stream.toList()`** is unmodifiable; **`collect(Collectors.toList())`** gives no such
  guarantee and may or may not be. Do not rely on either being mutable.

So state which you want at the point you create it. `List.copyOf` when the receiver must be
independent, the view when sharing is the intent and the lifetime is short. A view stored
in a field is the shape to look at hardest: it outlives the expression that made it, and
the backing collection can change under it at any point afterwards.

The same distinction governs iteration. Modifying a collection while iterating it throws
`ConcurrentModificationException` even single-threaded, because the iterator is a view too
— use `removeIf`, or iterate a copy, or collect the removals and apply them after.

Failure-mode check, for any collection received or returned: **if somebody writes to this,
what else changes?** If you cannot answer without opening the factory method's Javadoc,
copy it.
