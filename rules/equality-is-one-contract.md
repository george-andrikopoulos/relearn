+++
tag = "R:equality-is-one-contract"
title = "equals and hashCode are one decision, and a mutable key breaks both"
error_class = "Overriding `equals` without `hashCode`, or deriving either from a field that can change after the object is used as a key -- so the object is silently lost inside every hash-based collection: `contains` returns false for an object the set holds, and the entry can never be removed"
home = { kind = "domain", name = "java" }
created = "2026-09-19"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-19 with the rest of the Java set. It earns a rule rather than a style note because the failure is silent in the specific way this corpus keeps naming: nothing throws, nothing logs, and the collection reports an answer that is wrong rather than an error that is right."
+++

Decide equality once, implement both halves in the same change, and derive them from
fields that cannot change while the object is reachable from a collection.

The contract is mechanical: equal objects must have equal hash codes. Break it and a
`HashMap` looks in the wrong bucket, so `get` returns null for a key it contains and
`remove` cannot find an entry to remove -- a leak whose size grows with traffic. Nothing
detects this. The collection is behaving exactly as specified; the specification was
handed an object that lied.

The mutable-key half is the one that survives review, because the class is correct when
it is written and becomes wrong when someone adds a setter. An object whose `hashCode`
depends on a field that is later mutated has moved bucket without the map being told, and
is now unreachable through the very key it is filed under. **So a type used as a key is
immutable, or it is not used as a key.** A `record` gives you both halves generated from
the components and final fields by construction, which is why it is the default shape for
a value type and why hand-writing these two methods should feel like a decision rather
than boilerplate.

Two more that follow from the same contract and are worth stating because each has its
own way of going wrong:

* **`compareTo` must agree with `equals`** wherever the type reaches a `TreeMap` or a
  sorted set, which use ordering and never call `equals` at all. A comparator saying two
  objects are equivalent while `equals` says they differ produces a set that silently
  holds one of them.
* **Inheritance and equality do not compose.** An `equals` written with `instanceof`
  accepts a subclass and is then asymmetric; one written with `getClass()` is symmetric
  and rejects every subclass. There is no third option, which is one of the reasons
  `[R:design-for-inheritance-or-forbid-it]` prefers `final`.

Failure-mode check, before a type is used as a key or put in a set: **which fields does
its equality read, and can any of them change while it is in there?**

This is `[R:make-illegal-states-unrepresentable]` where the language will not help: Java
cannot stop you writing an inconsistent contract, so the remedy is to generate it
(`record`) rather than to write it carefully.
