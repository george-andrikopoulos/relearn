+++
tag = "R:serializable-is-a-second-constructor"
title = "Implementing Serializable adds a constructor that checks nothing"
error_class = "Adding `implements Serializable` to a domain type, so deserialization builds an instance without running any constructor -- every invariant the class advertises is bypassed, every private field becomes writable by whoever controls the byte stream, and the field layout becomes a public API that cannot be changed"
home = { kind = "domain", name = "java" }
created = "2026-09-19"
origin = "codified"
status = { kind = "active" }
incident = "Codification-dated, not single-incident: written 2026-09-19 with the rest of the Java set. It is the sharpest Java instance of a shape this corpus already names in two places -- a second construction path that skips the checks the first one enforces -- and it is the one that is also a remote code execution class, which is why it is stated as a refusal rather than a caution."
+++

`implements Serializable` is not a marker. It is a second, invisible, public constructor
that takes a byte array and performs no validation.

Deserialization does not call your constructor. It allocates the object and writes the
fields directly from the stream, so every check the constructor makes — the non-null, the
range, the "these two fields must agree" — is skipped, and an attacker or a corrupted file
produces an instance the class's own author believes cannot exist. That is
`[R:private-fields-only]` and `[R:parse-dont-validate]` defeated by a language feature
rather than by anyone's code: the perimeter was built, and this walks around it.

Two consequences beyond the invariant:

* **The field layout becomes a published API.** Once instances are serialized anywhere
  durable, renaming or removing a private field is a compatibility break. A `serialVersionUID`
  controls only whether the break is detected, not whether it happened.
* **Deserializing untrusted data is remote code execution**, not a theoretical risk. The
  stream chooses which classes to instantiate, and a gadget chain assembled from whatever
  is on the classpath does the rest. The JDK's own filtering
  (`ObjectInputFilter`) exists because the mechanism cannot be made safe by being careful
  with it.

So the default is: **do not implement it.** Where an object must cross a process boundary
or reach a disk, use an explicit format with an explicit parser — JSON, a schema, a binary
codec you wrote — so the reconstruction runs through a constructor and the perimeter
holds. The serialization format is then a decision with a version, rather than a shadow of
the class's private fields.

Where it cannot be avoided, the obligations are real and none of them is optional: a
`readObject` that validates exactly what the constructor validates, `readResolve` for a
type that must be a singleton — an enum is `Serializable` correctly and for free, which is
one more reason to prefer one — and a declared `serialVersionUID` so the break is at least
visible. Treat every one of those as evidence that the type should not have been
serializable.

Failure-mode check, before adding the interface: **what does this class's constructor check,
and am I content for a byte stream to skip it?**
