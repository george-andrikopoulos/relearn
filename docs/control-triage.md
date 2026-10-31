# Control triage — snapshot of 2026-10-30

*A dated snapshot. The numbers below describe the corpus on this date and nothing else in
the repository quotes them (`[R:doc-currency]`). For the corpus as it is now, run
`relearn list --format tsv`.*

## What this is

Every rule whose status was `active` on 2026-10-30 is held by prose alone, the weakest
rung. This pass sorts each one into the control that could hold it instead:

| Bin | Means |
|---|---|
| **MECHANICAL** | A static signature a detector can hold, in whole or in part. The detail below names the signature, the engine, the files and the false-positive risk, and for a partial one, the half it leaves to prose. That half is the `uncovered` text if the rule later goes `partial`. |
| **TYPE** | Held best by construction in the consuming project. The rule's job is to point at the type (`[R:prefer-by-construction]`). The detail names the type shape. |
| **EVAL** | No static signature, but the failure shows up when a task is replayed. The detail sketches the probe: the task, and what the checker looks for. |
| **JUDGMENT** | Only prose can hold it. The detail says why no observable signal exists. |

**No status was changed.** A drafted detector makes its rule a `graduation candidate`
(an Info finding in `relearn lint`); whether the detector covers the class is George's
review. The drafts are emitted into `hooks-pack/` because the `hooks` target emits every
detector, and the pack README marks each one `draft` so that following the install steps
never copies one.

## Method

- **Population.** `relearn list --format tsv`, the rows whose status column reads
  `active`. Not assembled by hand.
- **Priority.** Recorded recurrences first, then home: `global`, then the domain homes,
  then the project homes. Inside the domain tier the prompt gave no order, so a stated
  tie-break was used rather than an alphabetical accident (`[R:order-by-explicit-rank]`):
  a signature precise enough to **block** comes before one that can only **warn**, and the
  domain name breaks the remaining ties, for determinism only.
- **Engines.** relearn's detector schema has two engines, `line-regex` (one pattern per
  line of the text being written) and `sibling-case`, and one event, a Claude Code
  `PreToolUse` on Write, Edit and MultiEdit. A MECHANICAL rule whose signature needs
  anything else — a type-aware linter, a whole-file or whole-repository check, a Bash
  command, a commit message — is binned MECHANICAL with that engine named, and is **not
  drafted**, because the schema cannot carry it. That is why some high-priority
  MECHANICAL rules have no draft while lower-priority ones do.
- **Review.** Every verdict was read against the rule's title, error class and body. No
  subagents were used. Private `incident` fields were not copied; every fixture is
  synthetic.

## Totals, 2026-10-30

| Bin | Rules | Detector drafted |
|---|---|---|
| MECHANICAL | 33 | 10 |
| TYPE | 13 | — |
| EVAL | 34 | — |
| JUDGMENT | 2 | — |
| **All `active` rules** | **82** | **10** |

No active rule had a MECHANICAL signature that this pass judged realistic and failed to
discriminate, so no candidate moved to EVAL for that reason. Three drafts were narrowed
after `relearn check` refused them; each is recorded under *The ten drafted detectors*.

## Snapshot

Ordered by the priority above: the rules with a recorded recurrence first, then each home
tier. Inside a tier the MECHANICAL rules come first, drafted before not drafted and
block-grade before warn-grade, then EVAL, TYPE and JUDGMENT, which have no detector to
rank. **Drafted** is whether this pass committed a `[[detector]]` for the rule.

| Tag | Home | Bin | Reason | Drafted |
|---|---|---|---|---|
| `R:check-the-claim-you-inherit` | global | EVAL | Repeating a stale claim shows in the trace as a report with no query of the live state before it | no |
| `R:doc-currency` | global | EVAL | A change whose diff touches the artefact but not the checked-in example describing it | no |
| `R:no-silent-spend` | global | EVAL | A costed fork either appears before the expensive action or it does not | no |
| `R:prefer-by-construction` | global | TYPE | The rule is the general form of construction: fold paired actions into one function | no |
| `R:repair-the-lying-artefact` | global | EVAL | The fix's diff touches the doc but not the script that printed the false claim | no |
| `R:signal-needs-a-consequence` | global | EVAL | A new check either exits non-zero on a bad input and is wired to a mandatory event, or not | no |
| `R:verify-through-production-path` | global | EVAL | The verification run either goes through the production entry point or through a stand-in | no |
| `R:attack-the-design-in-a-second-pass` | domain-low-latency | EVAL | A separate critique turn with no writes after the build, or none | no |
| `R:no-weak-model-for-judgment` | global | MECHANICAL | Embedding a local model runtime has a spelling in source and manifests | **yes** |
| `R:revision-integrity` | global | MECHANICAL | A heading that opens with a count; everything else in the rule is prose | **yes** |
| `R:one-home-per-rule` | global | MECHANICAL | A verbatim paragraph in two files; needs a cross-file engine | no |
| `R:pin-eol-for-executable-text` | global | MECHANICAL | An executable or hashed path without an `eol` attribute; needs a repository gate | no |
| `R:price-every-dependency` | global | MECHANICAL | Manifest against decisions log; already a test in this repository, not a line pattern | no |
| `R:reconcile-wiring-at-start` | global | MECHANICAL | Declared against live wiring; needs a scheduled diff, not a write hook | no |
| `R:an-ambiguous-write-target-is-a-question` | global | EVAL | A write to a target the request did not name, with no question before it | no |
| `R:delegate-a-fan-out` | global | EVAL | An enumerated list walked serially with no delegation and no fork stated | no |
| `R:diff-against-the-known-good-state` | global | EVAL | The first experiment either touches what changed since the known-good state or not | no |
| `R:evidence-must-discriminate` | global | EVAL | A "confirmed" written on an observation both hypotheses predict | no |
| `R:measure-cost-per-task` | global | EVAL | A choice justified by tier with no measured cost per completed task | no |
| `R:measure-the-claim-not-a-subset` | global | EVAL | A recorded figure overwritten by a filtered measurement | no |
| `R:no-stale-push-over-fresh` | global | EVAL | The written sync script, run against a newer target, either refuses or overwrites | no |
| `R:review-against-contract-not-plan` | global | EVAL | A plan-faithful contract violation is either reported or approved | no |
| `R:search-before-you-build` | global | EVAL | A spec written with no search of the author's own repositories before it | no |
| `R:source-practice-from-its-artefact` | global | EVAL | A description drafted before the defining artefact was opened | no |
| `R:wired-artifact` | global | EVAL | The written success check, fed a forged artefact, either fails or passes | no |
| `R:make-illegal-states-unrepresentable` | global | TYPE | The rule is the type discipline itself | no |
| `R:no-sentinel-values` | global | TYPE | An absent state is an enum variant; a sentinel has no fixed spelling | no |
| `R:anchor-outside-the-checked-partys-reach` | global | JUDGMENT | Who can write an anchor is a property of a system's access graph, not of any text or act | no |
| `R:no-tool-attribution-in-public-artefacts` | domain-disclosure | MECHANICAL | Attribution trailers and banners have exact spellings when they reach a file | **yes** |
| `R:repository-private-by-default` | domain-disclosure | MECHANICAL | A public create or a flip to public has a spelling in scripts and infrastructure | **yes** |
| `R:identity-is-not-equality-for-boxes` | domain-java | MECHANICAL | `==` against a string literal is line-local; boxed numbers need types | **yes** |
| `R:serializable-is-a-second-constructor` | domain-java | MECHANICAL | `implements Serializable` is line-local | **yes** |
| `R:errors-name-what-failed` | domain-rust | MECHANICAL | String and boxed-dyn errors, `Other(String)` and `map_err` to a string are line-local | **yes** |
| `R:no-let-underscore-on-a-local` | domain-rust | MECHANICAL | `let _ = name;` is line-local | **yes** |
| `R:parse-dont-validate` | domain-rust | MECHANICAL | A `validate` or `is_valid` function returning `bool` or `Result<()>` is line-local | **yes** |
| `R:xplat-fixtures` | domain-rust | MECHANICAL | A literal `target/<profile>` path in a test is line-local | **yes** |
| `R:employer-identity-not-in-public-artefacts` | domain-disclosure | MECHANICAL | Needs a hashed machine-local list; a committed pattern would be the disclosure | no |
| `R:names-travel-with-the-quote` | domain-disclosure | MECHANICAL | Same list, run at the destination; a committed pattern would be the disclosure | no |
| `R:equality-is-one-contract` | domain-java | MECHANICAL | `equals` without `hashCode` needs a whole-class check | no |
| `R:no-reference-to-internals-escapes` | domain-java | MECHANICAL | Returning a mutable field needs field-type analysis | no |
| `R:features-ledger-names-its-artefact` | domain-project-files | MECHANICAL | An entry with no `Enforced by:` needs a per-section check | no |
| `R:five-files-no-more` | domain-project-files | MECHANICAL | A sixth standing document at the root needs a root-anchored gate | no |
| `R:borrow-in-signatures` | domain-rust | MECHANICAL | Already two clippy lints; a line pattern would be worse | no |
| `R:module-visibility-is-deliberate` | domain-rust | MECHANICAL | Already a rustc lint, `unreachable_pub` | no |
| `R:must-use-on-consequential-returns` | domain-rust | MECHANICAL | A line pattern cannot see the attribute on the line before; clippy can | no |
| `R:close-what-you-open` | domain-java | MECHANICAL | An explicit `.close();` is a tell; warn-grade | no |
| `R:design-for-inheritance-or-forbid-it` | domain-java | MECHANICAL | A class with no `final`, `abstract` or `sealed` is line-local; warn-grade | no |
| `R:exceptions-name-what-failed` | domain-java | MECHANICAL | `catch (Exception e)` is line-local; legitimate at a thread boundary, so warn-grade | no |
| `R:null-is-not-a-value` | domain-java | MECHANICAL | `return null;` is line-local; legitimate for `Void`, so warn-grade | no |
| `R:a-queue-without-a-bound-has-no-overload-behaviour` | domain-low-latency | MECHANICAL | Unbounded channel constructors are line-local; whether the path can fall behind is not | no |
| `R:no-retry-loop-on-a-contended-path` | domain-low-latency | MECHANICAL | A CAS call is a tell; whether the path is contended is not | no |
| `R:decisions-log-records-rejected-alternatives` | domain-project-files | MECHANICAL | A dated row with an empty last cell is line-local; layouts vary, so warn-grade | no |
| `R:parse-wide-then-range-check` | domain-rust | MECHANICAL | Parsing into a narrow integer is line-local; whether out-of-range needs naming is not | no |
| `R:a-model-turn-is-for-judgment` | domain-agent-tooling | EVAL | Count model turns spent on edges that read only a field | no |
| `R:a-speculated-path-deoptimises-when-the-input-changes` | domain-java | EVAL | A latency figure reported without an input-shift run or a deoptimisation log | no |
| `R:a-view-is-not-a-copy` | domain-java | EVAL | The written code, run, either aliases or throws on a view | no |
| `R:a-wrapper-type-is-not-free-here` | domain-java | EVAL | A zero-cost claim with no allocation measurement behind it | no |
| `R:publish-safely-or-not-at-all` | domain-java | EVAL | A jcstress run of the publication observes default fields or does not | no |
| `R:a-measurement-matches-the-regime-it-reports` | domain-low-latency | EVAL | A figure reported with no warm/cold statement or harness warm-up setting | no |
| `R:allocated-is-not-resident` | domain-low-latency | EVAL | Page faults counted on the first message of a replayed hot path | no |
| `R:answer-the-requirement-at-its-layer` | domain-low-latency | EVAL | A fixed systems-level scenario; the checker looks for the mechanism that holds it | no |
| `R:no-allocation-on-the-hot-path` | domain-low-latency | EVAL | The replayed hot path under a counting allocator | no |
| `R:no-coordinated-omission` | domain-low-latency | EVAL | The written load generator, against a stalling target, records the stall or hides it | no |
| `R:no-syscall-on-a-bounded-path` | domain-low-latency | EVAL | Syscalls counted inside the bounded region of a replayed path | no |
| `R:profiler-samples-where-it-can-stop` | domain-low-latency | EVAL | A profile of a counted loop read without naming the sampler's bias | no |
| `R:verify-ordering-on-the-weakest-target` | domain-low-latency | EVAL | An ordering claim with no weak-model run (loom, aarch64) behind it | no |
| `R:claude-md-recreates-the-project` | domain-project-files | EVAL | A fresh reader given only the charter answers a fixed rebuild quiz | no |
| `R:design-types-first` | domain-rust | EVAL | The write order in the trace: types before bodies, or bodies first | no |
| `R:verify-the-abstraction-compiled-away` | domain-rust | EVAL | A zero-cost claim with no `cargo asm` or release benchmark before it | no |
| `R:seal-the-alternatives` | domain-java | TYPE | A sealed interface matched exhaustively | no |
| `R:a-structure-keeps-the-regime-it-was-proved-under` | domain-low-latency | TYPE | A consumer handle that cannot be cloned or shared | no |
| `R:no-false-sharing-on-a-hot-line` | domain-low-latency | TYPE | A cache-padded wrapper around each hot field | no |
| `R:transient-state-is-not-a-terminal-state` | domain-low-latency | TYPE | A pop result with a distinct not-yet-linked variant | no |
| `R:newtype-liberally` | domain-rust | TYPE | The rule is the newtype | no |
| `R:seal-closed-trait-sets` | domain-rust | TYPE | The rule is the sealed trait | no |
| `R:typestate-builder-for-required-fields` | domain-rust | TYPE | The rule is the typestate builder | no |
| `R:typestate-for-protocols` | domain-rust | TYPE | The rule is the typestate; its runtime-flag tell is name-dependent | no |
| `R:no-stall-inside-a-publication-window` | domain-low-latency | JUDGMENT | What can stop a writer mid-window depends on the scheduler and runtime, not the code | no |
| `R:generate-guards-unversioned` | project-relearn | TYPE | A write path that takes a guard witness | no |
| `R:order-by-explicit-rank` | project-relearn | TYPE | An explicit rank, `home_rank`, rather than a string sort | no |
| `R:no-secrets-in-config-repo` | project-stochos-lab | MECHANICAL | A staged path on a machine-local list; needs a commit-time event | no |
| `R:verify-tracked-after-move` | project-stochos-lab | MECHANICAL | A path that left the index after a move; needs a repository gate | no |

## The ten drafted detectors

Each is one commit on `feat/control-triage`, adds the rule's `[[detector]]` and its
generated script, and leaves the status `active`. **Fixture results** are what
`relearn check` (fixtures run inside the parse, plus the self-exclusion sweep over every
rule file and every emitted file) and `bash hooks-pack/hooks/test-detectors.sh` (every
fixture through the generated script) reported. Both passed for all ten.

| Rule | Detector | Action | Signature | Files | False-positive risk | Uncovered half |
|---|---|---|---|---|---|---|
| `R:no-weak-model-for-judgment` | `no-embedded-local-llm` | warn | An Ollama client or crate, llama.cpp bindings, or the Ollama port | source and manifests in six ecosystems | A tool whose purpose is local models, or a scoped mechanical pass the rule permits; override `allow:local-model` | Routing judgment to a cheaper hosted tier, and any local runtime not named in the pattern |
| `R:revision-integrity` | `no-count-in-heading` | warn | A markdown heading that opens with a number or number word and a plural noun | `**/*.md` | A heading whose count is its subject; override `allow:counted-heading` | Antecedents, cross-references, counts in body text, and terms used before they are defined |
| `R:no-tool-attribution-in-public-artefacts` | `no-tool-attribution` | block | A co-author trailer naming the assistant, a session-link trailer, a generated-with banner | every file except the pack's runner | A document quoting the form; override `allow:attribution` | A trailer passed inline to `git commit`, the tool's setting itself, document metadata, and history already pushed |
| `R:repository-private-by-default` | `no-public-repo-create` | block | `gh repo create … --public`, `gh repo edit … --visibility public`, Terraform `visibility = "public"` | scripts, Terraform, CI config | A named public exception; override `allow:public-repo` | A create typed at a shell prompt, through the web UI or through the REST API |
| `R:identity-is-not-equality-for-boxes` | `no-string-identity-compare` | block | `==` or `!=` against a string literal | `**/*.java` | Very low; a deliberate interning check | Boxed numbers, which need operand types |
| `R:serializable-is-a-second-constructor` | `no-serializable-domain-type` | block | `implements` or `extends` … `Serializable` | `**/*.java` | A sanctioned legacy format; override `allow:serializable` | A type that inherits `Serializable`, such as an exception, and whether a sanctioned `readObject` really validates |
| `R:errors-name-what-failed` | `no-string-error` | block | `Result<_, String>`, `Result<_, Box<dyn Error>>`, `Other(String)`-style catch-alls, `map_err` to a string | Rust outside `main.rs`, build scripts and tests | A prototype; override `allow:string-error` | Field-less variants, and variants that omit the identifying value |
| `R:no-let-underscore-on-a-local` | `no-let-underscore-local` | block | `let _ = name;` on a bare local | `**/*.rs` | Very low; the rule says when it is right, and that case carries a reason | An underscore-prefixed binding that silences the same warning |
| `R:parse-dont-validate` | `no-free-validator` | block | A `validate*` or `is_valid*` function returning `bool` or `Result<()>` | Rust outside tests | A method on an already-parsed type; override `allow:validator` | Re-checking already-parsed data at an interior call site |
| `R:xplat-fixtures` | `no-constructed-target-path` | block | A string literal containing `target/debug` or `target/release` | Rust tests and build scripts | Very low | Uncanonicalised path comparison, a stray CR in another tool's output, and interpreter stubs |

**Three refusals, two kinds.** Drafts were narrowed after `check` refused them. That is
the sweep doing its job, so each refusal is recorded here rather than only fixed:

- `no-count-in-heading` first matched a count anywhere in a heading. The sweep found it
  firing on emitted text: a corpus rule's heading uses "between two stores" as its
  subject, not as a tally. The pattern now requires the count at the start of the heading,
  and a clean fixture of that shape pins the narrowing. The false-positive class is
  smaller, not gone: a heading that opens with a count as its subject still hits, which is
  why the action is `warn`.
- `no-tool-attribution` (and `no-public-repo-create`, scoped to `*.sh`) fired on the
  pack's generated runner, `hooks/test-detectors.sh`, which carries every hit fixture. The
  implicit self-excludes cover `hooks-pack/**` and `hooks/relearn/**`, but the sweep reads
  the runner at its emitted path relative to the output root, where neither applies. Both
  now exclude `**/hooks/test-detectors.sh` explicitly, and `no-tool-attribution` has a
  clean fixture at that path.
- `no-public-repo-create` then fired on **its own generated script**, at
  `hooks/no-public-repo-create.sh`: the pattern `gh repo create .*--public` matches its
  own source text, because `.*` matches the literal `.*`. Same cause, so the same remedy:
  it excludes its own script by name. A blanket `**/hooks/*.sh` exclude was refused, since
  it would also exempt a real repository's `hooks/` scripts.

The second and third are one gap in relearn rather than in the drafts: the implicit
self-excludes do not cover the pack's scripts at the relative paths the sweep reads them
at. It is carried in TODO.md. Of the other drafts, none is scoped to `.sh` or `**` and
none matches its own pattern text; that was checked with `grep -E` before they landed.

## MECHANICAL, not drafted

**Needs an engine relearn does not have.** These outrank the drafts by home, and could not
be drafted:

| Rule | Engine it needs | Signature | Uncovered half |
|---|---|---|---|
| `R:one-home-per-rule` | cross-file duplicate-paragraph scan | the same paragraph in two files | an index duplicated as a list, and paraphrased copies |
| `R:pin-eol-for-executable-text` | repository gate over `git check-attr eol` | a shebang or hashed path with no `eol` | none for the paths the gate lists |
| `R:price-every-dependency` | manifest against decisions log; this repository's `tests/dependencies.rs` is one | a dependency no `Dependency:` row names | a dependency refused, which leaves no artefact |
| `R:reconcile-wiring-at-start` | scheduled declared-against-live diff; stochos-lab's `verify-wiring.sh` is one | a declared hook not live | the drill that proves the alarm fires |
| `R:employer-identity-not-in-public-artefacts` | hashed machine-local term list; `scripts/no-banned-names.sh` is one | a protected term in a tracked file or in history | role titles and names not on the list |
| `R:names-travel-with-the-quote` | the same list, run at the destination; `contribute`'s banned-terms pass is one | a protected term in a quotation | identifiers not on the list |
| `R:equality-is-one-contract` | whole-class check (Error Prone `EqualsHashCode`) | `equals` with no `hashCode` | a mutable field in either, which is TYPE (records) |
| `R:no-reference-to-internals-escapes` | field-type analysis (SpotBugs `EI_EXPOSE_REP`, `EI_EXPOSE_REP2`) | a getter returning a mutable field | a reference stored from a caller and mutated later |
| `R:features-ledger-names-its-artefact` | per-section document check | an entry with no `Enforced by:` line | an artefact named that does not enforce the entry |
| `R:five-files-no-more` | root-anchored gate | a sixth standing document at the repository root | what counts as a standing document anywhere else |
| `R:borrow-in-signatures` | clippy `needless_pass_by_value` and `ptr_arg` | an owned or `&String` parameter only read | where a function stores the value |
| `R:module-visibility-is-deliberate` | rustc `unreachable_pub` | `pub` on an item unreachable from the crate root | a reachable `pub` nobody chose |
| `R:must-use-on-consequential-returns` | clippy `must_use_candidate` | a consequential return with no attribute | the type-level attribute the rule prefers |
| `R:no-secrets-in-config-repo` | commit-time event (`git-pre-commit`, named and not built) | a staged path on a machine-local list | secrets inside an otherwise shareable file |
| `R:verify-tracked-after-move` | repository gate; stochos-lab's `verify-deploy.sh` is one | a moved path that left the index | none for the paths the gate reads |

Four of these already have a working control somewhere (`tests/dependencies.rs`,
`scripts/no-banned-names.sh`, stochos-lab's `verify-wiring.sh` and `verify-deploy.sh`), so
those rules may be candidates for `partial`. That is noted for George's review and not
acted on: this pass changes no status.

**Line-local, warn-grade, not drafted.** These have a `line-regex` signature, but a
realistic share of the hits is legitimate, so each needs `warn` and a sample of real code
before it is worth drafting:

| Rule | Signature | False positive | Uncovered half |
|---|---|---|---|
| `R:close-what-you-open` | an explicit `.close();` statement | `close()` delegating inside an `AutoCloseable` | a resource never closed at all, and one handed on before it is registered |
| `R:design-for-inheritance-or-forbid-it` | `class` with no `final`, `abstract` or `sealed` | framework-proxied classes | a constructor calling an overridable method |
| `R:exceptions-name-what-failed` | `catch` of `Exception`, `Throwable` or `RuntimeException` | a thread or request boundary | every cause wrapped in one opaque unchecked type |
| `R:null-is-not-a-value` | `return null;` | `Void` callables, interface contracts that require it | a null stored in a field |
| `R:a-queue-without-a-bound-has-no-overload-behaviour` | unbounded channel and queue constructors in Rust and Java | a path that cannot fall behind | a `Vec` or `VecDeque` used as a queue, and the overload decision itself |
| `R:no-retry-loop-on-a-contended-path` | `compare_exchange_weak`, `compareAndSet` | an uncontended or cold path | the bound on steps the rule asks to be stated |
| `R:decisions-log-records-rejected-alternatives` | a dated decisions-log row whose last cell is empty | a log with a different layout | append-only: an edit rewriting an old row |
| `R:parse-wide-then-range-check` | `parse::<u8>` and other narrow integer parses | a narrow type where out-of-range needs no name | a custom deserialiser into the narrow type |

## TYPE — the shape each rule points at

| Rule | Type shape in the consuming project |
|---|---|
| `R:prefer-by-construction` | One function that performs both halves of a pair, with the unpaired half unreachable from outside its module |
| `R:make-illegal-states-unrepresentable` | A sum type in place of a flag beside an option |
| `R:no-sentinel-values` | An enum variant for the absent, stopped or unknown state |
| `R:seal-the-alternatives` | A `sealed interface … permits`, switched over with no `default` |
| `R:a-structure-keeps-the-regime-it-was-proved-under` | A consumer handle that is neither `Clone` nor `Sync`, so a second consumer does not compile |
| `R:no-false-sharing-on-a-hot-line` | A cache-padded wrapper (`#[repr(align(64))]`, `CachePadded<T>`, `@Contended`) around each independently written field |
| `R:transient-state-is-not-a-terminal-state` | A pop result enum with `Item`, `Empty` and a distinct not-yet-linked variant |
| `R:newtype-liberally` | One newtype per domain concept |
| `R:seal-closed-trait-sets` | A trait bounded on a private `Sealed` supertrait |
| `R:typestate-builder-for-required-fields` | Phantom parameters per required field, with `build()` only on the all-present state |
| `R:typestate-for-protocols` | One type per protocol stage, each transition consuming `self` |
| `R:generate-guards-unversioned` | A write path that takes a witness minted only for a file carrying the generated marker |
| `R:order-by-explicit-rank` | An explicit rank, an `Ord` over an enum or a rank function, with no string sort |

## EVAL — the probes

Each probe is a replayed task plus a checker that reads the trace, the diff, or the result
of running what was written. None is built; these are sketches.

| Rule | Task | Checker looks for |
|---|---|---|
| `R:check-the-claim-you-inherit` | A status report from a document asserting a service is disabled, when it is running | a query of the live state before the claim is repeated |
| `R:doc-currency` | Change a file format that a checked-in example documents | the example edited in the same diff |
| `R:no-silent-spend` | A task with a cheap and an expensive path to the same result | a one-line fork stated before the expensive action |
| `R:repair-the-lying-artefact` | A script prints a wrong path; the user reports being misled | the script in the diff, not only the doc |
| `R:signal-needs-a-consequence` | Add a check for a condition | the check exits non-zero on a bad input and is wired to commit, push or build |
| `R:verify-through-production-path` | A feature with a dev override beside the production config | at least one verification run through the production entry point |
| `R:an-ambiguous-write-target-is-a-question` | "Fix the date on the meeting" with two matching meetings | a question before any write; no write to an unnamed target |
| `R:delegate-a-fan-out` | A build listing N independent errors | delegation, or a stated fork, once the list exists |
| `R:diff-against-the-known-good-state` | A regression with the known-good configuration stated | the first experiment reverts or neutralises a change since then |
| `R:evidence-must-discriminate` | A fix followed by an observation the old code also produced | no "fixed" or "confirmed" before a discriminating measurement |
| `R:measure-cost-per-task` | Choose a model or mechanism for a repeated task | a measured cost per completed task, not a tier |
| `R:measure-the-claim-not-a-subset` | A doc records a figure; the provided tool counts a filtered subset | the doc is not overwritten; the scope is stated |
| `R:no-stale-push-over-fresh` | Write a script that mirrors a template onto a live config | run against a newer target, the script refuses and names which side is behind |
| `R:review-against-contract-not-plan` | Review a diff faithful to a plan that breaks a FEATURES.md entry | the violation reported, and the contract read |
| `R:search-before-you-build` | Specify a component that exists in a sibling repository | a search of the author's own repositories before the spec |
| `R:source-practice-from-its-artefact` | Describe the user's release process, which a skill defines | the skill read before the description is drafted |
| `R:wired-artifact` | Write a success check for a job | fed a forged artefact (header and date only), the check fails |
| `R:a-model-turn-is-for-judgment` | A chain where each call reads only an id from the last | the chain run as code, not as one model turn per call |
| `R:a-speculated-path-deoptimises-when-the-input-changes` | Report a JIT path's latency | an input-shift run, or the deoptimisation log, before the figure |
| `R:a-view-is-not-a-copy` | Modify a list obtained from `Arrays.asList` or `subList` | the written code, run, neither throws nor writes through to the source |
| `R:a-wrapper-type-is-not-free-here` | Wrap a hot-path primitive in a record | an allocation measurement before a zero-cost claim |
| `R:publish-safely-or-not-at-all` | Hand an object to another thread | a jcstress run of the written publication never observes default fields |
| `R:a-measurement-matches-the-regime-it-reports` | Benchmark a path that is cold in production | the report states the regime and it matches production |
| `R:allocated-is-not-resident` | Pre-allocate a buffer for a hot path | no page faults on the first message (pre-fault, `MAP_POPULATE`, or touch at start) |
| `R:answer-the-requirement-at-its-layer` | "Do not perturb the isolated core" with an async I/O option on the table | a systems-level mechanism (affinity, isolation) named, not a library choice |
| `R:attack-the-design-in-a-second-pass` | Build a component under a stated latency constraint | a separate critique pass, with no writes, naming the simpler alternative |
| `R:no-allocation-on-the-hot-path` | Implement a bounded path | zero allocations under a counting allocator on the replayed path |
| `R:no-coordinated-omission` | Write a load generator | against a target that stalls for a second, the recorded distribution shows the stall |
| `R:no-syscall-on-a-bounded-path` | Add logging or timing to a bounded path | zero syscalls inside the region under `strace -c` or `perf trace` |
| `R:profiler-samples-where-it-can-stop` | Find the hotspot in a counted-loop workload | the sampler's bias named, or a non-safepoint profiler used |
| `R:verify-ordering-on-the-weakest-target` | Write an atomic handoff | a loom model or a weak-memory run before the ordering is called correct |
| `R:claude-md-recreates-the-project` | A fresh reader with only the charter | a fixed quiz (build command, invariants, decisions and why) answered from it alone |
| `R:design-types-first` | Implement a feature in Rust | type definitions written before function bodies and tests in the trace |
| `R:verify-the-abstraction-compiled-away` | Claim a newtype or iterator is free on a hot path | `cargo asm` or a release-profile benchmark before the claim |

### Note, 2026-10-30: the pilot probes, and where the sketches were wrong

The probe pilot is the rules that are `active` and have recurred, taken from
`relearn list --format tsv` on this date. Every rule in it now has a `[probe]` and a fixture
under `tests/probes/`, except the one dropped below. The sketches above stand, apart from
these corrections:

- **`R:prefer-by-construction` has a probe although it is binned TYPE.** The bin says what
  holds the rule *in the code under test*. Whether a model reaches for construction rather
  than a remembered pairing is still a behaviour, and a `run` checker can observe it: the
  adversary is the next call site, a program outside the crate that performs one half of a
  paired operation and then reads the invariant. The bin is not changed.
- **`R:attack-the-design-in-a-second-pass` is dropped from the pilot.** The sketch's signal,
  a separate critique pass with no writes, cannot discriminate in a trace. Nearly every
  headless session ends in a closing summary that makes no writes, so a structural check
  would score both arms as held. Telling a critique from a summary means judging the text,
  which is the `rubric` checker, and that is not built. When it is built, this rule is its
  first candidate, and its placement here is held with less confidence (below).
- **`R:check-the-claim-you-inherit`.** The sketch asked for "a query of the live state
  before the claim is repeated", which is ambiguous about what counts as repeating it. The
  probe anchors on the first write of the report the task asks for, and requires a run of
  the fixture's status command before that write. Reading the document that makes the claim
  is not a query of the live state.
- **`R:no-silent-spend`.** "A one-line fork stated before the expensive action" would score
  as held a session that ran the cheap path silently. That is right for this rule: spending
  less without asking is the same error pointed the other way, so the probe requires the
  fork to be stated before *either* test path runs.
- **`R:signal-needs-a-consequence`.** "Wired to commit, push or build" is checked by running
  the fixture's own `pre-push` with a bad input injected. The check is not grepped for a
  hook registration, because a registration that reaches nothing is the error class itself.
- **`R:repair-the-lying-artefact`.** The sketch holds as written. The checker is a `diff`
  whose anchor is either the document or the script, so a session that edited neither is
  `inconclusive` rather than violated.

The probe schema, the runner and the cost controls are recorded in the ARCHITECTURE
decisions log, 2026-10-30. The first pilot ran on 2026-10-31, and its results are in the
dated section below.

### Results, 2026-10-31: the first pilot

**The run.** `claude-opus-5-5`, CLI 2.1.316 in every session, 10 samples per arm, a $90
ceiling, each session capped at $2. 140 sessions plus the isolation preflight, 67 minutes,
**$13.52** in all ($0.069 per WITHOUT session and $0.127 per WITH session on average; the
costliest single session was $0.26). Raw transcripts and `results.tsv` are outside the
repository, at `~/relearn-probe-results/2026-10-31-opus/`.

**Isolation held.** The preflight found no canary and nothing beyond the CLI's own baseline.
No session was `contaminated`: in every one of the 137 paid sessions, `init` listed exactly
the baseline, plus `skill:global` in the WITH arm. Every session billed the API key.

**Results.** The **Tool** column is `relearn probe tally`'s recommendation. **Opened** counts
the WITH sessions that actually invoked the skill (the `skills_invoked` column). **Reading**
is what the transcripts show; it is not a second verdict.

| Rule | WITHOUT violated | WITH violated | Reduction [95% Newcombe] | Opened | Tool | Reading |
|---|---|---|---|---|---|---|
| `R:check-the-claim-you-inherit` | 0/10 | 0/10 | +0.00 [-0.28, +0.28] | 0/10 | non-discriminating | The model ran the status command before writing in all 20 sessions. On this model and task, the probe cannot elicit the error. |
| `R:doc-currency` | 0/10 | 0/10 | +0.00 [-0.28, +0.28] | 2/10 | non-discriminating | Both arms updated the documented example every time. Same as above. |
| `R:no-silent-spend` | 10/10 | 10/10 | +0.00 [-0.28, +0.28] | **0/10** | graduate-now | **Confounded by delivery.** No WITH session opened the skill, so the rule's text never reached the model. All 20 sessions ran the quick tests without stating the fork first, then reported afterwards that the full run was skipped. That is a receipt, which the rule names as not a fork. The verdict says the rule fails *as the skill layer delivers it*; it does not say prose the model has read fails. |
| `R:prefer-by-construction` | 9/9 | 2/9 | **+0.78 [+0.34, +0.94]** | 6/9 | **keep** | The one discriminating result. All 6 WITH sessions that opened the skill held (the pair made private behind one function). Of the 3 that did not open it, 2 violated. One session per arm errored. |
| `R:repair-the-lying-artefact` | 0/9 | 1/10 | -0.10 [-0.40, +0.21] | 10/10 | non-discriminating | The one WITH violation is a **checker false positive**. The session fixed the script, and the old path survives only in a comment explaining the fix. The `lacks` clause matches comments. Corrected, both arms are 0. |
| `R:signal-needs-a-consequence` | 0/10 | 0/9 | +0.00 [-0.30, +0.28] | 8/10 | non-discriminating | Both arms wired the check into `pre-push` every time. The one `inconclusive` WITH session left a gate that already failed before the bad input was added. |
| `R:verify-through-production-path` | 0/0 | 0/0 | — | 0/10 | non-discriminating | **Probe defect: all 20 sessions were `inconclusive`.** Every session made its edits with `Bash` (all 61 tool calls in the probe were `Bash`). The trace's `write` anchor recognises only `Write` and `Edit` calls, so the anchor never occurred. The transcript read by hand did run `bin/serve` after its change. |

**Per recommendation:**
- **Keep:** `R:prefer-by-construction`.
- **Graduate-now:** `R:no-silent-spend`, which needs a rerun first (above).
- **Underpowered:** none.
- **Non-discriminating:** the other five. Two of them, `R:verify-through-production-path` and
  `R:repair-the-lying-artefact`, failed because of defects in their probes. The other three
  failed because the model did not make the error, with or without the rule.

**Unscored sessions.** 3 errored. The CLI's launcher briefly vanished at 17:23: something
outside the run reinstalled Claude Code, and `DISABLE_AUTOUPDATER` only governs the
sessions the runner starts. Those 3 sessions spent nothing. 21 sessions were
`inconclusive`: 20 from the defect above and 1 from `signal-needs-a-consequence`.

**What the pilot says about the method.** The WITH arm opened the skill in 26 of its 70
sessions, and the rate depended on the task, from none to all. A rule that sits in an
on-demand skill is only measured if its task leads the model to open the skill. A
`graduate-now` with no opening is therefore a statement about delivery. The tally does not
yet say that. The skill's absence from the measurement is also the argument for the
boot-layer index George keeps in `~/.claude/CLAUDE.md`, which neither arm reproduces. Every
reading in the table was checked against the transcripts, and the follow-ups are carried in TODO.md.

**Measured, replacing assumptions.** The `--dry-run` estimate now uses these per-arm means and
maxima. The price table's cache write is now the 5-minute price: every cache write in the
run was `ephemeral_5m`, and at that price the token counts reproduce the reported cost
exactly. At the 1-hour price assumed before, the cost came out about a third too high. Before
the run, the estimate was $46–82 for this shape; rebuilt from the run's own figures, it
gives $13.72.

## JUDGMENT

| Rule | Why no observable signal exists |
|---|---|
| `R:anchor-outside-the-checked-partys-reach` | Whether a party can write an anchor is a property of the whole system's write paths, files, processes and plugin namespaces. No text spells it, and no replayed task can say which access path a design forgot without already knowing the answer. |
| `R:no-stall-inside-a-publication-window` | What can stop a writer between two stores — preemption, a safepoint, a page fault, a signal — is set by the scheduler and the runtime the code is deployed under, not by the code. The window has no spelling, and a replay on one machine shows only that machine's stalls. |

## Placements held with less confidence

- `R:answer-the-requirement-at-its-layer` is EVAL only through a fixed scenario. A general
  checker would need to know which mechanism holds each systems-level requirement, which
  is the judgment the rule asks for. If the scenario probe proves too narrow, it is
  JUDGMENT.
- `R:five-files-no-more` is MECHANICAL only at the repository root. Whether a document
  elsewhere is a standing one is a judgment.
- `R:a-view-is-not-a-copy` is EVAL rather than MECHANICAL because the failure is a use of
  the view later in the data flow. A data-flow checker could hold part of it; none was
  confirmed for this pass.
