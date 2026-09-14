---
name: domain-low-latency
description: "Engineering discipline for working in low-latency. Covers: Answer a requirement at the layer it lives at; Attack the design in a separate pass, and build nothing in it"
---

# domain: low-latency rules

## Answer a requirement at the layer it lives at [R:answer-the-requirement-at-its-layer]

State which layer a requirement lives at before choosing anything, and satisfy it there.

A requirement about the machine -- do not perturb a neighbouring isolated core, do not
fault a page on the send path, do not take a lock a real-time thread contends for -- is
not answered by picking a faster crate, a better-reputed transport, or an asynchronous
API. Those are library-level decisions. They may be correct and they may even help, but
they do not *hold* a systems-level constraint, and treating them as though they do
converts an open question into a closed one with nothing behind it.

The failure is not the wrong choice; it is the wrong ledger entry. A constraint marked
satisfied stops being examined. An open constraint is still visible to the next person,
which makes leaving it open strictly better than closing it at a layer that cannot keep
it.

So name the layer explicitly, then name what holds the requirement there: a CPU affinity
mask, an isolated core list, a memory policy, a pre-faulted and locked mapping, a thread
priority, a cgroup. If the answer to "what holds this?" is the name of a library, the
requirement is unheld.

Two rules meet here and neither subsumes this one. `[R:measure-cost-per-task]` governs
*how to choose* a mechanism -- by what it actually costs and touches, never by
reputation -- and its failure-mode check asks under what configuration the mechanism
causes the exact harm it was chosen to prevent. That check is necessary and it is not
sufficient: a mechanism can pass it and still be the wrong layer to have asked at.
`[R:guarantee-needs-a-reader]` is the general form of the ledger failure -- a claim with
nothing reading the state it asserts -- and this is the shape that claim takes when the
state in question is a property of the machine rather than of the code.

Then run the pass the sibling rule requires: `[R:attack-the-design-in-a-second-pass]`
exists because the person who chose the mechanism is the last person able to see that
they answered at the wrong layer.

## Attack the design in a separate pass, and build nothing in it [R:attack-the-design-in-a-second-pass]

> Has recurred 1 time(s) since it was written; most recently 2026-09-14.

When a component is finished, do a second pass whose only job is to attack it. Build
nothing in that pass.

The pass that produced a design cannot review it. It carries every assumption that went
into the design, including the ones nobody stated, and it will read the output as
confirmation because that is what it was optimising for. A review folded into the build
is not a weak review; it is the build, wearing a different label.

So separate it in time and in intent, and give it something to do:

- **Justify the choice against the simpler alternative that was not taken.** Not "is this
  good" but "why not the obvious thing" -- and if the answer is that the obvious thing was
  never considered, that is the finding.
- **Walk the failure modes out loud.** Under what configuration does this do the thing it
  was chosen to prevent? What does it do under saturation, on a cold cache, on the first
  message after an idle period, when the peer is slow rather than absent?
- **Prove the stated constraint, do not restate it.** "It does not perturb the isolated
  cores" is a claim; the affinity mask, the thread inventory and the measurement are the
  proof. `[R:answer-the-requirement-at-its-layer]` is what this pass is usually checking.
- **Ask what the evidence would look like if the opposite were true.** An artefact that
  exists, a command that exited zero and a test that passed are all compatible with the
  work never having happened -- which is `[R:verdict-survives-the-channel]`, and it is the
  single most common thing this pass catches.

**Mandatory for anything latency-sensitive.** In that domain the failure does not announce
itself: a design that perturbs a neighbouring core, allocates on a send path, or faults a
page under load still passes every functional test, and the cost appears as unexplained
tail latency somewhere else entirely -- attributable to nothing, days later, by somebody
who was not there.

<!-- relearn:generated v0.1.0 sha256=1eaa94a4d13226a01e731fccd4622c51b98a88a8c78d779882ea1f1a97445881 rules=R:answer-the-requirement-at-its-layer,R:attack-the-design-in-a-second-pass -- DO NOT EDIT; regenerate with `relearn build` -->
