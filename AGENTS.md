# Agent instructions

## Update the doc in the same change as the thing it describes [R:doc-currency]

When you change a process, format, or artifact, update every checked-in description of it -- list, example, doc snippet -- in the same change. A doc that lags the code it describes is worse than no doc: a reader trusts it and acts on stale guidance. The behavior contract and its examples are only load-bearing if they are true.

Not to be confused with the sibling rule on a document's *internal* consistency
after an edit -- antecedents, cross-references, counts -- which currently lives
in the `~/.claude` skill layer under the tag this rule used to carry. This rule
is about a document's *external* currency: it still describes a reality that has
since moved. A file can pass one and fail the other.

The sibling is still named in prose rather than cited by tag, because it is not
in this library yet and a real citation would genuinely dangle. The *provenance*
above may now name it freely: since 2026-08-16 the linter reads citations from
rule bodies only, so recording why a tag changed no longer trips it.

## Make illegal states unrepresentable [R:make-illegal-states-unrepresentable]

Before writing logic, design the types so invalid states cannot be constructed: sum types over boolean flags, one field that cannot contradict another. If two fields can disagree, redesign until they cannot. R:no-sentinel-values is the corollary -- an absent or stopped state is an enum variant, not a magic value the surrounding logic must remember to special-case.

## Measure cost-per-completed-task; never choose by price tier [R:measure-cost-per-task]

Do not pick a mechanism or model by reputation or sticker price. State what it actually costs to complete the task -- tokens consumed times price, including retries -- and choose on that measured cost. After choosing, run one failure-mode check: under what configuration does this cause the exact harm it was chosen to prevent? Then bound that configuration.

## No sentinel values: absent states are enum variants [R:no-sentinel-values]

If "absent / stopped / unknown" is a real state, make it an enum variant, not a magic value of an existing type. Downstream code will forget to special-case a sentinel; it cannot forget a variant the compiler forces it to handle. If a range check reads a sentinel as a real quantity, it fails in the direction of the sentinel, not of safety.

## Never route judgment work to a weak model, and never embed a sub-tier local LLM [R:no-weak-model-for-judgment]

Work that needs judgment goes to a capable model. Do not embed a sub-tier local LLM (a 7B/13B behind Ollama, llama.cpp or similar) in a tool as a convenient, API-key-free fallback: a meaningfully dumber model degrades the tool it is wedged into, everywhere and silently, and the output looks like ordinary tool output rather than like a downgrade. When a tool needs intelligence, delegate to the capable model through the existing subscription -- the MCP server is the abstraction boundary and clients are peers. Note that the cost argument usually offered for the local model is the price-tier fallacy R:measure-cost-per-task names; but this rule is not an economic one and does not dissolve if the sums come out favourably. Mechanical, tool-restricted passes are a different matter and may be scoped tightly; the floor applies to work where the answer is a judgement.

## Pin the line endings of text a machine executes or hashes [R:pin-eol-for-executable-text]

When a file's exact bytes are load-bearing, do not leave its line endings to whatever the
checkout decides. Two kinds of file qualify: text a machine executes, where a stray
carriage return on the shebang means the interpreter is not found; and generated text
whose content is hashed and compared, where a rewritten line ending changes the hash and
the file reports as modified by a human who never touched it.

Pin it structurally, in `.gitattributes`, for the paths that need it. This is a
by-construction fix applied once at the repository boundary, not a check that runs later
and reports what has already gone wrong.

Do not repair this by teaching the comparison to forgive line endings. Normalising before
hashing makes the check tolerant of a class of real edit, and it hides the platform
difference rather than removing it. The guarantee is about the bytes on disk, so it is the
bytes on disk that must be fixed.

The characteristic damage is that it is invisible from where the work was done. The
authoring platform stays green; the breakage appears only on the other one, and it appears
as a symptom that names something else entirely -- a missing interpreter, a hand-edited
file, a failing gate with nothing wrong in the diff.

This is the at-rest half of a pair. Its sibling governs bytes in flight -- what another
tool emits into a pipe at runtime, which no file attribute can reach, so that fix belongs
at the consuming end instead. A repository can satisfy either and fail the other.

## Prefer by-construction impossibility over after-the-fact controls [R:prefer-by-construction]

When a class of mistake can be designed out, design it out, rather than adding a control that catches it after the fact. A control that catches a mistake still admits the mistake; a design that cannot express the mistake retires the whole class. Rank the options by how little must be remembered for them to hold: a type the compiler enforces beats a test that samples beats a review step that relies on attention. R:make-illegal-states-unrepresentable is this rule in the type system.

## Repair the artefact that made the false claim, not only the doc about it [R:repair-the-lying-artefact]

When an incident traces to a false claim, find the artefact the person was actually
reading at the moment they were misled, and repair **that** first. Usually it is not the
document — it is something that printed during the work: a script's final "Binary:" line,
a deploy script's success message, a generated file's header, a status endpoint, a test
name. A document is consulted; an artefact is *emitted at you* while your attention is on
the task. That asymmetry decides who gets believed.

A doc and a script disagreeing is not a tie. People run the script. Correcting the doc and
stopping there leaves the defect fully operational and adds a paragraph that makes it look
handled — the worst of both, because the next occurrence now has a written warning standing
over it as evidence that someone already dealt with this.

Apply the test before closing: **could the same person be misled again in exactly the same
way without ever opening the document I just fixed?** If yes, the fix has not landed. Ask
also what else in the repo asserts this same fact — a README snippet, a CI summary, a
printed usage line — because a claim usually has more than one mouth.

Then push it down a layer rather than restating it: make the artefact *derive* what it
reports instead of asserting it (resolve the path, stat the file, read the version it
actually built), and add the mechanical check that fails any future artefact making the
unresolved claim. Deriving beats asserting for the same reason
`[R:prefer-by-construction]` prefers designs to guards — a derived claim cannot drift from
what it describes, so it cannot go stale the way `[R:doc-currency]` describes.

This is the reporting half of `[R:verify-through-production-path]`. That rule says to
exercise the real channel; this one says the real channel must also tell the truth about
what it produced — verifying through a production path that reports a path it never
resolved proves nothing.

## Describe a practice from the artefact that defines it, never from the genre [R:source-practice-from-its-artefact]

Before writing anything that *describes* how the user works -- their process, their file
layout, their tooling, their organisation, their role -- enumerate the primary artefacts
that define it, and read them first. The definition of a practice is the artefact that
embodies it: the skill that specifies it, the repository that implements it, the document
that governs it. Domain literature and industry convention describe the *genre*, not this
instance. A description sourced from the genre will be fluent, well-structured, plausible,
and about somebody else.

Run this check before drafting, not after: **what artefact would prove this description
wrong, and have I read it?** If the answer names a file that has not been opened, open it.
If no such artefact exists, the description is an assumption -- ask, and say plainly that
the question is being asked because nothing available settles it.

Clarifying questions must cover the *subject*, not only the framing. Asking about audience,
evidence and venue while silently assuming what the thing is produces a well-scoped,
well-sourced artefact about the wrong topic, and the correction costs a rewrite rather than
an edit. The cheapest question is the one that establishes what is being described.

This is the reading half of the sibling rule on building: search the existing artefacts
before constructing something new. That one prevents rebuilding what exists; this one
prevents *describing* what does not.

## Verify through the production path [R:verify-through-production-path]

Before declaring anything verified, run at least one check through the exact channel production uses: same env var, same startup script, same config file, same transport. A test that exercises a stand-in is evidence the stand-in works, not that the feature does. Ask: which line of production wiring did my test NOT execute? That line is where it breaks.

## Async is async all the way down [R:async-all-the-way]

No `block_on` inside async code. An async runtime multiplexes many tasks onto few threads, so a blocking call does not delay one task -- it removes a worker from the pool for the duration and delays every task that would have run there. The victims are unrelated to the code that blocked, which is why the symptom is unexplained tail latency somewhere else entirely, and why it is close to unattributable after the fact.

`block_on` is correct only at the boundary where synchronous code enters async: `main`, a test, a callback from a C library. Once inside, stay inside -- async I/O, an async-aware lock wherever a guard must survive an `await`, and `spawn_blocking` for work that genuinely blocks, such as CPU-bound compute or a synchronous third-party client.

The same applies to a `std::sync::Mutex` guard held across an `await`. Nothing flags it as blocking, but it holds a lock while the task is descheduled, so the contending task blocks its own worker thread and the failure presents as a deadlock rather than as a lock.

Nested `block_on` on a current-thread runtime does not degrade -- it deadlocks outright. That is the honest failure; the multi-threaded case merely hides the same mistake behind a thread count, until load removes the hiding place.

## Take a borrow in the signature unless the function needs to own [R:borrow-in-signatures]

Take `&str` over `String`, `&[T]` over `Vec<T>`, `&Path` over `PathBuf`. A signature is a statement about what the function needs, and an owned parameter says "I will keep this" -- so when the body only reads, the signature is untrue and every caller pays for it in an allocation.

Own the parameter exactly where the function stores it. There, `impl Into<String>` is the courteous form: a caller holding a `String` moves it in for nothing, and a caller holding a `&str` allocates once, knowingly. On the way out, return `&str` rather than `&String` -- the extra indirection buys the caller nothing and pins the field's representation into the public API.

The cost is rarely the single allocation. It is that an owned parameter propagates: the caller clones to satisfy it, its caller clones to satisfy that, and a signature chosen without thought becomes a column of clones that each look locally necessary. R:justify-every-clone is where those clones surface; this rule is how they are never created.

## Design by writing the types first, before any logic [R:design-types-first]

Sketch the types until the design falls out, then write the logic. The type-level sketch is the executable specification a test-first red phase is reaching for, and it is the stronger one: a test samples points of the behaviour space, a type constrains the whole space and the compiler proves it everywhere, at compile time, for as long as the code exists. When the types are right much of the implementation writes itself, and many wrong implementations stop compiling. In Rust this ordering supersedes any test-first default -- tests are not removed, they are demoted to the layer where they are the right tool: property tests for behavioural laws the types cannot encode, unit tests as regression pins for past bugs. Apply each guarantee at the strongest layer that can hold it, and when reviewing, ask first not "does it pass" but "which of these guarantees could move up a layer?"

## Error enums carry the values that identify the failure [R:errors-name-what-failed]

Every distinct failure is a variant of a `thiserror` enum, and every variant carries the values that identify the instance:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    #[error("TCP connect to {addr} failed")]
    Tcp { addr: SocketAddr, #[source] source: io::Error },
    #[error("timed out after {elapsed:?}")]
    Timeout { elapsed: Duration },
}
```

Two things follow that a string cannot give. The caller can `match` -- retry a `Timeout`, surface a `Tcp` -- rather than parsing prose that changes the next time someone edits a message. And the message names the actual address, so one log line is enough to act on rather than the start of an investigation.

Keep the cause in `#[source]` instead of interpolating it into the text. The chain then prints once, at the edge, without each layer restating the layer beneath it.

One variant per condition a caller could plausibly treat differently. Collapsing four causes into `Other(String)` re-creates the string error inside an enum: it reads as a type and behaves as prose, and it is the shape this rule exists to catch.

## Every clone() carries its reason, or the design is wrong [R:justify-every-clone]

A borrow-checker error is a question about ownership. `clone()` does not answer it; it pays to avoid answering it. Try the answers first: restructure so one owner is obvious, take a borrow with a named lifetime, split the borrow across smaller fields, or share with `Arc`/`Rc` where the value is genuinely shared rather than copied.

Where a clone survives that examination, write the reason beside it, in the house form used throughout this codebase:

```rust
let sources: Vec<RuleTag> = rules.iter().map(|r| r.tag().clone()).collect(); // allow:clone: the OutputFile owns its provenance, outliving the &Library borrow
```

The comment is what makes the rule reviewable. Unannotated clones are indistinguishable from one another, so a reviewer has to re-derive the ownership argument for every one and in practice re-derives it for none. An annotated clone states a claim that can be checked, and that can be found and removed the day the design around it changes.

The reason must be about ownership or lifetime. "To make it compile" is the error class restated, not a justification. Two copies of a value the code believes is one value is a correctness bug waiting for whoever mutates the wrong one.

Most clones that survive review were created by a signature, not by a call site: R:borrow-in-signatures is where the pressure comes from, and fixing the parameter usually deletes the clone rather than annotating it.

## Modules are the encapsulation boundary; pub is a deliberate export [R:module-visibility-is-deliberate]

Use `pub(crate)` and `pub(super)` freely; reserve bare `pub` for what the crate deliberately exports. Visibility is not paperwork. It is the statement of what may still be changed freely, and it is the only such statement a compiler can check.

The default matters because `pub` is cheap to add and expensive to remove. Once an item is public an external caller may depend on it, so the module can no longer be reorganised, the field can no longer be renamed, and the helper written for one call site is now a supported API. None of that was decided; it was defaulted into.

A module holds an invariant in the same way a type does. A helper that may only be called after a check belongs beside that check, private to the module, so "only after" is enforced by nobody elsewhere being able to call it at all. Making it public converts that guarantee into a doc comment.

Re-export the intended surface explicitly at the crate root with `pub use`, and let everything behind it be as private as it can be. The public API is then a list someone wrote, rather than the residue of where the code happened to live.

R:private-fields-only is the same argument one level down, at the field rather than the item. The two are separable -- a private field on a `pub` type leaks structure, a `pub(crate)` type with public fields leaks none -- so satisfying either says nothing about the other.

## Mark consequential return values #[must_use] [R:must-use-on-consequential-returns]

Put `#[must_use]` on every function that returns a `Result`, and on every function whose return value is the point of calling it. The compiler then flags a discarded outcome at every call site, present and future, instead of leaving it to a reviewer to notice one bare statement among a hundred.

Prefer the attribute on the *type* over the attribute on the function. `#[must_use] struct Receipt;` travels to every function that returns a `Receipt`, including the ones written later by someone who never read this rule; the function-level attribute has to be remembered each time. Attach it to the type whenever the type is always consequential, and fall back to the function only for the narrower case where the same type is sometimes worth discarding.

Escalate the resulting warning to an error in CI. A `#[must_use]` whose violation prints a note nobody reads is a comment with extra syntax, and the guarantee it claims is held nowhere.

## Newtype liberally: distinct concepts get distinct types [R:newtype-liberally]

Give every domain concept its own type, even when the underlying representation is identical. `Miles(f64)` and `Kilometers(f64)`, `Host(String)` and `Port(u16)` -- never two bare `f64`s or a `String` and a `u16` whose order only a human remembers. Newtypes are ordinarily zero-cost: they compile to the same machine code as the primitive, so the only thing they add is the compile error you want -- but where that cost is load-bearing, R:verify-the-abstraction-compiled-away requires the claim to be checked rather than repeated. The unit mix-up that cannot be written is cheaper than the one caught in review, and far cheaper than the one that is not. R:parse-dont-validate is where the newtype comes from -- the boundary mints it as a witness; this rule is about carrying it everywhere afterwards instead of unwrapping back to the primitive.

## No anyhow in library return types [R:no-anyhow-in-libraries]

> Also enforced by hook:no-anyhow-in-lib.

Library crates return typed error enums (thiserror), so a Result says exactly what can go wrong and callers can match on it. anyhow belongs only at the outermost binary edge. This rule has graduated: the no-anyhow-in-lib hook now enforces it deterministically at write time.

## No unwrap() in production code [R:no-unwrap-in-production]

> Also enforced by hook:no-unwrap-in-src.

No unwrap() in production code. Use expect() only with a meaningful panic message that names the resource and the invariant, or return the error with `?` and let the caller decide how to surface it. This rule has graduated: the no-unwrap-in-src hook now enforces it deterministically at write time, so the instruction layer no longer has to.

## Parse, don't validate [R:parse-dont-validate]

Transform raw input into a rich domain type at the outermost boundary, producing a witness newtype whose existence proves the check happened. Interior code takes the witness and never re-checks -- one perimeter, one check, enforced everywhere after by the compiler. R:parse-wide-then-range-check sharpens this: the boundary must be able to see the illegal value in order to name it.

## Parse wide, then range-check [R:parse-wide-then-range-check]

Parse into a type wide enough to *represent* the out-of-range value, then range-check to mint the narrow newtype. The perimeter must be able to see the illegal value in order to name it illegal; parsing directly into the target type collapses "out of range" into "not a number" and makes the OutOfRange class a lie the compiler will not catch.

## Struct fields are private; construction goes through a constructor [R:private-fields-only]

Domain types have private fields. The only way to construct one is a smart constructor that enforces the invariant and returns a `Result`; the only way to read one is an accessor.

A `pub` field is a second constructor that checks nothing. Every guarantee the smart constructor establishes is void the moment a caller can write the field directly, and the type's name goes on claiming it. This is the same defect as a validator that some call sites skip, moved from the function layer down to the field layer, where it is harder to see.

Hand out the narrowest borrow the caller can use: `fn host(&self) -> &str`, not `&String`, and never `&mut` on a field the invariant depends on. Where a caller genuinely must change the value, give it a method that re-establishes the invariant, or one that consumes the value and mints a new one.

R:parse-dont-validate is what the constructor does; this rule is what makes it the only door.

## Seal a trait whose set of implementors is closed [R:seal-closed-trait-sets]

When a trait exists to describe a fixed family -- the kinds of unit, the supported wire formats, the stages of a protocol -- seal it, so only this crate can implement it:

```rust
mod private { pub trait Sealed {} }
pub trait UnitKind: private::Sealed { fn path_suffix() -> &'static str; }
```

The point is not to be unwelcoming. It is that "closed" is either a fact the compiler enforces or a sentence in a doc comment. An unsealed trait can never gain a required method without a breaking change, can never be reasoned about as a whole set, and can never assume it has seen every implementor -- yet the code around it will be written as though it can.

Seal by default where the set is closed today, and leave the trait open only where third-party implementations are a deliberate feature. Opening a sealed trait later is additive and painless. Closing an open one is a breaking change, which in practice means it never happens.

## Builders make a missing required field a compile error [R:typestate-builder-for-required-fields]

Track each required field in a phantom type parameter, and implement `build()` only for the fully populated combination:

```rust
pub struct Missing;
pub struct Present;
pub struct ServerBuilder<HasHost, HasUser> { /* ... */ }
impl ServerBuilder<Present, Present> { pub fn build(self) -> Server { /* ... */ } }
```

An incomplete build is then not an error value to handle but a method that does not exist. Compare the alternative: `build()` returns `Result<_, MissingField>`, every caller writes the same `?`, and the one caller who writes `unwrap_or_default()` ships a server pointing at nothing.

This is R:typestate-for-protocols applied to construction rather than to sequence -- the same remedy, a different illegal thing. Optional fields stay plain setters; only what is genuinely required earns a parameter.

Beyond three or four required fields the parameter list costs more than the problem. Take a required-arguments struct in `new()` instead: the same omission is still a compile error, with none of the machinery.

## Typestate for protocols: out-of-order calls should not compile [R:typestate-for-protocols]

When a sequence has rules -- connect before authenticate, init before run, configure before start -- encode the stage in the type, not in a field. Each step consumes the value in one state and produces it in the next, so a method that is invalid in the current state simply does not exist and calling it is a compile error. This is R:make-illegal-states-unrepresentable applied to time rather than to structure: the illegal thing is not a contradictory pair of fields but an operation at the wrong moment, and the same remedy applies -- make it unrepresentable rather than guarded. A runtime `if !self.authenticated { return Err(...) }` in a method that should not exist yet is the shape to look for.

## Verify a zero-cost claim; never assert it [R:verify-the-abstraction-compiled-away]

"Zero-cost" is a property of a particular abstraction, under a particular optimiser, on a particular build profile. In Rust it is usually true, which is precisely why it gets asserted instead of checked, and why the cases where it is false survive review.

Where the cost is load-bearing -- a hot path, a latency budget, an allocation-free claim -- look at what was emitted. `cargo asm` for the function, `cargo bloat` for the binary, a benchmark on the profile that actually ships. A debug build proves nothing about a release binary: a newtype that is free at `opt-level = 3` need not be at `opt-level = 0`, and the two are different programs.

Then run the failure-mode check R:measure-cost-per-task states in the general case -- under what configuration does this cost exactly what it was chosen not to cost? A `#[repr(transparent)]` newtype crossing an FFI boundary, an iterator chain that fails to fuse because the closure captures by reference, a generic that is never monomorphised because it went out through a trait object. Bound that configuration, or drop the claim.

This is R:verify-through-production-path applied to code generation: a claim measured on a build the user never runs is evidence about that build alone. Where the cost is not load-bearing, do not make the claim at all -- an unverified performance assertion in a doc comment is read as a measured one.

## A test fixture must work on every OS the repository runs on [R:xplat-fixtures]

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

## A generator never overwrites content it did not generate [R:generate-guards-unversioned]

A generator that writes into a directory shared with hand-authored files must never overwrite a file it did not itself generate. Stamp every generated file with a marker the generator can recognise on the next run, refuse to overwrite any target lacking it, and make the check all-or-nothing: abort the whole write before touching disk if any target is unversioned, rather than leave a half-generated tree. A dropped rule is a lost correction; a clobbered human file is a lost correction the tool itself destroyed.

## Order by an explicit rank, not an incidental string sort [R:order-by-explicit-rank]

When output has a meaningful order, derive it from an explicit rank that states the intent (general to specific, most to least severe), not from an incidental lexical sort of some string field. An alphabetical accident is not an ordering decision, and it changes silently the day the underlying strings change.

## Keep secret-bearing, machine-local config out of a shared config repo [R:no-secrets-in-config-repo]

When a repository versions configuration that is meant to be shared, keep the secret-bearing and machine-local parts out of it: credentials, permission allowlists, and per-machine settings are not shareable configuration and do not belong in shared history, even (especially) when the repo is otherwise a config repo. Version the shareable wiring separately from the secrets, so tracking the former never commits the latter. A secret in history is a secret to rotate, not a secret to delete.

## Verify a moved file is still tracked in a whitelist-gitignore repo [R:verify-tracked-after-move]

After moving or renaming a tracked file in a repository whose .gitignore is a whitelist, verify the file is still tracked before considering the change done. A whitelist ignore silently drops anything outside its re-included paths, so a relocation can remove a file from version control with no error and no diff line to notice. Run the repo's tracking/deploy verification as the gate: the failure mode is invisible precisely when you most assume the move was safe.

<!-- relearn:generated v0.1.0 sha256=e463e0c3b057c43866f05a88209e2b3a82c4e4d4399858dc4e851c6f3ef7f57a rules=R:doc-currency,R:make-illegal-states-unrepresentable,R:measure-cost-per-task,R:no-sentinel-values,R:no-weak-model-for-judgment,R:pin-eol-for-executable-text,R:prefer-by-construction,R:repair-the-lying-artefact,R:source-practice-from-its-artefact,R:verify-through-production-path,R:async-all-the-way,R:borrow-in-signatures,R:design-types-first,R:errors-name-what-failed,R:justify-every-clone,R:module-visibility-is-deliberate,R:must-use-on-consequential-returns,R:newtype-liberally,R:no-anyhow-in-libraries,R:no-unwrap-in-production,R:parse-dont-validate,R:parse-wide-then-range-check,R:private-fields-only,R:seal-closed-trait-sets,R:typestate-builder-for-required-fields,R:typestate-for-protocols,R:verify-the-abstraction-compiled-away,R:xplat-fixtures,R:generate-guards-unversioned,R:order-by-explicit-rank,R:no-secrets-in-config-repo,R:verify-tracked-after-move -- DO NOT EDIT; regenerate with `relearn build` -->
