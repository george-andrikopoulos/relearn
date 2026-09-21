# TODO.md — relearn

*Contains every `NOTHING YET — exposed` gap from FEATURES.md until closed, plus discovered work.*

**Markers.** `[ ]` open work someone could start today. `[x]` done. **`[!]` carried
exposure — recorded, understood, and blocked on something this repository cannot supply.**

The third marker was put to use on 2026-09-19, when a sweep of the list found that a
majority of what read as a backlog was not work at all. Three things were sitting under
`[ ]` and could not move: the **federation** items, every one of which is gated on a
second install existing — there is one contributor and one subscriber, both George, so
nothing about whether a rule mined from one person's sessions helps anyone else can be
exercised, measured or validated; items belonging to **another repository**
(`Design-Architecture-Tool`, `stochos-lab`, `~/.claude`), which this tree cannot edit;
and **paper and study-design** items waiting on a venue or a field site.

Marking them is not closing them — no word was deleted and each still states its own
exposure. It stops the list claiming that twenty-one things are waiting for an afternoon
when they are waiting for a second person, a different repository, or a journal. A
backlog that cannot be worked is one nobody reads, which is how a real item hides among
them. `[R:doc-currency]`

## Phase 0 — scaffold
- [x] Five project files authored (CLAUDE, ARCHITECTURE, FEATURES, TODO, README)
- [x] Relocated to `%USERPROFILE%\Documents\relearn` (the live Windows dev area, beside win-health-mcp/mesh-watchdog/Design-Architecture-Tool). NB: the named "dev root with Ferridis/sysmand/linux-health-mcp" was `D:\linux-george`, a stale Linux-home backup — see 2026-08-13 note.
- [x] `cargo init` (bin + lib, edition 2024); module skeleton per ARCHITECTURE; `git init`; **private** GitHub repo `george-andrikopoulos/relearn`
- [x] CI: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` on push + PR


## Phase K — provenance for codified rules (shipped 2026-09-18), and two gaps it exposed

- [x] **`Origin::Codified` carries an optional `SourceArtefact`.** `source = "..."` beside `origin`, mirroring the mandate's `approval`; refused on any other origin and refused blank; `Origin::parse` takes all three halves at one perimeter with no catch-all over the triple. Absent stays the common case — all 57 committed rules parse and emit unchanged, and bare `verify` reported 69 files up to date across the change. 7 pins in `tests/codified_source.rs`; `arb_origin` now generates **both** codified shapes, because they share the spelling `codified` and a round-trip that only saw the payload-free one would pass while the serializer dropped every source in the corpus. Full account in the decisions log.
- [x] **The source is announced in every emitted format**, spliced fourth and last of the notes (`[R:order-by-explicit-rank]` — provenance is the reader's last question, not their first). Held by the same pair the audience note is: a counting wiredness property across five splice sites, and `a_source_reaches_a_body_only_through_the_source_note`, which deletes the note and requires the remainder byte-identical to a sourceless build. Verified through the binary as well as in-process.
- [x] **A defect the change would have caused, found and closed in the same change.** `source` is the third authored field that travels on `contribute`, and the banned-terms scan list lived at the CLI call site, named two, and carried a comment asserting there were only two — so a new authored field reached the drive unscanned by being absent from a list somewhere else. The enumeration moved to `Contribution::authored_texts`, beside the projection that decides what travels. `[R:names-travel-with-the-quote]`
- [x] **`[R:verify-the-abstraction-compiled-away]` widened to cover the benchmark the optimiser deleted (2026-09-18).** A repair to an existing control rather than a new rule beside it, which is step 3 of the relearn skill: the class is "reading an absence of cost out of an optimiser nobody inspected", and asserting zero cost from reputation and accepting a near-zero benchmark are its two directions. The second is the more dangerous, because it arrives carrying a measurement — an implausibly good number is the only signal, and it is exactly what the author was hoping for. Error class and title both widened; three output roots rebuilt and re-verified. The `pack_counts` gate caught the consequence immediately: `copilot-pack/README.md` claimed 905 lines against a file that now has 909, which is the hand-written number the gate exists to catch, fixed in the same change.
- [x] **`relearn new` ships (2026-09-18) — the request this whole thread opened with.** Authoring a rule meant hand-writing eight required keys and two payload correspondences that are parse errors rather than warnings. The command takes every field at the perimeter and hands the assembled `Rule` to the same witness, serializer and guarded write path `adopt` uses, so its output is byte-identical to the same rule typed by hand. Ten unit pins; smoke-tested through the real argv path. Note what it is **not**: a substitute for writing the rule. The diagnosis at the top of this thread stands — the blocker was never the TOML, and a scaffold that made it faster to write the wrong thing would have been answering at the wrong layer.
- [x] **The create-never-replaces defect, found by running the binary (2026-09-18).** `fsio::write_rule`'s guard permits overwriting the *same* rule, which is right for `adopt` and wrong for a create: `new` replaced a rule with a changed title and discarded its incident, its recurrences and its body. A correction lost, by the tool whose subject is not losing corrections. The unit tests had only exercised the foreign-file case and passed throughout. Pinned by `new_never_replaces_an_existing_rule_even_with_itself`. `[R:verify-through-production-path]`
- [x] **Two gates caught their author in the same change, and both deserve recording.** `tests/pack_counts.rs` refused a hand-written line count that a rule-body edit had made stale. `tests/exhaustiveness.rs` refused to *run* when a second test-module marker appeared in `src/cli.rs` — it truncates a file at the first marker, so a second one would have left production code unscanned and the gate would have passed while measuring less than it claims. Then it fired a second time, on the comment written to explain the first, because the comment spelled the marker out and the gate counts occurrences textually. `[R:detector-excludes-own-definitions]`, live.
- [x] **The truncation rank is explicit (2026-09-18).** `Status::instruction_reliance` → recurrence descending → tag. A dropped title is a rule the matcher cannot fire on, and the old order was the alphabet: on a sixteen-rule low-latency home the three dropped were the three whose tags sort last, including the two independently ranked most valuable. On the real corpus this moved `global`'s three graduated rules out of the description and its recurred rules to the front, spending the same fifteen slots on the rules that have nothing else holding them. `domain-rust` was unaffected (it does not truncate, 972 of 1024). Verified by probe: the rank was reverted to the tag sort, all three ordering pins failed, and they passed on restore — a test that passes under the behaviour it forbids locks nothing. `[R:order-by-explicit-rank]` `[R:wired-artifact]`
- [x] **Recurrence now outranks reliance in the truncation rank (2026-09-19).** The two keys were swapped. Reliance-first was defensible alone — the rank answers what it costs for a title to be absent, which is a question about what else holds the rule — and inconsistent with everything around it: `lint` fails CI on an unheld recurrence, the session-start hook leads with it, and the framework treats it as the number that says whether a rule is working. The live casualty was `[R:guarantee-needs-a-reader]` (`partial`, one recurrence), dropped from `global`'s description while never-fired rules were kept; it is back, and `global` now leads `doc currency; no silent spend; verify through production path; guarantee needs a reader`. Pinned by `a_recurred_rule_outranks_a_never_fired_one_even_when_it_is_held_elsewhere` — the only one of the three ordering tests that discriminates, verified by probe: reverting the swap fails exactly that test and no other.
- [x] **Closed 2026-09-19: the field stopped being an inventory.** Was: the rank decides which titles are dropped, not that titles are dropped. `global` drops fifteen of thirty; `domain-low-latency` will truncate again as it grows past about a dozen. Cutting two rules from the drafts took the drop from five to three and left the same rules invisible, because pruning changes the count and never the selection. What remains is the question the rank cannot answer: 1024 characters cannot hold both an inventory and a trigger, so the field has to stop being an inventory. The options from the load-path entry below stand, and (b) — derive trigger vocabulary from `error_class` rather than listing titles — is the one this makes most attractive, because it changes what the field is *for* instead of rationing what it lists. **The fix was to list tag bodies rather than titles** — `no coordinated omission`, `a view is not a copy` — which cost 749 characters for `global`'s thirty rules where the titles cost 1824. Every home now fits: global 884/1024, low-latency 612, rust 508, java 457, no remainder counter anywhere. Held by `tests/description_reaches_every_rule.rs`, a budget check over the real corpus that fails the day a home outgrows the field. The rank stays for the residual case, and the note above about option (b) is superseded: the tag already carries the vocabulary `error_class` would have supplied, in a quarter of the space and without a format change.
- [x] **The compliance rules are rehomed, and `domain=disclosure` exists (2026-09-20).** `[R:employer-identity-not-in-public-artefacts]`, `[R:no-tool-attribution-in-public-artefacts]` and `[R:repository-private-by-default]` came out of `skills/second-brain` — which loads only when that vault workflow does — and joined `[R:names-travel-with-the-quote]` and `[R:report-the-hit-not-the-match]` in a new home. All written generically; the hashed term list stays machine-local. The vault skill keeps pointers.
- [x] **`global` split 38 → 33 because the description budget is a real cap.** `tests/description_reaches_every_rule.rs` failed at 1010/1024 with five subjects dropped, two of them prose-only. Letting *graduated* subjects drop was tried and measured out: only two of 38 were graduated, so the prose-reliant 36 were 60 characters over before any choice existed. Now 956/1024.
  - [ ] **Headroom is ~68 characters — about two more global rules.** The gate will fire again, and soon at the current rate. The prepared answer is the second split: the seven project-document rules (`claude-md-recreates-the-project`, `five-files-no-more`, `decisions-log-records-rejected-alternatives`, `features-ledger-names-its-artefact`, `definition-of-done-every-change`, `doc-currency`, `revision-integrity`). **Not taken now** because the natural domain name collides with the existing hand-authored `project-discipline` skill, and that collision is really a question about which of the two owns the five-files discipline — a `[R:one-home-per-rule]` question deserving its own pass rather than a naming dodge.
- [ ] **72 commits carry an assistant `Co-Authored-By` trailer; 67 are pushed.** Measured 2026-09-20 across relearn and stochos-lab, against a standing rule of never. Three separate decisions, none taken here: (a) set `includeCoAuthoredBy: false` in `settings.json` — the key is absent and the default is on, so nothing currently prevents the next one; (b) what to do about the 67 published, where the only remedy is history rewriting across two repositories, one of them public and cited by three papers; (c) whether the 5 unpushed are worth amending while they still can be. George's call on all three; the rule and the repaired note are in place either way.
- [x] **`[R:money-is-not-a-float]` written (2026-09-20).** Mined from a greenfield financial build that selected binary floating point for monetary fields with the full discipline loaded and the domain stated in its documentation. Checked first for a recurrence and there is none: `[R:newtype-liberally]` covers two concepts becoming interchangeable and its own worked example is `Miles(f64)` beside `Kilometers(f64)`, so it *endorses* the shape at issue and is homed `domain=rust` besides; `[R:make-illegal-states-unrepresentable]` covers contradictory states, and a float holding a decimal is lossy rather than contradictory. `global`, no `applies_to` — `double` in Java and C++ and `float` in Python fail identically.
- [~] **`[R:money-is-not-a-float]` shipped with NO enforcing control; it now has one, holding half.** Written 2026-09-20 as `status = active` — deliberately not `partial`, because `Partial` must name what its controls do *not* cover and with no control the answer was the entire class. **Moved to `partial` on 2026-09-21**, when the scan below landed: there is now a nameable remainder, and leaving it `active` would have understated its own coverage, which is the defect the 2026-09-19 status repair found in five other rules. The remainder is the half the incident came from and the scan cannot reach — a greenfield decision has no declaration to read — so this row stays open until the structural fix lands. `[R:features-ledger-names-its-artefact]` `[R:signal-needs-a-consequence]`
  - [x] **Interim: a source scan (2026-09-21).** `scripts/no-float-money.sh`. Flags `f32`/`f64` in a monetary role across three shapes — a field or parameter, the tuple newtype the rule names by name (`struct Price(f64);`, which satisfies `[R:newtype-liberally]` completely and is still the defect), and the alias (`type Notional = f64;`). Reads the **concept**, not the spelling: camel-case and underscore components, de-pluralised, matched against the thirteen-word vocabulary this row originally listed — so `unit_price`, `NotionalAmount` and `fees` hit while `costume_weight`, `valuation_model_id` and `totality` do not. `[R:detector-excludes-own-definitions]` is handled by construction rather than by a filter: the vocabulary lives in a `.sh` file and in fixtures excluded by path, and only `.rs` files are read, so neither the script's header nor the rule file can match it. An explicit `// money-scan: measured` escape exists because a check that goes red on the first honest false positive is a check that gets muted.
    - **Its own detector has a reader.** `--self-test` runs the matcher over two committed fixtures — `caught.rs` must produce byte-for-byte the eleven hits in `caught.expected`, `clean.rs` must produce none — and reads both **exit codes** as well as both outputs, so an awk that died cannot pass as a clean file `[R:guarantee-needs-a-reader]`. Probed on every path before being trusted: clean → 0, seeded tree → 1 naming all eleven, missing root → 2, `awk` removed from `PATH` → 2, `find` removed from `PATH` → 2, fixtures moved → 3. The `find` probe exists because the design pass found the hole: the first version checked only for `awk`, and a missing `find` yields an empty file list that the scan would have called a clean tree — a false green inside a gate for a rule about false greens `[R:verdict-survives-the-channel]` `[R:attack-the-design-in-a-second-pass]`.
    - **It is built to leave.** A monetary field will never appear in `relearn`, so a `cargo test` here could only report green by measuring nothing `[R:wired-artifact]`. The roots are command-line arguments and FEATURES.md carries the two-command install into a financial repository; running it here keeps the detector exercised on a real tree rather than on fixtures alone.
  - [x] **Attached to the push, never to a command (2026-09-21).** `.githooks/pre-push` runs the self-test and then the scan, reading each status, after the banned-name gate. A gate someone must remember to invoke is the defect one level up, the entire content of `[R:signal-needs-a-consequence]`. **And to CI, which the banned-name gate could not have** — this check needs no machine-local list, so both OSes run it and a fork is not failed for something that is none of its business.
  - [ ] **The structural fix, which is the real answer.** A scaffolding crate that already contains `Money<C>`, `Price` and `Qty` with an exact representation and **no `Div`**, so a greenfield financial project begins with the types present and the model *selects* rather than *invents*. This is the only item here that closes the class: it moves the guarantee from layer 4 to layer 1, and it is the only one that works when the type layer is what is being authored. The scan is the interim; this is the fix.
- [x] **Two rules written and five statuses repaired (2026-09-19), corpus 84 → 86.** `[R:one-home-per-rule]` — the framework's own P2, cited by tag in six places and defined in none — and `[R:review-against-contract-not-plan]`, from the 2026-07-16 mesh-watchdog build. Candidates were taken from the 2026-08-16 untagged inventory and each one checked against the live corpus first; most had been absorbed by the migration and were not written again.
  - **Five statuses were understating their own controls.** `async-all-the-way`, `justify-every-clone`, `private-fields-only` and `definition-of-done-every-change` read `active` — prose alone holds this — while a wired, blocking PreToolUse hook already enforced them; `no-unwrap-in-production` named one of the two hooks holding it. The 2026-10-03 review would have read four rules as unheld and considered strengthening what is already at Layer 1.
  - **`partial`, never `graduated`.** No hook claims its rule's whole class — `no-pub-fields` gives a private field and not the constructor that enforces the invariant, `no-block-on-in-async` misses a `std::sync` lock held across an await, and all three see only Rust under `src/`. A false `graduated` prints "Also enforced by X" into five layers and arms a CI **error** against a recurrence in the part the hook never held.
  - **A third rule was recommended and then withdrawn**, which is the more useful half. Reading `hooks/tdd-gate.sh` showed it enforces *the enforcing artefact ships with the code* — check 1 of `[R:definition-of-done-every-change]` — not test-first, which would have contradicted the standing `rust-typedd` override that types precede tests in all Rust work. It became a status repair rather than a second home for half an existing rule.
  - **Two advisory hooks were deliberately not counted**: `fan-out-advisory` and `session-wiring-check` warn and do not block, so their rules stay `active`. Promoting on the strength of a signal that gates nothing is the error `[R:signal-needs-a-consequence]` names.
  - [ ] **Still untagged: the eleven rules-monitor doctrine imperatives** — `citations-gate-nothing` (six untagged copies, the most duplicated unnamed rule in the library), `citation-needs-counterfactual`, `no-teaching-to-the-test`, `bare-arm-must-be-uncontaminated`, `challenge-only-as-a-gate`, `monitor-is-on-its-own-cut-list`, `never-graduate-into-false-confidence`, `automation-attributes-itself`, `graduation-is-per-surface` and two more. Home is `project=stochos-lab`. Deliberately not written piecemeal: `citations-gate-nothing` alone spans six copies, and porting them one at a time creates the drift they are about.
  - [ ] **Still untagged: the compliance rules** homed in `skills/second-brain`, a vault skill that loads only when the vault does — `repo-private-by-default`, `no-employer-in-public-artifacts` (single copy, highest loss severity in the inventory), `no-public-ai-attribution`. The *mechanics* are covered by `[R:names-travel-with-the-quote]` and `[R:report-the-hit-not-the-match]`; the policy itself is untagged and in the wrong home.
- [x] **`[R:signal-needs-a-consequence]` written (2026-09-19), corpus 83 → 84.** The class the rejection below identified: a check that is correct, whose verdict arrives intact and whose wiring is live, that gates nothing — read once and then invisible while the condition it reports degrades. Fourth position in the family with `[R:guarantee-needs-a-reader]`, `[R:wired-artifact]`, `[R:verdict-survives-the-channel]` and `[R:reconcile-wiring-at-start]`, and the one where every link holds. Authored with `relearn new`, `global` / `mined` / `active`. The sharpest half is in the incident rather than the title: a gate that *would* have failed on the same drift already existed and was not run either, because running it was also something a person had to remember — so the consequence has to attach to an event that must happen anyway, not to a command someone must choose to invoke.
- [ ] **The graduation this rule asks for, and it lives in stochos-lab.** `status = active` — prose alone holds it, recorded in FEATURES.md as `NOTHING YET — exposed`. The shape that would hold it: a check asserting that every hook or script which emits a warning either exits non-zero or is named in a reviewed register, so an advisory has to be *declared* advisory rather than becoming one by default. Not built, and deliberately not stubbed here — the signals are in stochos-lab, this repository has no hooks, and a control written where the thing it controls is absent is the `[R:wired-artifact]` failure.   - [x] **The concrete first instance shipped the same day, in stochos-lab.** `.githooks/pre-push` blocks a push while the generated layer on that machine disagrees with this library, and section 8 of `verify-deploy.sh` was factored into `scripts/verify-relearn-layers.sh` so the hook runs the same check rather than a second copy. The two callers differ only in policy on *cannot check* — a note inside the wider audit, a block at push time, since a gate that was never armed must not be able to report clean. Probed on all four paths before being trusted, and `verify-deploy` now fails on an unset `core.hooksPath`, because git cannot make a hook mandatory and the honest move is to report the hole rather than hide it.
  - **The general check is what stays open here**, and it is the bigger half: this closed one signal, not the class.
- [x] **A second recurrence on `[R:guarantee-needs-a-reader]` (2026-09-19).** stochos-lab's `verify-deploy.sh` printed `ok  Cowork export matches repo sources` while the check behind it hashed one file per skill, `SKILL.md`, and the bundle it certifies archives the whole directory — `evals/`, `references/`, `assets/`. An eval case was added to `skills/relearn/evals/evals.json`, the bundle was not rebuilt, and the gate called the export fresh. Recorded as a `[[recurrence]]` with its own date rather than by editing `incident`; `lint` now reads 2, and the truncation rank moved the tag to the **front** of `global`'s emitted description, which is the 2026-09-18 rank working on live data rather than on a fixture.
  - **It is the narrow-check-wide-language half of the rule, not the no-check-at-all half.** Something did read the world; the sentence describing what it read was wider than the read. That distinction is already in the rule body (second paragraph) and in the `status = partial` field, whose `uncovered` reads *"any guarantee stated outside FEATURES.md and the pack READMEs"* — the Cowork claim lives in another repository's gate output, so the status predicted this one correctly and needs no widening. Fixed upstream by `scripts/skill-hash-lib.sh`, one definition of a skill's hash shared by the writer of the manifest and its checker, probed both ways.
  - **A second candidate was rejected, and the rejection is the more useful record.** The same session's stale-binary incident — a `SessionStart` hook printing its warning correctly for five days with nobody reading it — was *proposed* as a recurrence here and does not belong: nothing failed to read the state, a human failed to read the output. `[R:wired-artifact]` is a check accepting forgeable evidence and `[R:verdict-survives-the-channel]` is a verdict destroyed in transit; neither is *a correct verdict, correctly delivered, that gates nothing*. **No rule in this corpus covers that class**, which makes it a step-3 missing rule rather than a recurrence, and forcing it onto the nearest neighbour would have inflated the one number the framework treats as evidence. Carried as George's call: it is his machine's hook and his cost to trade a warning against a failure.
- [x] **The ledger caught up with the description fix (2026-09-19).** Closing the item above left four stale statements in `FEATURES.md`, the one file whose whole job is to be true: the skill-emitter row still said a colon reaches the field through a rule *title*; the bounded-trigger row still said the list is of titles; the truncation row still carried a live `NOTHING YET — exposed` for an exposure closed hours earlier, so the ledger was overstating a gap rather than understating one — the rarer direction and the one that makes every other `exposed` row cheaper to ignore; and the emitter row cited `description_is_quoted_so_a_colon_in_a_title_stays_valid_yaml`, renamed to `..._in_a_home_label_...` in that same change and therefore resolving to nothing. `[R:repair-the-lying-artefact]` `[R:doc-currency]`
  - **Why the gate did not see it.** `tests/ledger.rs` resolves **path-qualified** citations — one naming its file and its function — and a bare backticked test name is not one, so a renamed test cited without its path goes dangling silently. Found by sweeping every backticked snake_case identifier in `FEATURES.md` against `fn <name>` across `src` and `tests`: 322 identifiers, four expected misses (a serde attribute, a crate function, a rule-body example), one real. The whole file was swept rather than the section that raised the suspicion `[R:measure-the-claim-not-a-subset]`.
  - **Not closed by widening the gate, and that is a judgement, not an omission.** To resolve bare names it would have to tell a test name from a type, a field and a crate function — the three kinds of expected miss above — and a gate that guesses wrong either fails on a legitimate citation or goes green on a dangling one. Recorded as a known blind spot in the ledger's own coverage; the cheap half of the fix is a habit, which is that a citation names its path.
- [x] **26 rules promoted, corpus 57 → 83, and `domain-java` created (2026-09-19).** 13 Java rules and 13 low-latency (12 written + 1 rehomed). `check` 83, `lint` 7 findings identical to the 57-rule baseline, `verify` green on all three output roots. **Contradiction review at 83, done on landing:** the near pairs are all cross-referenced in the bodies rather than left implicit — `null-is-not-a-value` ↔ `[R:no-sentinel-values]` (the same class in a language where the sentinel is built into every reference type), `seal-the-alternatives` ↔ `[R:seal-closed-trait-sets]` ↔ `[R:make-illegal-states-unrepresentable]`, `no-reference-to-internals-escapes` ↔ `[R:private-fields-only]` (nearly automatic in Rust, only the first half here), `exceptions-name-what-failed` ↔ the Rust error rules, `publish-safely-or-not-at-all` ↔ `[R:verify-ordering-on-the-weakest-target]`. The one real tension is deliberate and stated in the rule that carries it: `[R:a-wrapper-type-is-not-free-here]` contradicts `[R:newtype-liberally]`'s *cost* claim on purpose, because that claim is true in Rust and false in Java, and a reader holding both disciplines needs the difference stated rather than inferred.
- [x] **Two defects caught on landing, both by gates, both mine.** `tests/pack_counts.rs` failed twice: the copilot table still claimed 57 rules against 83, and `every_file_in_a_pack_is_counted_by_its_readme` found the new `domain-java` skill with no line in the claude-pack inventory — a test I did not know existed, doing exactly its job. And `identity-is-not-equality-for-boxes` carried an `incident` reading "it belongs beside the instrument rules in the latency domain", which reads as a homing claim contradicting its own `home = java`; reworded before commit. `[R:repair-the-lying-artefact]`
- [x] **A documentation sample is now checked as a rule (2026-09-19).** `tests/doc_samples.rs` extracts every `+++` block from `README.md`, `ARCHITECTURE.md` and the two worked examples and runs it through `rule::parse_document`. Three tests, because the obvious one is not enough: the samples parse; every listed document still exists and still carries a sample (the list is paths, and a rename would make the gate quieter rather than louder); and the extractor works on CRLF as well as LF, since these documents are not LF-pinned in `.gitattributes` and a `+++\r` line matches no delimiter — the gate would find nothing and pass on the platform it was written on. Plus a count assertion, because a gate that finds nothing passes. Verified by probe: deleting `origin` from the README sample fails it at `README.md:150`. `[R:wired-artifact]` `[R:xplat-fixtures]`
- [x] **`contribute` now scans every authored field (2026-09-19).** `title` and `error_class` joined `published_incident`, `body` and `source` in `Contribution::authored_texts`. They were left out on the ground that widening the scan would newly refuse contributions that pass today, and a gate that refuses previously-legal input gets muted — which traded a real exposure for a hypothetical one. A contribution is a deliberate act behind `--confirm` and an explicit term list, so refusing one is the gate working; nothing about a `title` stops a product name being in it; and scanning four fields while reporting clean was the more expensive mistake, because the report named what had been checked and not what had not. `tag` and `home` stay out, and the reason is recorded on the method rather than left for the next reader to re-derive: a tag is an identifier the corpus publishes everywhere, and a project `home` never reaches a contribution, being withheld by `Home::federation`. `[R:names-travel-with-the-quote]`
- [x] **Decided 2026-09-19.** Correct as it stands. A domain that is not a language has no file extension that means "this code has a latency budget", so it gets no glob and cannot be path-scoped; emitting it always-resident instead would put fifteen latency rules into every session in every repository, which is the cost `Home` exists to control. The description is now a working trigger, so the reachable path is the right one. Original entry: **`domain-low-latency` is reachable only by its description, and that is now a working trigger rather than a broken one.** `LoadSemantics::for_home` maps a domain to globs through `domain_globs`, which knows languages. `low-latency` is not one, gets no globs, and is therefore `OnRequest` — which `claude_rules` refuses to emit at all, confirmed by the absence of `.claude/rules/domain-low-latency.md` in the committed tree. So where `domain-rust` auto-attaches on any `.rs` read, `domain-java` on any `.java`, and `global` is always resident, **every low-latency rule reaches a session only if the skill matcher fires on its `description`.** *Half of this is fixed.* The description used to list sixty-character titles and truncate — `global` carried fifteen of thirty plus `+15 more` — so rules were not merely hard to match, they were absent from the only string a matcher reads. It now lists tag bodies with hyphens as spaces, every home fits with room, and `tests/description_reaches_every_rule.rs` fails the build if any home ever truncates again. What remains is the load model itself: a description that can be matched is still a description that must be matched, where `domain-rust` and `domain-java` need no matcher at all. The options are unchanged — carry trigger vocabulary in the format; accept `OnRequest` and invoke deliberately; or extend `domain_globs` to non-language domains, which stays rejected on sight because no file extension means "this code has a latency budget", the argument already recorded for `Org`. **Nothing decided.**
## Phase A — portability (v0.1)

### Types first (before any emitter)
- [x] `RuleTag` newtype, `parse` only, shape `R:[a-z0-9][a-z0-9-]*`
- [x] `Home` sum type: `Global | Domain{name} | Project{path}` (+ `DomainName`/`ProjectPath` non-empty newtypes)
- [x] `Status` sum type carrying payloads: `Active | Graduated{to} | Attic{reason,date}` (+ `Destination`/`Reason` newtypes)
- [x] `Incident`, `ErrorClass` newtypes (non-empty by construction, over a shared `nonempty` perimeter)
- [x] Date field: parse wide, then range-check (dogfood `[R:parse-wide-then-range-check]`) — `Date::parse` into `i64` then narrow; property-tested
- [x] `Library<Unvalidated>` / `Library<Validated>` typestate; `validate` is the only path to `Validated` (`emit` will accept only the latter)

### Parsing
- [x] TOML front-matter reader (`+++` delimited) → `Rule` (`rule::parse_document`; dates are quoted strings so `Date::parse` keeps the range check)
- [x] Parse failure names the file and field, never skips (`ParseError` + `fsio::LoadError::Parse`)
- [x] Duplicate-tag detection at library level (`Library::validate` → `ValidationError::DuplicateTag`)

### Emitters (pure functions returning `Vec<OutputFile>`)
- [x] Shared emit types: `OutputFile` (path + contents), `RelativePath` (portable forward-slash, `pub(crate)` construction), `HomeSlug` (`Home` → `[a-z0-9-]+`, also the skill `name`) — `emit.rs`
- [x] `emit` shared scope helper: `Home` → (globs, alwaysApply) via a domain→file-pattern table (`rust → **/*.rs`, …); read by `cursor` (and later `copilot`) so scope translation is one place (`emit::Scope::for_home`)
- [x] `claude` — one skill **per home layer**: `skills/<home-slug>/SKILL.md`, description aggregating the home's rules (YAML-quoted); makes the `Library<Validated>` typestate load-bearing (`emit::claude::emit`)
- [x] `cursor` — `.cursor/rules/<tag-body>.mdc`; `globs`/`alwaysApply` from `Scope::for_home`, not the rule; lenient (unquoted) front-matter, colon-free filename via `RuleTag::body` (`emit::cursor::emit`)
- [x] `copilot` — `.github/copilot-instructions.md` (all rules, home-rank ordered; `emit::copilot::emit`)
- [x] `agents` — `AGENTS.md` (all rules, home-rank ordered; `emit::agents::emit`)
- [x] `claude_rules` — project-layer `CLAUDE.md` (project-home rules only; no file when none; `emit::claude_rules::emit`)
- [x] Shared `emit::home_rank` (general→specific home ordering for the concatenated emitters)

### fsio + guards
- [x] Read side: load `rules/*.md` (sorted) into `Library<Unvalidated>`, file-named diagnostics (`fsio::load_rules`)
- [x] Generated-by header + content hash on every emitted file (`fsio::write_all` appends marker + version + source tags + `sha256`)
- [x] Overwrite guard: pre-flight marker check, all-or-nothing abort; refuse to clobber a file lacking the header (`fsio::write_all` → `WriteError::WouldClobberUnversioned`; `[R:generate-guards-unversioned]`)
- [x] Detect a hand-edited generated file (recompute body `sha256`, compare to the header's) — **done via `relearn verify`** (`fsio::verify_all`): classifies each generated file `Ok`/`Missing`/`Unversioned`/`HandEdited`/`Stale` and exits non-zero on drift. Integrity (body-hash vs own header) checked before freshness (vs a fresh emission), so a hand edit is never mislabelled stale. The marker guard *prevents* clobbering; this *detects* drift — complementary. `build`/`verify` share `cli::emit_selected`.
- [x] **Orphan detection in `verify`** — done. `verify` now reports `VerifyStatus::Orphan` for a marker-bearing file in a relearn-owned location that the current rules no longer emit (its rule/home was deleted). Ownership policy: scan only the subtrees relearn emits to (`skills/`, `.cursor/rules/`, and the fixed `AGENTS.md`/`CLAUDE.md`/`.github/copilot-instructions.md`) and flag only marker-bearing files, so a hand-authored file — even inside an owned dir — is never claimed (`fsio::scan_orphans`/`collect_files`; 4 new pins). Verified through the binary: deleting a rule → its `.cursor/rules/*.mdc` reported `orphan`, exit 1.

### CLI
- [x] `check`, `build --targets`, `list --home` (`cli::run` → `check`/`build`/`list`; `CliError` wraps `LoadError`/`ValidationError`/`WriteError`; `main` is a thin `ExitCode` shell)
- [x] Unknown target rejected before any write (`Target` value-enum names only buildable emitters, so an unknown `--targets` value is a `clap` parse error)

### Tests (per `rust-typedd` hierarchy — types first, then properties, then pins)
- [x] proptest: emission idempotent over generated rule sets (`tests/properties.rs::emission_is_idempotent`, all 5 emitters)
- [x] proptest: round-trip preserves tag, home, status, body (`tests/properties.rs::neutral_round_trip_preserves_the_rule`, via the new `rule::to_document` serializer)
- [x] compile-fail test: unvalidated library cannot reach `emit` (`trybuild` pin `tests/compile_fail/emit_rejects_unvalidated_library.rs` — checks the type mismatch, not merely that it fails; **closes Phase A**)
- [x] Standing practice, not a task: one unit pin per bug found, added with the fix. Check 1 of the definition of done says the same thing and is the thing that enforces it. Closed 2026-09-19 as a duplicate of a rule.
- [x] **`#[must_use]` sweep — resolved 2026-08-14, premise corrected.** The 2026-08-13 TDP-scan item claimed the `Result`-returning `parse` constructors (18 sites) "lack `#[must_use]`" and should get it. That premise was **wrong**: `Result<T, E>` is *itself* `#[must_use]`, so a caller who drops one already gets `unused_must_use` from the type — the fn attribute adds nothing, and `clippy::double_must_use` (warn-by-default) fires on it, which the crate's `-D warnings` gate rejects (verified empirically in a scratch crate). So the 18 `Result`-returning sites correctly stay **un-annotated** — do not "fix" them. The genuine gap was 4 `pub(crate)` *value*-returning helpers in `emit.rs` (`RelativePath::{from_segments,from_forward_slash}`, `OutputFile::new`, `yaml_double_quote`) that clippy's `must_use_candidate` doesn't flag (non-exported) but the sibling `pub(crate)` fns already carried; now annotated. Public API was already clean (`must_use_candidate` = 0). Also fixed a pre-existing rustfmt drift in `fsio.rs` orphan tests that had slipped into commit `e6244da`.

### Seed content
- [x] Migrate a first real rule set into the neutral format (`rules/*.md`) — **the first real test of format adequacy**. The schema (tag / title / error_class / home / created / status / incident / body) held every rule with no missing field.
- [x] Grow the corpus to **10 rules** (added `parse-dont-validate`, `no-unwrap-in-production`, `no-anyhow-in-libraries`, `make-illegal-states-unrepresentable`). Now exercises: `graduated` status on real data (`no-unwrap`/`no-anyhow` graduated to the actual `no-unwrap-in-src`/`no-anyhow-in-lib` hooks); multiple rules per non-global home (4 in domain-rust); and real cross-references (`parse-dont-validate`→`parse-wide`, `illegal-states`→`no-sentinel-values`, both resolving to active — the reference checker runs clean on real data). Homes: 5 global, 4 domain-rust, 1 project-relearn. `check`/`lint`/`build` all green (16 emitted files); contradiction re-review clean.
- [x] Grow the corpus to **14 rules** (added `prefer-by-construction`, `generate-guards-unversioned`, `verify-tracked-after-move`, `no-secrets-in-config-repo`). All four carry `[R:...]` tags that already exist in George's live instruction corpus — `prefer-by-construction` (global) and the two stochos-lab tags from that repo's CLAUDE.md, plus `generate-guards-unversioned` which relearn's own FEATURES/TODO already cite. Adds a **second project home** (`project-stochos-lab`) and one resolving cross-reference (`prefer-by-construction`→`make-illegal-states-unrepresentable`). Homes: 6 global, 4 domain-rust, 2 project-relearn, 2 project-stochos-lab. `check`/`lint`/`build`/`verify` all green (21 emitted files); contradiction re-review clean.
- [x] Grow the corpus to **15 rules** (added `source-practice-from-its-artefact`, global — drafted in a Cowork session, landed here under the tool's own discipline 2026-08-20). `check` 15 validated; `lint` no findings (the deliberately-untagged prose reference to the sibling *search-before-you-build* rule is not flagged — the `[R:doc-currency]` precedent; no home-slug collision, no overlapping scope); `build --targets claude,cursor,copilot,agents` wrote 21 files and the rule reaches all four layers; `verify --targets claude,cursor,copilot,agents` → 21 up to date. **Contradiction re-review 2026-08-20 (corpus 15; previous pass 2026-08-13 at 14): clean.** Contradicts nothing, duplicates nothing; *relates* to `[R:prefer-by-construction]` (source-correctness up front) and to `[R:verify-through-production-path]` (same authoritative-artefact-over-plausible-proxy family — the *describing* half to that one's *verifying* half), and names in prose the unported *search-before-you-build* sibling (building half vs describing half). Homes consistent. ~~NB: default `verify` (5 targets) flags `CLAUDE.md unversioned` — a **pre-existing structural collision** between the `claude-rules` target and relearn's own hand-written `CLAUDE.md`, not caused by this rule; the four-target build/verify is the operative one.~~ **Fixed 2026-08-22** — and the note above was the defect, not the record of it: it wrote a workaround down as a fact ("the four-target build/verify is the operative one") and left the default invocation broken, which is exactly how a control degrades into decoration. Bare `relearn verify` now exits 0 on a clean tree; 23 files, all five targets.
- [x] Grow the corpus to **19 rules** — the `rust-typedd` port, 2026-08-22. Added `newtype-liberally`, `typestate-for-protocols`, `design-types-first` (all `domain-rust`) and `no-weak-model-for-judgment` (`global`). All four carry **codification-dated** provenance, not single-incident: each `incident` names the real source in George's live instruction layer (`~/.claude/skills/rust-typedd/SKILL.md`, revised 2026-07-22, and Patterns 1/3 of `~/.claude/CLAUDE.md`) and the date it was codified here. **No incident was fabricated.** The remaining rust-typedd content — the standing override, the four-tier hierarchy of controls, *What unit tests are still for*, *Interaction with other skills* — is **not rule-shaped** and was deliberately not ported; it is the framing that makes the rules interpretable and the neutral format has nowhere to put it. That is the open format question above. `check` 19 validated, `lint` no findings, `build` 27 files, bare `verify` 27 up to date — all exit 0.
- [x] Grow the corpus to **21 rules** — `[R:xplat-fixtures]` and `[R:pin-eol-for-executable-text]` ported 2026-08-22, closing the last two tagged rules that lived only in `~/.claude`. Both carry **single-incident** provenance, not codification-dated: xplat-fixtures names the three Ferridis tests green on Linux and broken on Windows (2026-07-20) and its 2026-08-16 recurrence in `verify-wiring.sh`; pin-eol names the stochos-lab clone whose CRLF shebangs killed the hook layer (2026-07-22) and its recurrence in this repo when committing the emitted tree made line endings load-bearing for `verify`. **No incident was fabricated** — both were already tagged and dated in `usage/rules-ledger.md`. pin-eol was overdue in particular: this repository's own `.gitattributes`, `ARCHITECTURE.md`, `CLAUDE.md` and `TODO.md` all cite the tag, so the corpus was citing a rule it did not define. They are siblings and each body names the other (at rest vs in flight), which `lint` now resolves instead of reporting dangling.
  - **Drift check on `no-weak-model-for-judgment`:** the brief asked whether `measure-cost-per-task` already covers it. It does not, and the two are cleanly separable — that rule is *economic* (choose between capable models by measured cost, never by price tier), this one is a *capability floor* that holds even when the cheap option genuinely is cheaper per task. Each rule body names the other and states the distinction, so neither can be read as a restatement. `lint` reports no overlapping scope, correctly: the error classes differ.
  - **Finding about the linter, from the same check.** `lint` could not have flagged the two-homes problem this port fixes, and cannot flag one like it in future: `lint::lint` takes `&Library<Validated>` and sees only `rules/`. A rule duplicated into a hand-authored skill *outside* the library is structurally invisible to it. The four rules that were stated both in `rules/` and in `rust-typedd` were found by reading, not by the tool whose job this is. Recorded as a gap in FEATURES.md rather than closed here — the fix needs a decision about whether `lint` may read anything outside the rule library at all, which cuts against the purity invariant.
- [x] Grow the corpus to **22 rules** — `[R:repair-the-lying-artefact]` (global), landed 2026-08-24 from a live incident, not a port. **Single-incident provenance, and the incident is a recurrence**: Design-Architecture-Tool's `build.sh` printed `Binary: $ROOT/target/release/dat-designer` while `CARGO_TARGET_DIR` sent cargo elsewhere, so a five-month-old pre-rename designer binary was launched twice (2026-08-16, 2026-08-24) and read as "my fixes are missing". The first occurrence was closed by writing a CLAUDE.md paragraph *beside* the script that went on printing the false path — which is the rule. Enforcing artifact from day one: that repo's `scripts/verify.sh` check 8 `build_path_resolution()`, probed in both directions against the pre-fix `build.sh`. **Contradiction re-review 2026-08-24 (corpus 22; previous pass 2026-08-22 at 19 — corrected 2026-08-24: no pass was ever run at 21, so `xplat-fixtures` and `pin-eol-for-executable-text` are reviewed here for the first time): clean.** Contradicts nothing; the near neighbours are all separable — `[R:verify-through-production-path]` is the *verifying* half to this one's *reporting* half (exercise the real channel vs. that channel must tell the truth about what it produced); `[R:prefer-by-construction]` supplies the preferred fix shape (derive the claim, don't assert it) but is not about false claims; `[R:doc-currency]` is the closest and is **complementary, not overlapping** — it says update every checked-in description when reality moves, this one says the description is not where the fix lands when an executable artefact is the thing doing the misleading. `check` 22 validated, `lint` no findings (no overlapping scope reported against doc-currency), `build` 31 files, bare `verify` 31 up to date — all exit 0. Body cites three tags, all resolving to active rules. **Ledger follow-up 2026-08-24:** that change updated TODO.md but not FEATURES.md — check 3 of the definition of done, skipped. Closed now: the `claude-rules` row's verified `build`/`verify` counts moved 30 → 31 and are dated so they cannot silently go stale again, and the contradiction-detection row carries this pass (it had stood at corpus 19 through two corpus growths).
- [x] **Contradiction re-review 2026-08-22 (corpus 19; previous pass 2026-08-20 at 15): clean.** No contradictions, no problematic overlap, homes consistent under the homing principle settled below. Four relationships noted, none a contradiction: `newtype-liberally` ↔ `parse-dont-validate` (where the witness comes from vs carrying it afterwards, cross-referenced both ways); `typestate-for-protocols` ↔ `make-illegal-states-unrepresentable` (the same remedy applied to time rather than structure, cited); `design-types-first` ↔ `prefer-by-construction` (both the hierarchy-of-controls family, pushing a guarantee to the strongest layer that holds it); `no-weak-model-for-judgment` ↔ `measure-cost-per-task` (capability floor vs economics — distinguished in both bodies, see above).
- [x] **Homing principle settled 2026-08-22** — the 2026-08-13 review parked this pending recurrence and it recurred. The question "does the principle generalise beyond its home?" was the wrong one: *all four* of `parse-wide-then-range-check`, `parse-dont-validate`, `make-illegal-states-unrepresentable` and `no-sentinel-values` generalise, so generality cannot discriminate and the apparent inconsistency was an artefact of asking it. **`Home` is a scoping decision, not a taxonomy of generality** — it drives `alwaysApply`/globs (ARCHITECTURE, 2026-08-13), so every promotion to `global` spends always-resident context (P6). The principle: *a rule is homed in the narrowest scope that still catches every occasion its error class actually arises in this corpus; the breadth of the underlying idea is not the criterion.* Under it the existing homes are all correct and unchanged — `parse-wide`/`parse-dont-validate` stay `domain-rust` (every recorded incident is Rust and the remedy is stated in Rust vocabulary), `no-sentinel-values`/`make-illegal-states-unrepresentable` stay `global` (language-agnostic bodies, error class arises anywhere). The three new Rust rules follow it into `domain-rust`; `no-weak-model-for-judgment` is `global` because nothing about it is language-scoped.
- [x] Grow the corpus to **32 rules** — the ten remaining Rust patterns, ported 2026-08-24. Added `must-use-on-consequential-returns`, `private-fields-only`, `seal-closed-trait-sets`, `typestate-builder-for-required-fields`, `errors-name-what-failed`, `justify-every-clone`, `borrow-in-signatures`, `async-all-the-way`, `module-visibility-is-deliberate`, `verify-the-abstraction-compiled-away` (all `domain-rust`; the layer goes 8 → 18). All ten carry **codification-dated** provenance naming Patterns 4/6/7/8/9 and the General Coding Rules of `~/.claude/CLAUDE.md` as the source, plus the audit that surfaced them. **No incident was fabricated.** *How they surfaced:* the emitted Copilot artefact was read against the CLAUDE.md pattern list and ten practices had no rule here at all — so they reached Claude through the always-on boot index and reached Cursor, Copilot and `AGENTS.md` not at all. That is the vendor lock this tool exists to remove, occurring in its own corpus, and it was invisible to `lint` for the same structural reason recorded below: the linter sees `rules/`, never the layer a rule ought to have been ported *from*.
  - **Contradiction re-review 2026-08-24 (corpus 32; previous pass 2026-08-24 at 22): clean.** No contradictions; three relationships needed a body edit rather than a verdict, and got one in the same change. (i) `verify-the-abstraction-compiled-away` **contradicted `newtype-liberally` as written** — that rule asserted flatly that "newtypes are zero-cost", which is the exact unverified-claim shape the new rule bans. Resolved by repairing the artefact making the claim, not by noting it elsewhere: `newtype-liberally` now says *ordinarily* zero-cost and cites the new rule for the load-bearing case. `[R:repair-the-lying-artefact]` `[R:doc-currency]` (ii) `module-visibility-is-deliberate` ↔ `private-fields-only` are the same argument at item and field level; both now say so and state why satisfying one implies nothing about the other. (iii) `justify-every-clone` ↔ `borrow-in-signatures` are cause and symptom; cross-referenced both ways. Separable pairs needing no edit: `typestate-builder-for-required-fields` ↔ `typestate-for-protocols` (construction vs sequence, already cited); `seal-closed-trait-sets` ↔ `make-illegal-states-unrepresentable` (a closed set is one illegal state among many); `must-use-on-consequential-returns` ↔ `errors-name-what-failed` (discarding an outcome vs what the outcome says). `errors-name-what-failed` names the anyhow ban **in prose, not by tag** — that rule is graduated, and citing a retired rule is an `Info` finding that would fail `lint`; the `[R:doc-currency]` precedent.
  - `check` 32 validated, `lint` no findings, `build` 41 files, bare `verify` 41 up to date, `verify --targets copilot --out copilot-pack` 1 up to date — all exit 0.
- [x] Grow the corpus to **44 rules** — the repository discipline and its controls, ported 2026-08-25. `global` 10 → 22. **How it surfaced: George read the Copilot pack and could not find the four-file discipline in it.** He was right — zero of the 32 rules covered it. The library carried the type discipline and the verification discipline in full while carrying nothing about the repository discipline that *Aiming the Stochastic Machine* is about and that this repository is built on. Ported from `~/.claude/skills/project-discipline/SKILL.md`, the artefact that defines the practice, per `[R:source-practice-from-its-artefact]` — not from memory of the genre.
  - **Five new rules for the discipline itself:** `five-files-no-more`, `claude-md-recreates-the-project`, `decisions-log-records-rejected-alternatives`, `features-ledger-names-its-artefact`, `definition-of-done-every-change`. Codification-dated provenance; `features-ledger-names-its-artefact` anchors on the real ha-mcp v0.7.0 incident of 2026-07-16 (an IP allowlist that passed every smoke gate through a dev override while the production wiring failed open).
  - **Seven already-tagged controls that had never been ported:** `wired-artifact`, `revision-integrity`, `guarantee-needs-a-reader`, `detector-excludes-own-definitions`, `no-stale-push-over-fresh`, `reconcile-wiring-at-start`, and `case-collision` (landed `graduated` to `hook:no-case-collision`, its real status). Every one carries its genuine dated incident from the skill — Ferridis 2026-07-20, the v0.3 paper restructure 2026-08-11, the four-in-two-days sweep, the nine dark PreToolUse hooks of 2026-08-16. **No incident was fabricated.**
  - **The charter's own promise came due.** `CLAUDE.md`'s *Standing rules referenced by this repo* restated six rules in prose and said they were "scheduled for migration into `rules/` in Phase A, at which point this section points there instead." `wired-artifact` and `revision-integrity` were the last two outstanding, so the corpus had been **citing two rules it did not define**. Both are now in `rules/` and the section has been collapsed to the pointer it promised. It also carried `pin-eol-for-executable-text` **twice** — a duplicated bullet, which is `[R:revision-integrity]` failing inside the file that defined `[R:revision-integrity]`.
  - **Contradiction re-review 2026-08-25 (corpus 44; previous pass 2026-08-24 at 32): clean.** The close pairs were all separable and each body states the separation. `wired-artifact` ↔ `guarantee-needs-a-reader`: a check that accepts forgeable evidence vs no check at all — the distinction the source skill states in its own family table, preserved here. `wired-artifact` ↔ `verify-through-production-path`: the artefact-level and channel-level halves of one family, already cross-referenced. `features-ledger-names-its-artefact` ↔ `guarantee-needs-a-reader`: the ledger row is the specific artefact, the guarantee rule is the general claim; both name the other. `no-stale-push-over-fresh` ↔ `generate-guards-unversioned`: destroying the newer of two copies vs destroying the only copy. `revision-integrity` ↔ `doc-currency`: internal consistency after an edit vs external currency after reality moves — the pair `doc-currency`'s own body has named in prose since 2026-08-16, and which now finally resolves to a real tag instead of dangling.
  - `check` 44 validated, `lint` no findings, `build` 53 files, bare `verify` 53 up to date, pack `verify` 1 up to date — all exit 0.
- [x] Grow the corpus to **45 rules** — `[R:no-silent-spend]` (`global` 22 → 23), landed 2026-08-31 from a live incident, not a port. Nine tagged releases of Design-Architecture-Tool in a single session, in which every trade of George's time against his tokens against thoroughness was resolved unilaterally and always in the same direction, toward thoroughness: ~10 compile failures walked one at a time *after* the build had already enumerated them, a five-file doc set per release, forty-line commit messages, the full gate re-run where the fast path would have served. None of the work was wrong; all of it was his money spent without him. **His own diagnosis is the rule** — *"it is visible, but at the same time I dont have the option to choose a higher cost or not based on whats worth it. Remember me and you are a compound not a mixture."*
  - **Its Claude-Code sibling was deliberately NOT ported.** `[R:delegate-a-fan-out]` (`~/.claude/rules/ecc/common/agents.md`) is the delegation instance of the same class, and it names a subagent mechanism that Copilot, Cursor and AGENTS.md readers do not have — porting it would put dead vocabulary in every non-Claude layer. Kept narrow there on George's explicit call so the two carry separate metrics; this rule carries the general class and is the one that travels.
  - **Contradiction re-review 2026-08-31 (corpus 45; previous pass 2026-08-25 at 44): clean.** The near neighbours are separable: `[R:measure-cost-per-task]` governs picking a *mechanism* by measured cost rather than by price tier, this governs who *makes* the choice at all; `[R:prefer-by-construction]` supplies a preferred fix shape, not a subject; `[R:source-practice-from-its-artefact]` shares the go-to-the-real-source flavour but is about sourcing a *description*, not spending a *resource*. Body is target-neutral by construction — no subagent or harness vocabulary — because it emits to four non-Claude layers.
  - `check` 45 validated, `lint` no findings, `build` 54 files, bare `verify` 54 up to date, `verify --targets copilot --out copilot-pack` 1 up to date — all exit 0. `copilot-pack/README.md` figures updated by hand to 45 rules / 505 lines / `global` 23, which is the open row below still doing exactly what it says it does.
- [x] Grow the corpus to **46 rules** — `[R:verdict-survives-the-channel]` (`global` 23 → 24), landed 2026-09-03 from a live incident, not a port. In one session on Design-Architecture-Tool the project's single verification gate was run roughly fifty times and **not once unfiltered** — every invocation was `sh scripts/verify.sh 2>&1 | tail -N` or piped into `grep`. One run printed `[verify] BLOCKED — failing gates:` with five gates named beneath it and came back as `[exited with code 0]`, because a pipeline's status is its last stage's; `tail -3` had also cropped the BLOCKED header off the top, leaving three gate names that read like ordinary progress. Twice the shape was `verify.sh 2>&1 | tail -3 && git add -A`, where the `&&` tests the filter, so staging proceeded whatever the gate had said. **The gate was never wrong and was never unenforced** — its answer simply did not survive the trip to the reader.
  - **The mirror image, same session:** a `perl -pe 's/\Q...\E/.../'` whose `\Q...\E` inside a single-quoted shell argument searched for literal backslashes, matched nothing, exited 0 and reported success — twice, costing two CHANGELOG entries. An exit code reports that the program ran, not that the work happened, and that is one rule with the pipeline case, not two.
  - **Third position in an existing family, which is why it is a rule and not a duplicate.** `[R:guarantee-needs-a-reader]` fires when no check exists; `[R:wired-artifact]` when one exists and accepts forgeable evidence; this one when a correct check's verdict is destroyed in transit. Nothing forged, nothing unenforced. Both neighbours' bodies already name each other, and this body names both.
  - **Contradiction re-review 2026-09-03 (corpus 46; previous pass 2026-08-31 at 45): clean.** The near neighbours separate cleanly on the account above; `[R:verify-through-production-path]` is the remaining close one and asks a different question — *which channel did you test through*, versus *did the answer reach you intact*. `[R:repair-the-lying-artefact]` is about an artefact that states something false, and `verify.sh` states the truth. Body is target-neutral: shell vocabulary only, no assistant- or harness-specific terms, because it emits to Copilot, Cursor and AGENTS.md as well as the skill layer.
  - `check` 46 validated, `lint` no findings, `build` 55 files, bare `verify` 55 up to date, `verify --targets copilot --out copilot-pack` 1 up to date, `cargo test` 157 passed — all exit 0, each run unfiltered and its own status read, which is the rule being applied to its own landing. `copilot-pack/README.md` figures updated by hand to 46 rules / 538 lines / `global` 24 — the open row below, behaving exactly as it says it will.
  - **Enforced, not only stated:** `~/.claude/hooks/gate-verdict-intact.sh` (stochos-lab) refuses a Bash command that pipes a gate-shaped invocation without `pipefail` or `PIPESTATUS`. Landed in the same change, in the other repository. `[R:guarantee-needs-a-reader]`
- [x] Recorded provenance, not open work (closed 2026-09-19): the first 6 rules are single-incident-dated; later batches mix single-incident with **codification-dated** provenance. This latest batch ports **already-tagged real rules** from George's live instruction layer (the tag string is not invented — it exists in his corpus); the `created` date is the codification-into-this-corpus date (2026-08-13), with the genuine source named in each `incident`. No fabricated incidents, no invented tags.
- [!] Still no real **attic** rule in the corpus (only `graduated`) — the retired-reference *attic* path stays unit-test-only until a rule is genuinely retired.
- [x] Port the remaining `[R:...]` rules as their incidents are on hand — **the project-discipline layer is done as of 2026-08-25** (twelve rules; see the corpus-44 entry above). Every tag the `project-discipline` skill states now has a rule here. What remains unported is not rule-shaped: the skill's *Why* framing and its family table, which are interpretation rather than instruction, the same category as the `rust-typedd` framing.
- [x] **Decided 2026-09-19.** The charter keeps its copy. The generic rule states the five checks; the charter's version names `FEATURES.md`, its enforcing artefacts and `ARCHITECTURE.md`'s decisions log, which the generic rule cannot. A contributor reading the charter must be told what to actually do, and `rules/` stays the source where the two overlap. Original entry: **The charter now restates one rule the library holds — deliberate, and George's call whether it stays.** `CLAUDE.md`'s *Definition of done* section states the five checks in full, and `[R:definition-of-done-every-change]` now states the general form in `rules/`. The original justification for writing it out (an external contributor cannot load the author's private skill) has partly expired, because the generic rule is now public in this repository too. The remaining justification is **specialisation**: the charter's version names `FEATURES.md`, its enforcing artifacts, and `ARCHITECTURE.md`'s decisions log, which the generic rule cannot. Options: leave it and mark the charter text as derived (done — the footnote now says `rules/` is the source); reduce the charter to a pointer plus only the repo-specific clauses; or add a mechanism for a rule to carry a project-layer specialisation, which is a format change and should not be undertaken for one instance. **Nothing decided; no change beyond the footnote.**

### Open design question surfaced by the graduated rules
- [x] Should `emit` **filter by status**? **Decided 2026-08-13 (George): emit `active` + `graduated`, suppress `attic`; graduated is emitted *annotated* with its destination.** Rationale: the instruction layer *tunes* generation before the fact, the graduated-to hook *catches* after — distinct controls, both raising first-time-right, so a graduated rule keeps its tuning job (annotated so a reader knows a stronger control also holds it); a retired rule must never leak into an active instruction file. Implemented as `Status::emittability` (exhaustive match → a new status must decide its policy) applied once in `emit::emittable`, with `emit::graduation_note` for the annotation. Property-tested whole-space (`atticked_rules_never_leak_into_any_emitter`, `active_and_graduated_rules_all_reach_copilot`) + unit pins; FEATURES row "Emission respects rule status". Verified through the binary: the two seed graduated rules render `> Also enforced by hook:…` in all four target formats. (Attic *suppression* is proven by property + unit, not the binary — no honest attic rule exists in the corpus to feed it, and fabricating one is out.)

## Phase B — linter (functionally complete; `lint` module + `relearn lint` + `relearn verify`, advisory/read-only)
*Only open item is cold-surface, which is blocked on Phase-C runtime data — relocated to Phase C below where its data lives.*
- [x] Overlapping-scope detection (same error class, case-insensitive) — `lint::overlapping_scope`
- [x] Home-slug collision detection: two *distinct* homes whose `HomeSlug` collides would silently share one skill file — flagged `Error`-severity (not merged, not deleted). `lint::home_slug_collisions`. Closes the 2026-08-13 TDP-scan edge.
- [x] Dangling references (rule cites an `R:...` tag absent from the library; `OR:`/`FOR:` in prose excluded; a tag cited in body **and** incident is one finding) — `lint::reference_checks`
- [x] Retired references (rule cites a rule that exists but is atticked/graduated) — flagged `Info`; `lint::reference_checks`
- [x] `relearn lint` CLI: advisory, writes nothing; exit non-zero only on `Warning`/`Error` (`Info` informs without failing CI); clean on the seed
- [x] Contradiction detection — **resolved as a process-control**, not a code check: a periodic Claude review pass over the corpus (semantic judgment, delegated per doctrine; a keyword heuristic would be dishonest). **First review 2026-08-13: no contradictions among the six seed rules, no problematic overlap, homes consistent. One scoping observation — `parse-wide` (domain rust) and `order-by-explicit-rank` (project relearn) express principles that generalize beyond their homes; promote only if the class recurs elsewhere.** Re-run each time the corpus changes materially.
- [x] `relearn verify` — read-only drift detection: each generated file classified `Ok`/`Missing`/`Unversioned`/`HandEdited`/`Stale`, non-zero exit on any drift (`cli::verify` → `fsio::verify_all`). The CI "committed generated tree is in sync with the rules" gate; complements the write-side clobber guard. (Was the Phase A hand-edit-detection item; landed here as it is a checking feature.)
- [→] Cold-surface / uncited report — **relocated to Phase C** (see below): "cold" needs runtime invocation data (stochos-lab observability), which does not live in this repo; "uncited" alone is noise (a standalone rule is legitimately uncited). Not buildable here without faking a signal.

## Phase D — public release (originally gated on an arXiv ID; that gate is void — see Phase E)
- [x] README public framing: versioned instruction artifacts + the governance loop (Why/Background + Status link the worked example; the loop — persist-or-perish, one-home-per-rule — is stated).
- [x] Licence (Apache-2.0 — includes the patent grant that matters for enterprise adoption). `LICENSE` file added; `Cargo.toml` already declares `license = "Apache-2.0"`.
- [x] Worked example: one incident → one rule → five emitted formats (`docs/worked-example.md`) — **generated from real `relearn build` output**, not a mock-up, with the regeneration command recorded so it can't silently drift (honours `R:revision-integrity`).
- [x] **Repo made public 2026-08-19 on George's explicit go.** Per the private-by-default rule this required an explicit instruction — given. Note: the two gate conditions were (a) the real arXiv ID exists and (b) George's explicit go; George deliberately waived (a) and gave (b), taking the repo public ahead of the paper (arXiv not yet visible). Release language de-contradicted first (README/CLAUDE no longer say "not yet released").
- [x] **arXiv identifiers added 2026-08-19.** Both live and verified against arXiv: Paper 1 *Tuning the Stochastic Machine* = arXiv:2608.19125 (the discipline this implements); Paper 2 *Grouping the Stochastic Machine* = arXiv:2608.19140 (references this repo). Threaded into README (Status + Background) and `CLAUDE.md`; Paper 2's repo reference now resolves against a public repo.

## Phase E — alignment with Paper 3 (2026-08-22)

*Triggered by the submission of* Aiming the Stochastic Machine *on 2026-08-21, which uses this repository as its worked case and reports two of its defects as open.*

- [x] **The flagship gate is invoked.** `relearn verify` was named in FEATURES.md as "the CI gate" while CI ran only `fmt`/`clippy`/`test`, and every emitted path was untracked, so a clean clone had nothing for it to verify. Both closed: the emitted tree is committed (23 files) and `ci.yml` runs `cargo run --quiet -- verify` **bare** on `ubuntu-latest` and `windows-latest`. The FEATURES row now carries an **Invocation** paragraph naming the workflow step — Paper 3 §9's proposed extension ("record the invocation, and require that the invocation recorded is the one CI runs"), applied to the repository the paper uses as its case.
- [x] **The `claude-rules` collision is fixed by construction.** The project layer emits to `.claude/rules/<home-slug>.md`, one file per project home, path derived from `Home` via `HomeSlug` — so no emitter can name the hand-authored root `CLAUDE.md`. Locked by a whole-space property test over all five emitters, not only a pin. Bare `verify` exits 0.
- [x] **Home leakage in the project layer** — found while fixing the above and not previously recorded anywhere: `emit::claude_rules` filtered `Home::Project { .. }` without distinguishing *which* project, so this repo's project layer carried stochos-lab's rules (a P2 violation in the emission layer). Fixed by the same change; property-tested.
- [x] **EOL pinned for the emitted tree** (`.gitattributes`, `text eol=lf`). Committing generated files makes line endings load-bearing, because `verify` hashes the body: with `core.autocrlf=true` a Windows checkout would rewrite LF to CRLF and every file would report `HandEdited` — green on Linux, red on Windows. `[R:pin-eol-for-executable-text]`
- [x] **Paper 3 named everywhere Papers 1 and 2 are** — README (Status + Background), `CLAUDE.md`, this file. ~~With the identifier as a single literal placeholder token so one grep would find every site.~~ **Superseded 2026-08-24:** the placeholder is gone and every site now carries the Zenodo DOI — see the Phase E entry below. The token itself is deliberately not reproduced anywhere in this repository any more, including in this record: a correction note that repeats the exact string a future sweep greps for becomes a permanent false positive in its own detector.
- [x] **arXiv declined Paper 3; every site now carries a Zenodo DOI (2026-08-24).** The paper and its evidence were deposited to Zenodo instead, both published 2026-08-24: the paper at version DOI `10.5281/zenodo.22083202` (concept `…201`), the study materials — pre-registration, blinded pack, both codings, `kappa.py`, results — at `10.5281/zenodo.22082967` (concept `…966`), the two records linked by `isSupplementedBy`/`isSupplementTo`. The PDF's checksum was verified against the corrected local build (`a688106438f71d0188dbca9518486cd6`), so what is public is the 23 August source — definition-of-done numbering fixed, "four files plus the README", the four- to elevenfold figures — not the superseded 21 August submission.
  - **Why this is the repo's own rule and not housekeeping.** `README.md` and `CLAUDE.md` are public on GitHub and told every visitor a paper was awaiting arXiv announcement. That is the artefact the reader was actually looking at, and it was emitted at them; the fix had to land there and not only in a note. `[R:repair-the-lying-artefact]`
  - **The second-order version was worse.** The tracking item that stood here described a wait that could never end — *replace the token when the identifier is announced* — and its presence made the situation look tracked. A tracking item for an event that cannot occur is not tracking; it is a note that makes a defect look handled, which is the same rule's sharpest form. It has been rewritten to track the condition that can actually occur (a journal DOI), not ticked as though the awaited event arrived.
  - **Scope of the sweep, reported because an empty result you looked for is evidence and an empty result you assumed is not.** Five occurrences of the placeholder, all in hand-authored prose: `README.md` ×2 (Status, Background), `CLAUDE.md` ×1, `TODO.md` ×2 (this Phase E record and the open item). `grep -rn -iE 'arxiv|zenodo|pending'` over `skills/`, `.cursor/rules/`, `.claude/rules/`, `.github/copilot-instructions.md`, `AGENTS.md`, `copilot-pack/` and `rules/` returned **no matches** — the claim never entered a rule body, so it never reached an emitted artefact and there was nothing to fix at source and re-emit. `ARXIV_METADATA.txt` is not in this repository; it lives with the paper.
- [x] **The measured result is stated, including the refutation** (README → Status → *What was measured, including what failed*), with relearn's own arm qualified as "plausible but unconfirmed" per `invented-requirements-feasibility.md`. No efficacy claim added.
- [x] **`skedasis` and `invented requirements` introduced** — once each, defined at first use, in README *Why* and `CLAUDE.md` *What this is*. Deliberately not scattered further (`[R:doc-currency]`).
- [x] **The prose rule count is gone rather than corrected.** README said "fourteen" against a corpus of fifteen. Replaced with a pointer to `relearn check`, which prints the count — a claim nothing checks is how it went stale in the first place.
- [x] **`AGENTS.md` committed** — George's call 2026-08-22. The `agents` emitter is a legitimate portability target for others and this is a public reference implementation, so the emitted file is a live worked artefact; excluding it from CI's default set would have recreated the very defect above.

### Open

- [!] **Paper 3's citation will change once more, and the trigger is a journal DOI — not an arXiv announcement.** arXiv's stated route is publication in a conventional journal with a resolving DOI to the published version, then appeal. If that lands, every site currently carrying the Zenodo version DOI wants the published DOI beside or instead of it: `README.md` (Status + Background), `CLAUDE.md` (What this is), and the Phase E record below. **Grep for `zenodo.22083202`, not for a placeholder** — the placeholder is gone and reintroducing one to make the sweep easier would put a false claim back in a public README to serve a private convenience. The concept DOI `10.5281/zenodo.22083201` already resolves to the latest version, so a *new version* of the preprint needs no edit here; only a different venue does. Nothing to do until then.
- [!] **`rust-typedd` route — George's call, and it is now a ratification rather than an open choice.** *Entry rewritten 2026-08-24; nothing implemented in this change.*
  - **The premise this entry opened with is no longer true.** It said the Rust discipline has two homes, the hand-authored `rust-typedd` skill and this repo's emitted layer, with four rules stated in both. Verified by reading the skill on 2026-08-24: its *Core practices* section was replaced on 2026-08-22 by a pointer to `~/.claude/rules/domain-rust.md` and an explicit "Do not restate a practice here." No rule body remains in it. The P2 violation in the author's own configuration is closed.
  - **Three things settled since the (a)/(b) framing was written.** (i) Route **(c)** exists and is *installed*, not merely available: `~/.claude/rules/domain-rust.md` is on disk, dated 2026-08-22, carrying `paths: ["**/*.rs"]` and the relearn generated-by marker. (ii) The claim that user-scope `paths:` rules never load is **refuted** — they load lazily on reading a matching file, `load_reason: "path_glob_match"`; method in `docs/load-semantics-measurement.md`. (iii) **Windows directory junctions are not traversed by rules discovery**, so sharing one rules directory between scopes by link is not available on this machine — an option that was open when (a)/(b) were framed and is now closed.
  - **The three routes, for the record.** **(a)** a `homes/<slug>.md` prose header in the library, rendered into the emitted skill by `emit::claude` — puts the framing under one roof, but introduces a non-rule input to a library whose stated invariant is that emitters read nothing the neutral rule does not carry, so it dents the design decision this repo exists to demonstrate. **(b)** a clean split — the skill keeps only the framing, the practices come from the library. **(c)** emit the practices to the user-scope rules layer, path-scoped to `**/*.rs`. *(b) and (c) are not rivals*: (b) says where the framing lives, (c) says where the practices land, and what is in force today is both.
  - **Recommendation: ratify (b)+(c) as built.** **The argument against it, which is the part worth George's attention:** (c) puts a generated copy outside version control, and it goes stale silently. It has. `relearn verify --targets claude-rules --home domain-rust --out $HOME` reports **`stale`, exit 1** as of 2026-08-24 — the installed file carries 8 rules against a library of 18, because the ten patterns ported that day were never re-emitted there. Nothing failed, nothing warned, and the discipline loaded into every Rust session on this machine is missing more than half of itself. A second cost: the rules layer has no `description`, so it cannot be summoned by intent — asking "review this design" with no `.rs` file open loads nothing, where a skill would match. Closing both is a mechanical re-emit plus a check that runs it, and it belongs in the `~/.claude` change, not here.
  - **Out of scope in this change, by instruction:** anything under `~/.claude`. The stale install above is reported, not fixed.
  - [x] **Both halves closed 2026-09-06, in stochos-lab.** The stale rules layer was re-emitted (8 rules → 18; ten rules had not loaded in any Rust session on this machine since 2026-08-24), the four generated skill layers were installed to `~/.claude/skills/`, and `global` + `domain-rust` were added to the Cowork bundle export — so both of George's surfaces now carry the same compiled library. `rust-typedd.skill` had been pointing Cowork at `~/.claude/rules/domain-rust.md` since 2026-08-22, a path that does not exist inside a Cowork bundle, so Cowork's Rust guidance was a pointer to nothing.
  - [x] **And the reason it went stale is now gated.** `verify-deploy.sh` §8 verifies both generated layers against this rule library on every run (skipping with a note where `relearn` is not installed). The stale install was never hard to fix — the re-emit is a one-liner — it was that **nothing read it**: this repo cannot see `~/.claude`, and `~/.claude` had no check that read this library. `[R:wired-artifact]`. The section was confirmed to fail on real drift before being trusted: it reported the skills layer stale while a pre-fix `relearn` binary was still on PATH.
  - **Still open, and still George's call: the CLAUDE.md prose was deliberately not retired.** Parts I and II are always-resident; a skill loads only when its description matches, so replacing one with the other would downgrade the global discipline from always-on to on-demand — a weakening dressed as de-duplication. Note the installed skills are *not* a competing home: `rules/` is the home and a skill is a compiled target like `AGENTS.md`. The competing home is the hand-authored prose, which is the four-homes item below.
  - **A third route opened 2026-08-22, after the decision was framed.** The brief that framed (a)/(b) directed that the choice be weighted toward a skill, on the stated ground that `~/.claude/rules/ecc/rust/*.md` had "never loaded in any session" and that `paths:` frontmatter was "honoured in a project's `.claude/rules/` and silently ignored in `~/.claude/rules/`". **That is refuted.** Measured the same day in a throwaway repo with an `InstructionsLoaded` hook: user-scope path-scoped rules load lazily *on reading a matching file*, emitting `memory_type: "User"`, `load_reason: "path_glob_match"`. All five `ecc/rust/*.md` fired on reading a `.rs` file, and all five `ecc/typescript/*.md` on reading a `.tsx` file. The `/context` readings that showed "none" were taken before any matching file had been read, which is the documented behaviour, not a failure. So route **(c)** is live: emit the Rust discipline to `~/.claude/rules/` as a path-scoped rule file — one home, loading automatically whenever Rust is touched, with no skill matcher and therefore none of the `description` problem below. Cost: the `.claude/rules/` emitter writes **no** `paths:` frontmatter today (verified — `.claude/rules/project-relearn.md` carries none), so (c) needs frontmatter emission; the per-`Home` glob data already exists in the `Scope` helper built for Cursor, so it is reuse rather than new modelling, and it is the same gap `relearn_load_semantics_2026-08-22.md` names — a path for every target, load semantics for one. **Do not settle (a)/(b) without pricing (c).**
  - [x] **Route (c) built 2026-08-22 — emitter half and install half.** `build`/`verify` now take `--home <slug>` (`cli::restrict_to_home`), without which emitting the rules layer to a user scope would also write every project layer there — always-resident, so they would load in every session in every repo. **Route (c) emitter half.** `Scope` is now the three-variant `LoadSemantics` (`Always` / `WhenReading(Globs)` / `OnRequest`) with `Globs` non-empty by construction, and the rules layer emits known-language domains scoped with `paths:`. `relearn build` → 28 files including `.claude/rules/domain-rust.md` carrying `paths: ["**/*.rs"]`; bare `verify` → `ok: 28 generated file(s) up to date`, exit 0. Two new whole-space properties lock it: no rules file ever carries an empty `paths:` list, and no on-request domain reaches the layer. **This does not close the decision** — it makes (c) available. The remaining half is George's call and lives outside this repo: pointing `~/.claude` at the emitted rules layer and taking the duplicate rule text out of `rust-typedd`.
  - [ ] **The duplication is four homes, not two** (found 2026-08-22 in `~/.claude/usage/untagged-rule-inventory-2026-08-16.md`). `make-illegal-states-unrepresentable` and `parse-dont-validate` each exist in `~/.claude/CLAUDE.md`, `~/.claude/skills/rust-typedd/SKILL.md`, `~/.claude/rules/ecc/rust/*.md` **and** `relearn/rules/`. The `ecc/rust` copy was assumed harmless because it was believed never to load; that belief is refuted (above), so it is a live duplicate. Whichever route lands must retire the `CLAUDE.md` and `ecc/rust` copies too, not only the `rust-typedd` one — and that edits the layer loaded into every session on the machine, so it wants its own change with its own commit.
  - [x] **`emit::claude_md` renamed to `emit::claude_rules` (2026-08-22).** It stopped writing a `CLAUDE.md` earlier the same day, so the name no longer described what the module did. Renamed with it: `Target::ClaudeMd` → `Target::ClaudeRules`, and the CLI value `--targets claude-md` → `--targets claude-rules`. **Breaking, and deliberately not aliased** — the crate is `0.1.0` with no tags and no releases, so a deprecated alias would only keep a wrong name alive for a caller that does not exist. Every call site was updated in the same change, including the two outside this repo (`~/.claude/skills/rust-typedd/SKILL.md` and `~/.claude/rules/ecc/rust/testing.md`, both of which document the re-emit command). Three test names containing `root_claude_md` were left alone: they name the hand-authored root `CLAUDE.md`, not this module. `[R:doc-currency]`
- [x] **Format gap closed 2026-09-06: the emitted skill `description` is now a bounded trigger.** It was worse than "not a trigger" — it was **invalid**: 6924 characters for `global` and 4322 for `domain-rust` against Claude's 1024-character cap, so the two skills anyone would want could not be installed at all. Now a lead sentence derived from `Home` plus as many rule **titles** as fit, remainder counted, bounded by the `SkillDescription` type rather than by a check. Error classes are gone from the field. Route (b)'s "leaves it open by design" no longer applies: the description is generated and adequate, and a hand-authored one would break the emitters-read-only-the-rule invariant. ~~*Original entry:*~~ **Format gap the port exposed: the emitted skill `description` is an inventory, not a trigger.** `emit::claude` concatenates error classes (`"Rules for domain: rust. Covers: No anyhow in library return types (anyhow in a library crate's..."`) where the hand-authored one is written for the matcher (`"George's Rust design discipline - Type-Driven Design supersedes any TDD-first default..."`). The `description` is the string a skill matcher triggers on, so a straight port would make the skill worse at the one job that field has. Tracked here whichever route (a)/(b) is chosen — route (b) leaves it open by design.
- [x] **Homing principle settled** — see Seed content above. Existing homes unchanged; the inconsistency dissolved once the criterion stopped being "does the principle generalise?".
- [x] ~~**`rust-typedd` still states four rules that `rules/` also states**~~ — **closed; verified by reading the skill 2026-08-24.** `~/.claude/skills/rust-typedd/SKILL.md` carries a revision note dated 2026-08-22 replacing its *Core practices* section with a pointer to `~/.claude/rules/domain-rust.md` and an explicit "Do not restate a practice here." No rule body remains in it; what it keeps is the framing that has no home in the neutral format — the standing override, the hierarchy of controls, *What unit tests are still for*, skill interactions. Route (c) is therefore complete on both halves and the P2 violation in the author's own configuration is closed. **This item stood `[ ]` for two days after the fact it asserted stopped being true** — recorded rather than quietly ticked, because a stale open item is the same defect as a stale doc. `[R:doc-currency]`
  - Not closed by it: the **four-homes** item above (`~/.claude/CLAUDE.md` and `~/.claude/rules/ecc/rust/*.md` copies), which is a separate edit in a separate repo, and the ten patterns ported 2026-08-24 that `~/.claude/CLAUDE.md` Patterns 1–9 still state in prose — the boot index is now a second home for those too, and retiring that text is the same `~/.claude`-side change.
- [x] **Decided 2026-09-19.** Refused, and the reason is the invariant. Granting `lint` a search path outside `rules/` makes its result depend on the machine it ran on, so a finding could not be reproduced from the repository alone and a *clean* result would mean only "clean where I happened to look" — the exact shape of claim this repository exists to refuse. A separate, openly filesystem-bound tool remains available and unbuilt. Original entry: **`lint` cannot see a competing home — George's call, unchanged.** `lint::lint` takes `&Library<Validated>` and reads only `rules/`, so a rule duplicated into a hand-authored file outside the library is structurally invisible to it: the two-homes problem was found by reading, and so were the ten unported Rust patterns on 2026-08-24. **The trade-off in one paragraph.** Detecting a competing home requires reading files the library does not own — a skill directory, a user-scope `CLAUDE.md`, an arbitrary path — which breaks the invariant that makes every other guarantee here cheap to trust: that `lint` and `emit` are pure functions of validated rules, deterministic, filesystem-free, and testable without fixtures. Granting `lint` a search path turns it into something whose result depends on the machine it ran on, so a finding could not be reproduced from the repository alone and a *clean* result would mean only "clean where I happened to look" — which is precisely the shape of claim this repo exists to refuse. The honest alternatives are a separate tool that is openly filesystem-bound and never confused with `lint`, or an explicit opt-in path list carried in configuration so the search space is at least declared and diffable. Recorded in FEATURES.md as `NOTHING YET — exposed`. **Nothing implemented; no decision taken here.**
- [x] **`claude-pack/` ships (2026-09-06)** — the library as installable Claude Skills, one folder per home, with a hand-authored README covering a Claude Code project (`.claude/skills/`), a whole machine (`~/.claude/skills/`), and claude.ai. Generated, never transcribed; `.gitattributes` pins it LF; CI verifies it as a second output root exactly as it does `copilot-pack/`. It could not have existed before the same day, because the skills it packages were over Claude's description cap and therefore uninstallable.
- [x] **Closed 2026-09-13 by the same gate, in both directions.** Was: the same exposure — it names per-home rule counts that nothing checks. Verified two ways when written; stale the day a rule moves home. Same fix as the item below, and they should be closed together: either generate the factual block from the library, or add a check that greps the numbers back out of the emitted artefacts. **Refreshed by hand 2026-09-12**, alongside the copilot pack: 24 / 18 / 2 / 2 → 27 / 18 / 2 / 2 / 3, the fourth home added, per-skill line counts added to a new install prompt, and the collision hazard written down (`cp -r` over a hand-authored skill of the same name destroys it silently — a skill with no `relearn:generated` footer is exactly what nothing here can recognise and refuse). As with the copilot pack, the refresh is the symptom: it was needed because nothing failed when the numbers went stale. Every number now sits beside the command that measures the real one, which is a mitigation, not the check this item asks for.
- [x] **Closed 2026-09-13 by `tests/pack_counts.rs`.** Was: numbers nothing checks. Added 2026-08-24. The pack's instruction file is generated and CI-verified; its README is hand-authored and asserts a rule count, a line count, a per-home breakdown and the eighteen `domain-rust` tags by name. All of those go stale on the next rule added, with nothing failing. Recorded in FEATURES.md on the same row rather than left implicit. Closing it means either generating the README's factual block from the library or adding a check that greps the numbers back out of the emitted file — the second is cheap and is the likely fix. `[R:doc-currency]` **Refreshed by hand 2026-09-12** (46 → 52 rules, 538 → 746 lines, 24 / 18 / 4 → 27 / 18 / 7, three graduated → eight, four project rules → seven, and a new install prompt that does not assume `.github/`). The refresh is the symptom, not the fix: it was needed because nothing failed when the numbers went stale, and nothing will fail the next time. The README now says so under its own table and points the reader at `grep -c '^## '`, which is the count that cannot rot — a mitigation in prose, not the check this item is asking for.
- [x] **Shipped 2026-09-19 as `--targets copilot-paths`.** One `.github/instructions/<home>.instructions.md` per home, `applyTo` derived from `LoadSemantics` — reuse of the glob table exactly as predicted. The design question the entry did not anticipate: what to do with a home whose load model is not a glob. `claude_rules` skips those because the skill target carries them; Copilot has no second channel, so skipping would have dropped fifteen low-latency rules from the layer entirely. Every home is emitted; an unscopable one declares `applyTo: "**"` and says in its header that it is unscoped and why, because these files are installed one at a time and the reader is the one choosing. Deliberately **not** in the default targets and not committed: it is the alternative shape to `copilot`, and emitting both into one tree states every rule twice. Held by `tests/copilot_paths.rs` over the real corpus, whose load-bearing assertion compares the two Copilot shapes carry exactly the same rules. Was:  Copilot also reads `.github/instructions/*.instructions.md` with `applyTo:` front-matter, which is the same load-semantics idea the Cursor emitter already models in `LoadSemantics` and the rules layer already emits as `paths:`. A `copilot-instructions` target that emitted one file per home with `applyTo: "**/*.rs"` for `domain-rust` would stop the eighteen Rust rules from occupying context in a repository with no Rust in it — the P6 argument that motivated `--home`. Reuse, not new modelling: the per-`Home` glob table exists. Not started; noted 2026-08-24 while building `copilot-pack/`, where the whole library goes into one always-on file because that is the only Copilot shape this tool emits today.
- [x] **Third home for the `relearn` skill — closed 2026-09-14, and it was three other things.**
      The text staleness was fixed **the same day it was flagged**: this line was written at
      14:28 on 2026-08-22 and `stochos-lab@baf41d9` brought `~/.claude/skills/relearn/SKILL.md`
      up to its 2026-08-20 revision at 16:53, two and a half hours later. The box was never
      ticked, so the entry outlived its subject by three weeks. What *was* stale, each found by
      fixing the one before it (`stochos-lab@a8ba857`):
  - **The binary.** `~/.cargo/bin/relearn` was the 6 September build — five subcommands where
    the tree has ten, and no knowledge of `applies_to`, so it could not parse this library at
    all. The `unheld-recurrence-report` SessionStart hook reported it on every session start,
    exactly as designed, and nobody read it. Reinstalled and verified by reading back the three
    properties that were wrong, not the exit code `[R:verdict-survives-the-channel]` — which is
    the same rule whose recurrence on `[R:attack-the-design-in-a-second-pass]` was recorded
    earlier today, against this very binary.
  - **The generated skills layer**, visible only once the binary could read the corpus.
    `~/.claude/skills/global/SKILL.md` was 123 lines behind and `domain-low-latency` and
    `project-design-architecture-tool` were absent. That is the **always-loaded** layer: the
    machine had been running a global rule set this corpus had already moved past.
  - **The eval suite** — the part of the July copy that was still July. `baf41d9` updated
    `SKILL.md` and left `evals/evals.json` at 22 July, and all three cases expect the
    *missing rule* branch, so nothing in the suite could tell the pre-2026-08-20 skill from the
    post one. Step 3's whole point is that a new rule is the **wrong** answer when a control
    exists or the fix belongs in code, and neither branch was exercised. Two cases added.

## Phase F — recurrence (2026-09-06)

*A rule that has bitten twice looked exactly like a rule that has bitten once. `Rule` recorded the incident that created it and nothing about the incident that followed it, so a recurring error class left two bad options — edit the original `incident` and destroy the record of what the rule was written for, or write a second rule, which is the P2 violation this tool exists to prevent.*

- [x] **`Recurrence` is modelled and the library can hold it.** `rules/<tag>.md` takes zero or more `[[recurrence]]` tables (`date` + `incident`). Separate field, not a second element of an `incidents` list: the first incident's date is the rule's `created` and a recurrence has none, so it carries its own. Backward compatible by construction — **no rule file was edited**, and `tests/corpus.rs` asserts all forty-six survive a parse/serialize round trip byte-identically.
- [x] **`relearn lint` reports `UnheldRecurrence`** — an `Active` rule (held by prose alone) that has fired again. `Warning`, no count threshold. The message names the remedy: promote to a control that can hold it, record with `Status::Graduated`.
- [x] **Every emitted format annotates a recurred rule** — `> Has recurred <n> time(s) since it was written; most recently <date>.`, beside the graduation note. Wired-ness locked by a whole-space property over all five emitters, which counts the notes per file, because there are five independent splice sites and a unit test of the renderer proves nothing about whether an emitter calls it.
- [x] **`rules/**` pinned `text eol=lf` in `.gitattributes`.** The corpus round-trip assertion is byte-exact, so a CRLF checkout would fail it on Windows only — the same defect already fixed for the emitted tree. Content unchanged: the index was LF throughout; only the working tree was renormalized. `[R:pin-eol-for-executable-text]`

- [x] **The corpus's first recurrence row, and it is `[R:verdict-survives-the-channel]`** (2026-09-06). Created 2026-09-03; `~/.claude/CLAUDE.md` documents a dated recurrence three days later — the rule was deployed in halves, the hook held the pipeline half, the edit half had no loaded home, and the class ran four more times in one session. Post-creation, dated, sourced from a written document, error class its own. The annotation now appears in four emitted layers (`claude_rules` excludes `Global` by design).
  - **Three of the four candidates in the design brief were rejected, and one rejection matters.** The **build-path incident** is *not* a recurrence: `repair-the-lying-artefact`'s own `incident` field already contains both the 16 and 24 August occurrences, and `created = "2026-08-24"` — the rule was *created by* the second. Recording it would have claimed the rule failed after being written, inflating the exact number this feature exists to make honest. The **producer/consumer derivation** cannot be dated from anything on this machine (it is Design-Architecture-Tool's, on a mount that is not here) and `date` is mandatory, so it was not invented. The **codepoint ban** has no rule in this library to attach to.
- [x] **`relearn lint` is now invoked by CI** — `--deny error`, both runners. The check had existed since Phase B with **no invocation at all**, which is `[R:wired-artifact]`, and it was found only because the first recurrence row made the check capable of failing.

### Open

- [x] **`[R:verdict-survives-the-channel]` was half-held — closed 2026-09-06, ticked 2026-09-16.** This box said the edit half was prose-only and that `Status::Graduated { to }` being single-destination made `Active` the honest status. **Both halves have had a hook since 2026-09-06:** `hook:gate-verdict-intact` for the pipeline half and `hook:multiline-pattern-eol` for the edit half, both present in `~/.claude/hooks/` and both wired in `settings.hooks.json` — verified 2026-09-16. The rule is `graduated`, with the two controls written into one `to` string separated by `" + "`. So the premise expired when the second hook was built and the box stayed open regardless, which is the same failure as the stale-skill box: *the fix landed and the record did not*. Found while designing `Status::Partial`, by reading the rule file instead of trusting this line — `[R:measure-the-claim-not-a-subset]`, one day after porting it.
- [x] **`[R:guarantee-needs-a-reader]` needed a status that could say "partly held" — shipped 2026-09-16.** Two controls for it exist here (`tests/ledger.rs::every_enforced_by_row_names_an_artefact_or_declares_itself_exposed` and `tests/pack_counts.rs`), each covering part of the class, and neither `Active` nor `Graduated` could record that: `Graduated` would print *"Also enforced by"* into five layers for ground those tests never claimed, and arm `RecurrenceAfterGraduation` — an `Error` — against the uncovered part. `Status::Partial { by, uncovered, date }` now does, and the rule is the first to carry it. See ARCHITECTURE.md's 2026-09-16 decision for the rejected alternatives. **The load-bearing part was not the variant** but the two exhaustive policy methods it forced: `lint` decided both the unheld-recurrence and the retired-reference questions with `matches!(status, Status::Active)`, which answers for a new variant *silently*, so `Partial` would have shipped quietly excluded from one finding and quietly flagged by the other. Probed by breaking each policy deliberately — one probe would not even compile.
- [x] **`Destination` was a string that encoded a list — closed 2026-09-16.** `Controls` is now a non-empty list of `Control { kind, name, covers }`, parsed once at the perimeter from the same text the rule files have always held. **The on-disk format deliberately did not change**, so no rule file moved, the federation wire format is untouched, and `verify` passed on all three output roots *without a rebuild* — the representation changed and not one emitted byte did. Three silent defects closed with it: a destination naming no control at all (`to = "we added a test"`) used to parse and emit, a mistyped prefix used to vanish from the federation's statistics, and `report` used to re-split the raw string on whitespace, truncating every name that contained one. `report`'s duplicate five-variant `Control` enum collapsed into `ControlKind`. The structured-TOML alternative was costed and rejected in ARCHITECTURE.md's decision for that date.
- [x] **Decided 2026-09-19.** Recorded behaviour, not open work. Only one spelling can round-trip, so accepting `proptest:` as an alias would rename itself on the next rewrite and produce a diff nobody asked for. Original entry: **`proptest:` is no longer accepted as a spelling of `property:`.** The old ad-hoc reader in `report` took both; `ControlKind::from_prefix` takes only the canonical one, because two spellings cannot both round-trip and an alias that silently rewrites itself produces a diff nobody asked for. No rule in this corpus or in `relearn-corpus` uses it, so nothing broke — but a *pulled* rule from an install that does would now stop the build rather than be quietly accepted. That is the intended direction (a parse failure is loud), and it is recorded here because it is the one behaviour change in an otherwise format-preserving commit.
- [x] **The next `matches!` on a domain enum now has a guard — closed 2026-09-16.** `tests/exhaustiveness.rs` fails on `matches!`, `if let` or `let … else` over any of ten domain enums in production code, exempting test code and the enum's own module — the two narrowings that make it buildable, since a blanket ban would hit `VerifyStatus::is_clean` and every `assert!(matches!(..))` in the suite. A test rather than a script, so CI cannot forget it. It found a tenth violation on its first run (`Rule::is_publishable` deciding `Federation`), which the manual sweep had passed over *because it was safe* — safe by the accident of which variant happened to be tested. Probed both ways: reintroducing the original `lint.rs` defect fails it, as does the `if let Authority::Cached` form.
- [!] **A wildcard arm in a `match` on a domain enum is still uncaught**, and it is the shape that has actually bitten. `poke.rs` had `other => other.version()?` — a wildcard wearing a name, which a grep for `_ =>` misses and which only the compiler found, during the sweep. `tests/exhaustiveness.rs` states this gap in its own header rather than implying coverage it lacks. Catching it needs to know *what is being matched*, which is a parser rather than a substring: either `syn` as a dev-dependency (priced, and it would be the first parse of our own source), or a `dylint` lint, which is a dependency plus a build step for a check that would otherwise fit in eighty lines. Neither is obviously worth it for one historical instance; recorded so the next instance is the second, not the first.
- [x] **Decided 2026-09-19.** Correct as it stands. `Partial` makes no whole-class claim by design, so attaching a control kind to it would publish a claim the control never made — and the aggregate's question is which control kinds hold which classes. Under-reporting is the honest direction for something that leaves the machine. Original entry: **A partial graduation reports no control kind to the aggregate.** `report::Control::of` reads only `Status::Graduated`, so a `Partial` rule contributes its recurrences with no control kind attached, even though `by` names real controls. An under-report, not a false one, and publishing less is the safe direction for something that leaves the machine — but it biases the cross-install view toward full graduations, in a dataset whose question is *which control kinds hold which classes*. Noted 2026-09-16 while sweeping; the doc comment now says so. Fixing it means deciding what a partial graduation's control kind *means* when the control covers half the class, which is a question about the aggregate's semantics rather than a bug.
- [x] **Ported 2026-09-16. Three tagged rules had no home in this corpus.** `[R:search-before-you-build]` and `[R:measure-the-claim-not-a-subset]` are defined in full, with incidents, in `~/.claude/CLAUDE.md`; `[R:delegate-a-fan-out]` in `~/.claude/rules/ecc/common/agents.md`. All three are cited by tag — `delegate-a-fan-out` twice — and none of the three exists in `rules/`. They reach only the hand-maintained machine-local layer: they compile to no other assistant, carry no provenance this repository can audit, and no rule here can cite them in a way `relearn lint` resolves. That is the condition `[R:attack-the-design-in-a-second-pass]` and `[R:answer-the-requirement-at-its-layer]` were in before 2026-09-14, and that port is the precedent. Found by **reading**, like both of those — `lint` takes `&Library<Validated>` and cannot see a rule that is not in it, which is the Phase B exposure recorded above. **Done:** all three ported, homed `global`, `origin = "mined"`, each keeping its ORIGINAL `created` date (2026-08-16, 2026-08-30, 2026-08-31) rather than the port date, per the `[R:guarantee-needs-a-reader]` precedent — the rule existed from its incident, only its home moved. Corpus 54 -> 57. Each carries a hand-authored `published_incident`; the private `incident` keeps the specifics, and both were scanned by the name gate.
- [x] **Resolved 2026-09-16 — pointers.** The three ported rules briefly had TWO homes, exactly as the 2026-09-14 port did: `~/.claude/CLAUDE.md` stated `[R:search-before-you-build]` and `[R:measure-the-claim-not-a-subset]` in full prose, `~/.claude/rules/ecc/common/agents.md` stated `[R:delegate-a-fan-out]`, and the compiled corpus reached that same machine through `~/.claude/skills/global/SKILL.md` — the P2 shape this tool exists to prevent, appearing in the tool's own supply chain. **George chose pointers, installed the same day** (stochos-lab `acaf069`). Deliberately NOT bare links: the compiled skill is on-demand, while `CLAUDE.md` and the rules layer load into every session, so a bare link would have demoted three always-on rules to ones that fire only when something thinks to invoke the skill. Each section therefore keeps its trigger and its failure-mode check and hands the depth to the corpus — the shape the file already uses for `rust-typedd`, which names itself "the always-on summary" and "the boot-layer index". Each rule's signature question had to survive, because the two sections that follow open with *"the three checks above"* and *"the four checks above"* and enumerate them. **`verify-deploy.sh` caught a real defect on the first run:** the new pointers named `~/.claude/skills/global/SKILL.md` as the home of the full text while that deployed copy was still stale at 27 rules — `[R:guarantee-needs-a-reader]`, committed by the very change adding the guarantee. Regenerated to 30 (binary checked against a fresh build byte-for-byte first, not by mtime), Cowork export rebuilt behind it, gate GREEN. **Still true:** nothing checks for a competing home, so the next one will also be found by reading.
- [x] **The counter-metric shipped 2026-09-06.** `origin = "mined" | "codified"` is a mandatory field on every rule; `relearn lint` prints `N rule(s): X recurred, Y inert (codified and never fired)` and never the recurrence count alone. Corpus: **46 rules, 1 recurred, 19 inert.** Mandatory rather than defaulted, because a default classifies unlabelled rules silently — the exact under-reporting the counter exists to detect. The 19 codified rules were classified by **their own `incident` text** ("Codification-dated"), never by a judgement made during the migration; the tempting second signal, "ported" (32 rules), was measured and rejected as describing migration between homes rather than origin. `Codified` deliberately carries no source payload: "codified without a source" is not an illegal state, so a payload would make nothing unrepresentable.
- [x] **Done 2026-09-06.** `Status::Graduated` carries a `date`, and `Finding::RecurrenceAfterGraduation` is an `Error` — the one recurrence finding that fails CI, because the emitted `> Also enforced by {to}` line is false in all five formats. A recurrence *before* the graduation is deliberately not a finding: it is usually the incident that prompted it. The boundary is `>` not `>=`, pinned by a test. The three graduated rules were dated from git (each hook's first commit, 2026-07-21), not guessed.
- [x] **Shipped 2026-09-06**, unconditionally rather than as an option: the recurrence count is never printed without its counter, so there is nothing to opt into. `lint::Tally` holds both.
- [x] **The first real recurrence entries — answered 2026-09-06; see the row above.** One recorded (`[R:verdict-survives-the-channel]`), three rejected with named reasons, and the rejections were the substance: the build-path candidate was *not* a recurrence at all. Original entry, kept for its reasoning: Three are already documented in prose elsewhere and would be the corpus's first rows. Deciding that a later incident is *the same error class* rather than a neighbouring one is a judgement, and getting it wrong inflates the only number that says whether a rule is working. Candidates reported, none added: (i) the **producer/consumer derivation**, corrected twice and surviving in a third document until somebody happened to read it — *"third home of a correction made in two"*; (ii) the **build-path claim**, written up 2026-08-16 and recurring on the 24th because the script that made the false claim was never repaired; (iii) the **codepoint ban**, violated three times while written down.
- [x] **Installed 2026-09-06 in stochos-lab**, with four assertions in its hook suite. Reports unheld recurrences and recurrence-after-graduation at session start, always exits 0, is not a gate and not a detector, and prints the counter-metric beside the metric. Building it found two bugs of its own error class: the first draft folded a hard failure into `|| true` (silence indistinguishable from a clean library, `[R:verdict-survives-the-channel]`), and MSYS did not convert a `/tmp` path for the native binary (`[R:xplat-fixtures]`). Both are pinned.

## Phase G — repository hygiene (2026-09-06)

An employer-owned product name was found in this public repository: once in
`rules/repair-the-lying-artefact.md`'s `incident` field and twice in `TODO.md`. Found by
a PostToolUse output hook firing on an unrelated read of the rule file, not by any check
either repository owned. The source repository (Design-Architecture-Tool) has had a
working detector for that name since its M33-A4 and was green throughout — a gate scans
its own tree, and the quotation travelled without it.

- [x] Redact the working tree. Three occurrences, replaced by pattern so the term was
      never typed or printed; re-scanned to zero. `[R:report-the-hit-not-the-match]`
- [x] `scripts/no-banned-names.sh` — the matcher, safe to publish because it contains no
      names. Two-pass (join runs of up to three tokens; slide each stored length inside
      each token), reports location and count only, exits 2 when disarmed.
- [x] `.githooks/pre-push` — the invocation. Install with
      `git config core.hooksPath .githooks`.
- [x] `.gitattributes` pins `scripts/**` and `.githooks/**` to `eol=lf`; a CRLF checkout
      leaves a trailing CR on the shebang and the gate arrives unrunnable, failing OPEN.
      `[R:pin-eol-for-executable-text]`
- [x] `[R:names-travel-with-the-quote]` added to the corpus (global, mined).
- [x] `[R:report-the-hit-not-the-match]` **ported** — it was cited by this repository, by
      the source repository's own scanner, and by the stochos-lab hook, and it was in the
      library nowhere. `lint` reported it as a dangling reference the moment the new rule
      cited it. Graduated to `hook:banned-name-in-output`, 2026-09-05.
- [x] `[R:verdict-survives-the-channel]` graduated — the unheld-recurrence warning Phase F
      added had been firing at every session start since 2026-09-06. Both halves now have
      hooks: `gate-verdict-intact` (pipeline) and `multiline-pattern-eol` (edit).

### Open

- [!] **The history still carries it — 52 of 52 commit trees, and the remote is public.**
      HEAD is clean as of this commit; every commit before it is not. The treatment is
      the one George chose for the same class in Design-Architecture-Tool M35-A3:
      **delete and recreate the remote, not force-push**, because a force-push leaves the
      old objects reachable on GitHub until they are collected. **Blocked on scope:**
      `gh auth status` reports `gist, read:org, repo` — no `delete_repo`. Needs George to
      add the scope or delete the repository in the browser; the local rewrite can be
      prepared either way.
- [!] **The gate is configuration-dependent and FEATURES.md says so.** It is armed only
      where a maintainer has run `git config core.hooksPath .githooks` *and* has the term
      list. CI cannot run it — a fork has no list, and failing every fork for a reason
      that is none of its business is worse than the gap. There is no design here that
      gives a public repository a committable list: one protected name normalises to four
      characters, so publishing the salt publishes the name.
      **Partly mitigated 2026-09-16:** `README.md` now carries the install line in
      *Working in a clone*, which is the one file a reader of a fresh clone actually
      opens, and states plainly that nothing in the repository can verify the clone ran
      it. That lowers the odds of an unarmed clone; it does not close the item, because
      documentation is not a reader `[R:guarantee-needs-a-reader]` — the gap is still
      that a clone which skipped the line pushes with no warning.
- [!] **The term list now exists twice.** `Design-Architecture-Tool/scripts/banned-terms.sha256`
      is the source — it must stay repo-local, or a clone of that repository arrives
      disarmed — and `~/.claude/usage/banned-terms.sha256` is the machine-wide copy this
      gate reads. A term added to one does not reach the other. Nothing checks that they
      agree. Two homes for one fact is the P2 shape; the honest options are a check that
      compares the two files, or making the machine-wide file the source and the
      repo-local one a generated copy.
- [!] **`[R:report-the-hit-not-the-match]` now has a competing home.** Its prose still
      stands in `~/.claude/CLAUDE.md`, and the corpus copy is emitted beside it into
      `~/.claude/skills/global/`. This is the same two-homes problem already carried
      above for four Rust rules, and `lint` is structurally unable to see it. Not
      resolved here: gutting George's always-loaded layer is his call, not a tidy-up.

## Phase H — the design-architecture-tool home (2026-09-06)

The corpus's third project home, and the first mined from a repository that had already
written every one of these incidents down in its own `CLAUDE.md` and gone on to violate
them anyway. Nothing was authored to fill the home; each rule points at prose that
existed and failed. Corpus 48 -> 51.

- [x] `[R:role-is-an-edge-property]` — producer/consumer derived from spin mode.
      **The corpus's first rule to carry two recurrences**, and the first time the
      Phase F machinery changed an outcome rather than reporting one: the graduation was
      drafted against the renderer test that was already in place, and `lint` raised
      `recurrence after graduation` — correctly, because that test reads a renderer and
      the 2026-08-30 recurrence was a sentence in a markdown file. The control was built
      (`shared/src/doc_claims_tests.rs`, 11 tests, probed both ways) and the graduation
      re-dated to the day it became true.
- [x] `[R:seeded-data-needs-a-migration]` — seed data reaches only fresh installs, and
      the suite agrees with you because a test database is always a fresh install.
- [x] `[R:verify-the-glyph-exists]` — a codepoint chosen from the document it was typed
      in rather than from the four faces the app loads. The instruction file prescribing
      the substitute was itself the defect.
- [x] Two of that repository's incidents recorded as **recurrences of global rules**
      instead of new project rules — M34's per-connection `PRAGMA foreign_keys` against
      `[R:verify-through-production-path]`, M33's six-week-old all-clear against
      `[R:guarantee-needs-a-reader]`. Both now report as unheld recurrences, which is
      the true state: the instances are held, the classes are not.
- [x] Emitted into that repository as `.claude/rules/project-design-architecture-tool.md`
      and pulled in by its `CLAUDE.md`.

### Open

- [!] **The emitted layer in that repository is verified only from here.** Its footer
      carries a body `sha256`, so a hand-edit is detectable — by `relearn verify`, which
      needs this binary and this rules directory. A clone of that repository has neither,
      so its own gate cannot check the file it loads every session. Its FEATURES.md
      records the row as configuration-dependent and its TODO.md carries the buildable
      alternative: a check on that side that recomputes the footer hash itself.

## Phase I — pricing a dependency (2026-09-12)

`[R:price-every-dependency]`, `origin = "codified"` and labelled so deliberately: it came
out of a design conversation, not out of a failure that cost something, and calling it
mined would corrupt the inert fraction that keeps the corpus honest. Corpus 51 -> 52.

- [x] The rule: every dependency gets a decisions-log entry — purpose, why this one, what
      was refused (including writing it by hand), feature flags, dev-only — and so does
      every dependency **refused**, which is the half with no artifact and therefore the
      half that is lost. Sibling of `[R:decisions-log-records-rejected-alternatives]`,
      which it cites rather than restates.
- [x] The anchoring pair measured here before the rule was written, not assumed: nine
      dependencies against one named in the decisions log as a crate choice. Recorded in
      the rule's `incident` and in FEATURES.md.
- [x] `relearn lint` reports **no** `OverlappingScope` against the sibling rule — and the
      negative is weak evidence, because that check groups on the *literal* error-class
      text (case-insensitive). Two rules can overlap in meaning and never collide there.
      Reported rather than resolved; merging the two is George's call.

- [x] **The gate, built 2026-09-12: `tests/dependencies.rs`.** Every dependency in every
      manifest must be named by a decisions-log row whose **Decision** cell opens with
      `Dependency:` or `Dependencies:`. A test, not the `scripts/verify.sh` this item
      originally waited for: `cargo test` already runs in CI on both platforms, so the
      gate is wired by construction and cannot arrive disarmed the way
      `scripts/no-banned-names.sh` can. Three further tests keep the check upright — the
      exemption list must shrink, a stale exemption fails, and a renamed log section
      fails rather than quietly emptying the priced set. Probed in eight directions
      before being trusted; see FEATURES.md for the list. **The refusal half remains out
      of scope for any gate** — no artifact exists to check it against; that half is P4
      by construction, and the rule therefore stays `active` rather than graduating,
      because a single-destination `Status::Graduated { to }` would print an
      "Also enforced by" line that is false for half the rule. Same argument as
      `[R:verdict-survives-the-channel]`, recorded under Phase F.

### Open

- [x] **Decided 2026-09-19: not retrofitted, and the exemption list is the honest record.** Was: retrofit the existing log, or decide not to. Eight of nine dependencies have no
      entry: `serde`, `clap`, `thiserror`, `anyhow`, `sha2`, `proptest`, `tempfile`,
      `trybuild` (`toml` is named, as the reason for a format decision). Writing entries
      now means reconstructing rejected lists nobody remembers, which the sibling rule
      warns produces the flattering version. Listed as candidates only; adding them is a
      judgement about what was actually considered at the time.
- [x] **Resolved 2026-09-19 by the decision above: eight of the nine have no row to point at, so the pointers would dangle.** Was: the manifest comments are not yet pointers. Most carry standalone reasoning with
      no reference to the log — the drift shape the rule names. One line each to fix, but
      it belongs with the retrofit decision above rather than ahead of it.

## The federation programme — sequencing, and what is next

[`docs/federation-programme.md`](docs/federation-programme.md) sequences the design into phases
(A1 → A2 → B1 → B2 → B3 → C1 → C2), **one phase per session**, each with the thing that can go
wrong named in advance. It does not restate the design; `docs/federated-relearn.md` is the source
and the programme is wrong where the two disagree.

- [x] **Every phase A1 → C2 has shipped**, one per session, each with its own section below.
      What the programme sequences is done; what it does not settle is the question it
      says to answer *before* B1 and which is still open — see the item below.
- [x] **A1 — `applies_to`: home is not scope.** Shipped 2026-09-13 (`3798521`); see below.
- [x] **Phase 0 — repair the design document (no code). Done 2026-09-13.**
  - [x] **0.1** The config sentence in §2 is gone: an invocation names its audience on the command
        line, and **a config file is refused rather than deferred**, with the condition that would
        reopen it recorded in the decisions log (an install needing the same scope set every time
        — at which point the argument is made against invariant 3 explicitly and
        `tests/solo_mode.rs` changes in the same commit).
  - [x] **0.2** The install pseudonym lives **inside the cloned aggregate repository**, never on
        the machine: `reports/<install-id>.toml` is its own name, the clone is a path given on the
        command line, and `solo_mode.rs` needs no exemption. Rejected: a home dotfile (breaks
        invariant 3), typing the id each run (a typo silently forks one install into two and
        inflates every count), a machine fingerprint (derivable is re-derivable).
  - [x] **Found while repairing, fixed in the same commit:** the status banner still said
        *"Nothing in this file is implemented"* after A1 shipped, and invariant 1 named
        `Audience::Everything` — a type that was never built — as what held it. Same class as 0.1;
        an enforcing artefact that points at a plan is what `FEATURES.md` exists to prevent.
  - [x] ~~0.3 `copilot-pack/README.md` is stale~~ — **already closed**; measured before filing
        (52 rules / 746 lines claimed, 746 lines and 52 `^## ` headings actual, layers 27/18/7
        matching `relearn list --home`). Its counts remained hand-written with nothing checking
        them, and that exposure — not the staleness — was the real lesson of the item.
        **Closed 2026-09-13** by `tests/pack_counts.rs`, which reads both pack READMEs in both
        directions: every claim must match its file, and every file must be claimed.
- [x] **Answered 2026-09-14: yes, a second corpus — not a second person.** Federation's value is entirely in
      the second person running it. A1 and A2 are worth building regardless; B onward is not.

## There is a second install (2026-09-13)

Not a second person — that cannot be manufactured, and two pseudonyms from one person is
exactly the fake population the k-floor exists to detect. But a second **corpus**, on this
machine, taking rules from a shared one it does not own.

- [x] **The shared corpus exists**: `~/Documents/relearn-corpus`, a git repository with
      `rules/`, `reports/`, a README stating the rules of engagement, and the scheduled
      recompute C1 recorded as missing. A directory, not a service.
- [x] **Five rules published**, each with a `published_incident` written for it — the raw
      quotations stayed home, verified by sixteen probes for repository names, product names,
      milestone identifiers, file paths and type names: **zero** crossings, all still present
      locally. The real term list was the matcher's input, not a fixture.
- [x] **`mesh-watchdog` is the subscriber** — aptly, since the sentinel-value rule was mined
      from its own failure and has now come back to it as a cache from a corpus it does not
      own. It holds five cached rules, lints clean, and builds its own `AGENTS.md` and
      `.claude/skills/global/SKILL.md` from them.

**What the second install found in its first ten minutes**, which no fixture had:

- [x] **The publishable unit is a rule plus its citation closure, not a rule.** Publishing
      three rules gave the subscriber a library with two dangling references, and `lint` fails
      on those at the default deny level — so a new subscriber's *first* command fails, which
      is the worst possible first impression. The closure of those three was five; publishing
      the other two closed it.

Carried:

- [x] **`contribute` now warns about citations the destination does not carry** — shipped the
      same day the second install found the failure. Two of the three candidate responses were
      wrong on inspection: a **refusal** would make any rule citing a project-homed one
      permanently unpublishable, and **offering to publish the closure** cannot work, because a
      closure member with no published incident cannot be published either and the offer would
      fail halfway. So it warns, and distinguishes *not published yet* (resolves when somebody
      publishes it) from *never publishable* (does not). Probed both ways against the real
      corpus.
- [!] **The corpus has one contributor and one subscriber, both George.** The mechanics are
      exercised; nothing about whether a rule mined from one person's sessions helps someone
      whose sessions they never saw is. The aggregate stays empty and correctly so: five
      distinct installs, and there is one.
- [x] **Committed 2026-09-14 in that repository, no remote per its own charter.** Was: uncommitted, left for
      review rather than pushed into another repository's history.

## The public face caught up with the code (2026-09-13)

Everything above shipped in one session, and the README's Status paragraph still described a
repository with no federation in it — the first thing a reader sees, and `[R:doc-currency]`
in the file where it costs most.

- [x] **README Status** names the sharing flows and says plainly that they are optional and
      that a single engineer needs none of them.
- [x] **A "Sharing a corpus between installs" section** with the real two-install workflow:
      publish, status-check, apply. It leads with what does *not* change for one engineer,
      because that is the product, and names the three refusals as the point rather than as
      inconveniences.
- [x] **`ARCHITECTURE.md`'s dependency direction** now mentions `pull`; **`CLAUDE.md`'s command
      block** now lists the federation commands it had never carried.

Carried:

- [x] **`docs/worked-example-two-installs.md`** — the two-install round trip, with real
      output for every step: publish, status check, pull, build, revise, the backwards
      refusal, the poke, refresh, fork, and the pull that leaves the fork alone. Its
      reproduce script was run verbatim from a checkout before it was written down.

## Multiuser mode: both publisher and cache creator (2026-09-13)

George's call, in three decisions: `contribute` carries a version, the shared corpus lives on a
common drive both installs can see, and the cache write path gets its own type-level witness
rather than a flag.

- [x] **`relearn pull`** — the receiving half, and the only thing that creates a cache. Four
      refusals asked once at the constructor: a withheld home never arrives, an unnumbered
      upstream rule is refused rather than cached, and pulling over a local rule or a
      deliberate fork is refused. Re-pulling over a cache is the update path.
- [x] **`PulledRule`, the second witness.** `write_rule` takes an `EditableRule` that refuses a
      cache; `write_cache` takes a `PulledRule` that is only ever a cache. Neither can be minted
      for the other's subject, so "edit a cache in place" stays unconstructible rather than
      becoming reachable by passing `true`. It **owns** its rule, because the cache does not
      exist until the witness is minted — one construction site, no second to keep in step.
- [x] **`contribute --version` is required.** Given on the command line rather than read from the
      rule, because publication is what assigns it and a fork's authority records the revision it
      was *forked at*, not the one it is being published as.
- [x] **Observed end to end through the binary**, two installs and a shared directory: publish at
      revision 3 → pull → build (the cache emits into the receiving install's global skill) →
      adopt → pull refused → publish revision 4 → the stale-cache poke fires. The first time any
      of this has run against something that actually happened rather than a fixture.

Carried:

- [x] **A published revision must supersede the one already there** — closed 2026-09-13 by
      `SupersedingVersion`, a witness `to_document` takes instead of a bare `Version`, so a
      backwards republication is a document that cannot be built rather than a check somebody
      performs. `contribute` now reads its destination for that one question.
- [x] **Found while closing it: `contribute` had no write guard at all.** A bare `fs::write`
      since B2 — publishing over an existing rule destroyed it silently, whether it was an
      older revision of the same rule or somebody else's rule at the same path. It survived
      because it was the only rule-file writer with no sibling to compare against; `write_cache`
      arriving made two that guarded and one that did not. All three now share one
      `guarded_write`.
- [x] **`pull --all`, with a plan, an audience and a prune** — George's call, shipped
      2026-09-13. Without `--confirm` it is the status check to run before starting work:
      every rule in both corpora named exactly once, including the ones it will not touch
      and why. `--scope` bounds it (§10's own answer to a corpus too large to compile);
      `--prune` removes caches that are gone or retired upstream, and **only caches** —
      `DroppableCache` refuses a rule you own and a fork you took, because a cache is
      regenerable and source is not. Idempotent: the second run says `Nothing to do.`
- [x] **The stale-cache poke now has a command to act on.** `cache-behind` still reports one
      rule at a time inside `lint`; `pull --all` is what refreshes them, and `--prune` is
      §12.6's third resolution taken deliberately rather than applied behind anyone's back.
- [x] **Found while building:** a retired upstream rule the install already held was reported
      twice in one run — *"already at upstream's revision"* and *"unwanted, and kept"*. The
      attic check had been conditioned on whether the rule was held; it is not.

Carried:

- [!] **`pull --all` re-reads and re-writes the whole corpus every run.** Fine at fifty rules
      and pointless at five thousand; nothing here is incremental, and nothing records what
      was last pulled. The convergence test is what makes the waste safe rather than absent.
- [x] **A prune on a retirement has no inverse, and now says so before it is taken.** A cache
      dropped for being *gone* returns if it comes back; one dropped because upstream
      **retired** the rule cannot be pulled again at all, since `pull` refuses a retired rule
      — which is what "retired" means. Noticed while writing the carried list, so the plan now
      counts those drops and names `adopt` as the alternative for anyone holding evidence
      upstream does not.

## The ledger reads its own enforcing artefacts (2026-09-13)

Seventy `Enforced by:` claims, and nothing checked that any of them named something real.
`tests/ledger.rs` now does, in two directions.

- [x] Every path-qualified citation resolves — the file exists and defines that function,
      comments stripped so a test named only in a comment cannot satisfy a claim that it
      exists. Thirty-two citations across twelve test files.
- [x] Every `Enforced by:` row names something in backticks **within sixty characters of the
      marker**, or says `NOTHING YET`. The window is measured, not chosen: fifty-eight rows
      open with their citation and the rest reach it within thirty.
- [x] **The first version of that second direction was vacuous and the probe caught it.**
      Asking whether a backtick appeared anywhere after the marker passes on every row by
      accident, because the rows are paragraphs. It was found by probing — replacing a
      citation with "careful review at commit time" and watching the check stay green.
- [x] **Found while building:** the contradiction row claimed enforcement by "a
      **process-control**, not a code artifact" — the only row naming neither an artefact nor
      a gap, while this file's header had said since 2026-08-13 that the row *"stays exposed
      by design"*. Corrected to agree with the header rather than carving the checker an
      exception for it.
- [x] The marker is matched **in bold**: the ledger's header explains the "Enforced by" column
      in prose, and the first version failed on that sentence —
      `[R:detector-excludes-own-definitions]` inside the artefact meant to keep the ledger
      honest.

Carried:

- [x] **Decided 2026-09-19.** Refused. Resolving bare `module::item` citations means searching the tree, and the same shape covers `env::var` and `ExitCode::FAILURE`; a checker carrying an exclusion list of standard-library names grows an entry per release until somebody mutes it, and a muted check is worse than none. The path-qualified form stays the recommendation because it carries its own file. Original entry: **Bare `module::item` citations are not resolved** — `lint::tally`, `Rule::serves`,
      `Status::emittability` and about a hundred more. Resolving them means searching the tree,
      and the same citation shape covers `env::var` and `ExitCode::FAILURE`; a checker carrying
      an exclusion list of standard-library names grows one entry per release until somebody
      mutes it. The path-qualified form was chosen because it carries its own file. Closing
      this properly probably means citing artefacts by path everywhere, which is a convention
      change across seventy rows and **George's call**, not a refactor to do in passing.
- [x] **Decided 2026-09-19.** Cannot be automated, and saying otherwise would be the failure it describes. Whether a named test *exercises* a feature is a judgement about meaning; a check that claimed it would be a green light consuming forgeable evidence, which is `[R:wired-artifact]` exactly. It stays a review question, and the ledger's value is that the artefact is named and runnable. Original entry: **Nothing checks that a named test actually *exercises* the feature its row claims.** The
      check is existence, not relevance — a test could be renamed to match and assert nothing.
      That is the half no gate performs, and it is the same boundary `[R:wired-artifact]`
      draws: evidence that is forgeable by someone who wants to forge it, and sufficient
      against the failure that actually happens, which is a rename nobody noticed.

## The counting gate caught its author (2026-09-13)

`tests/pack_counts.rs` shipped, and within the hour `[R:doc-currency]` fired twice in the
file the gate was recorded in.

- [x] **FEATURES.md's copilot-pack row still said "Nothing checks those numbers against the
      library"** — made false by the change that shipped beside it, in the document whose
      whole job is to be the behaviour contract. Repaired, and the row now names the gate.
- [x] **FEATURES.md's claude-pack row said "four skills: `global` 24 rules"** against five
      skills and twenty-seven. Repaired by **removing the counts**, not by resetting them: the
      live numbers have one home, the checked pack README, and the row points at it.
- [x] **Recorded as a recurrence on `[R:doc-currency]`**, which is what the corpus is for. It
      was `active` with no recurrences; it now carries one, and `lint` reports a third unheld
      recurrence as a result. That is the honest reading, not a regression.
- [x] **The gate caught the drift its own recurrence caused.** Recording the recurrence added
      two lines to `copilot-instructions.md`; `tests/pack_counts.rs` failed naming the file,
      the line and both numbers, and the README was corrected. The loop ran end to end inside
      one change.
- [x] **The row describing the gate had restated the counts too**, which would have been the
      third occurrence. It now describes them instead of quoting them.

Carried:

- [x] **Closed 2026-09-19: the prose now names every rule in the pack.** It had grown worse than when written — sections existed for the repository discipline and Rust only, so the 13 Java and 15 low-latency rules, 28 of 83, were undescribed. Two sections added and checked mechanically: every java- and low-latency-homed rule is now named in the file. Was: the prose is hand-authored and unread: named rule
      tags, the "Two rules describe an enforcement…" paragraph, and the per-home alternatives
      in the install prompt. Sentences rather than counts, and the gate reads counts.
- [x] **Decided, and the decision is in the item: dated historical counts stay.** Each is true as written and a gate that failed them would be demanding that history be rewritten. Closed 2026-09-19 as a recorded position rather than open work. Was: dated historical counts stay — FEATURES' "Corpus at 2026-09-06: 46 rules", the
      design document's "State when written: 52 rules", the decisions-log rows quoting what a
      README said at the time. Each is true as written, and a gate that failed them would be
      demanding that history be rewritten. They are tagged with their date and now say so.

## Phase C2 — the poke, inside `lint` (shipped 2026-09-13)

§6's four triggers, the reactive one uncapped and on, broadcast capped at a number the
operator passes. Emission unchanged for all 52 rules (`verify`: 63 generated files up to
date) — no emitter reads the poke, and a test reads their source to keep it so.

*A fifth trigger landed the same day, after this section was written* — §12.6's upstream
retirement, on George's call, recorded under Phase J below and in the decisions log. The
items in this section describe C2's four and stay as they were.

- [x] The default table is `Trigger::on_by_default`, one exhaustive match and nowhere
      else, asserted with data present for **all four** triggers so the two that stay
      quiet are making a decision rather than finding nothing.
- [x] The cap reaches broadcast only, because `Reach` is a property of the trigger rather
      than a `bool` where the cap is applied — and withheld pokes publish their count.
- [x] **A poke can never change a verdict.** No severity, printed after the exit code is
      decided, asserted through the real binary twice (with and without a clone) and in
      both directions (a failing lint still fails when poked).
- [x] `lint` reports and `build` emits: a source-level scan, the same shape as the
      aggregate's.
- [x] No `SessionStart` hook installed. The snippet is `docs/session-start-poke.md` and
      stays there.
- [x] The k-floor reaches the poke by construction — the population trigger reads the
      aggregate's rows, so there is no second policy to keep in step.
- [x] The one model change the phase forced: `Authority::Local { version: Option<Version> }`,
      so an upstream rule can say which revision it is. Same field as a cache's, so a
      cached file never carries the number twice.
- [x] **Found while building, fixed and pinned:** `lint` had called
      `recurrence_after_graduation` twice since 2026-08-16 and pushed both results, so
      every such finding printed twice. Invisible — this corpus holds no graduated rule
      that has recurred, and every test asked only whether the finding was *present*. The
      pin asserts the count, and was probed both ways.
- [x] **Also found:** the parser read `authority.version` for `kind = "local"` and threw
      it away. A copy's provenance under a local authority is now `UnexpectedField`
      rather than a silent drop.

Carried, not decided here:

- [!] **"New to you" means "absent here", and nothing else.** There is no record of what
      an install has already been shown, because keeping one would be the per-machine
      state invariant 3 forbids — so a contribution poke dismissed today returns
      tomorrow. It is the honest reading and it is also why that trigger is off by
      default; whether a *dismissal* is worth per-clone state (in the clone, as with the
      pseudonym) is a real question this phase did not open.
- [!] **An install's audience is whatever its own corpus declares, so an install that
      declares no scope matches every upstream rule.** `Rule::serves` reads an empty
      audience as "no filter", which is A1's safety default doing exactly what it should
      — and it means `--poke contributed` on an unscoped corpus is maximally loud. Loud
      rather than wrong, capped, and off by default. A `--scope` flag on `lint` would
      narrow it and was not added: it is a second way to say what an install is.
- [!] **The reactive trigger matches error-class text literally** (case-folded), through
      the one `ErrorClass::match_key` the linter also reads. Two rules describing one
      class in different words never meet. Nothing short of judgement closes that, and a
      fuzzy key dressed as certainty would be worse; the failure is under-matching, never
      a poke about an unrelated rule.
- [x] **Closed 2026-09-13: `--version` is required, and must supersede.** Was: neither required nor bumped. A rule contributed today
      publishes with no `version`, so no cache of it can ever be told it is stale — the
      stale-cache trigger is real and has nothing to read. Closing it is a new refusal in
      B2's one constructor, which is a change to a shipped phase. **George's call.**
- [!] **No rule in this corpus is a cache, and none is numbered**, so no trigger can fire
      against this repository's own library. Every behavioural test is synthetic, plus a
      hand-built clone run against the real 52 rules. The first real signal needs the
      second install — the question the programme says to answer before B1.
- [!] **The cap's default of 3 is still a judgement**, exposed as a flag and published in
      `BroadcastCap::DEFAULT`, but a number somebody chose. §6 asked for it to be
      settable rather than for a particular value; there is no config file to hold one
      and there must not be, so a flag with a default is as far as this goes.
- [!] **Nothing invokes `lint --upstream`.** FEATURES claims a command, and CI runs
      `lint` without a clone because there is no corpus to clone. The invocation half of
      that row is honestly absent, not quietly assumed.
- [x] **Two stray tracked files, `src/cli.rs.tmp` and `src/fsio.rs.tmp`** — removed
      2026-09-13, once measured rather than assumed: both were **zero bytes**, added by
      accident in `a1d7a23` and tracked since. Noticed at the start of the session and left
      alone twice, because deleting tracked files on a guess is not a thing to do in passing;
      measuring them first made it a one-line decision with nothing to lose.

## Phase C1 — the aggregate (shipped 2026-09-13)

A repository, not a service: `relearn aggregate` is the recompute a scheduled job runs over the
reports in a clone. Emission unchanged for all 52 rules.

- [x] The k-floor applied where the counts are, reading the **same constant** the producer does;
      below it a rule does not appear at all, tag included.
- [x] The floor counts distinct installs, so repetition cannot fake a population.
- [x] The suppressed **count** publishes; the suppressed **tags** do not.
- [x] Both confounds as a `const`, written unconditionally — including in an empty aggregate.
- [x] `no_emitter_or_build_path_reads_the_aggregate`, probed both ways. Its first version matched
      the English word and failed on `home_skill_aggregates_its_rules_sorted_by_tag`.
- [x] Strict parsing of untrusted reports: unknown schema, unknown bucket, malformed pseudonym.

Carried, not decided here:

- [x] **The scheduled job exists** — closed 2026-09-13. The aggregate repository was created
      that day and carries a monthly workflow that installs the tool at a pinned commit and
      commits the recompute. Pinned to a **commit** rather than a tag because the tool has no
      tags: a workflow pinned to a tag that does not exist is a control that cannot run, and it
      would have failed on its first scheduled attempt, silently, in a repository nobody was
      watching.
- [!] **An aggregate reveals its own population size** (`installs = N`) and the number of
      suppressed rules. Both are deliberate — a reader cannot weigh a figure without them — but
      they are exact counts, and the same argument that bucketed recurrences could be made about
      them at small N.
- [!] **Nothing verifies that a published `aggregate.toml` matches the reports beside it.** A
      committed aggregate could be hand-edited and the repository would not object; `relearn
      aggregate` recomputes, but nothing compares. The `verify`-shaped gate for the aggregate
      repository is C1-adjacent work that phase did not do.

## Phase B3 — `report` (shipped 2026-09-13)

Anonymous, always. Five fields per observation and a pseudonym that never touches the machine.
Emission unchanged for all 52 rules (`verify` green).

- [x] `Observation` **is** the field list (five fields, so a sixth cannot be added in a renderer)
      and `Month` has no day field, so no edit or formatting slip can reintroduce one.
- [x] The whole-document day-level-date scan, which is the failure mode the programme calls real
      and easy to miss.
- [x] Four reporting conditions, two of them earlier phases consuming themselves: A2's federation
      exclusion in its second flow, and B1's `Authority` answering "is this tag upstream?".
- [x] `Bucket` and `K_ANONYMITY_FLOOR` as published constants a reader can check.
- [x] `Control` sealed at five values — `gate:internal-payments-lint` publishes `gate`.
- [x] **Phase 0.2 resolved in code**: the pseudonym is the filename in the cloned aggregate
      repository, `--install` needed once, a conflicting id refused, several reports an error.

Carried, not decided here:

- [x] **Closed 2026-09-19: merged into `1-4`, and no published bucket denotes a single count.** Was: the `1` bucket publishes an exact count, and the programme warns against exactly that.
      §12.3 and §5 specify `1 | 2-4 | 5-9 | 10+`; the programme says "a bucket of 1–1 is not a
      bucket" and that boundaries leak exact counts at small n. The design wins where the two
      disagree, so `1` ships — but the k-floor protects the *aggregate*, while a raw report sits
      in public git history where `recurrences = "1"` is exact. Merging into `1-4` closes it and
      is a one-line change. **George's call.**
- [x] **Decided 2026-09-19.** Correct as it stands. The field is single-valued, and choosing one of two named kinds would invent a fact; reporting none is the honest failure. Widening it is the same semantics question as the partial-graduation item above and would be decided with it, not separately. Original entry: **Four of eight graduations in this corpus name two control kinds** (three when this was written 2026-09-16; re-measured 2026-09-19) (`test:… + gate:…`).
      The field is single-valued, so those report no control rather than a chosen one. Honest,
      and it drops the signal for 37% of graduated rules.
- [!] **The report carries no day-level date; the commit that publishes it does.** The command
      says so in its own output because no artefact here can prevent it.
- [!] **A contributed rule's recurrences are unreportable.** Reportability is read from
      `Authority`, so a rule you wrote, contributed upstream and still own locally stays `Local`
      and is never reported. Under-reporting is the safe direction — it publishes nothing — but
      it is a gap, and closing it needs a way to say "this local rule is also upstream".

## Phase B2 — `contribute` (shipped 2026-09-13)

The projection that cannot hold the raw incident, the matcher that runs where the human is, and a
command whose default writes nothing. Emission unchanged for all 52 rules (`verify` green).

- [x] `Contribution<'a>` borrows only publishable fields — `incident` and recurrences are absent
      from the struct, not removed by a renderer.
- [x] `PublishedIncident`, a distinct type, so the raw and the published cannot be swapped.
- [x] Four refusals at the one constructor, including **A2's federation exclusion consumed** and
      a new one the projection forced: a mandate carries an approver's name.
- [x] `scrub::TermList` — the matcher over one string, a finding that reports location and length
      and never the match, and `--terms` required so the disarmed state is unconstructible.
- [x] Two steps: print exactly what would leave, write only on `--confirm`. Writes a file,
      transmits nothing.

Carried, not decided here:

- [!] **`--terms` has no escape hatch for a contributor with nothing to protect.** A list must
      carry a salt and at least one term, so someone with no protected names must still author a
      list with a term they would never write. Deliberately strict — the alternative is an opt-out
      that becomes the default — but it is a sharp edge and B2 did not smooth it.
- [!] **Two implementations of one matcher.** The Rust one and `scripts/no-banned-names.sh` share
      a format, not code, because sharing code means spawning a process. Nothing checks that they
      agree: a cross-check would need the real term list, which is deliberately not committed.
      The format spec in both headers is the contract, and that is weaker than a test.
- [!] **The published incident is checked by a human and by a word matcher, and by nothing else.**
      No artefact reads for meaning. Written here so the next reader does not mistake the
      matcher's green for a judgement about the text.

## Phase B1 — `Authority`, and `adopt` (shipped 2026-09-13)

`Authority::Local | Adopted | Cached`, `relearn adopt`, and the refusal that makes a silent fork
impossible. Emission unchanged for all 52 rules (`verify` green, 63 files).

- [x] The refusal is **structural**: `fsio::write_rule` takes an `EditableRule` witness whose only
      constructor refuses `Cached`, with a compile-fail pin asserting that a bare `&Rule` does not
      compile, by its error.
- [x] `Version` is a monotonic `u32` — a **total order** for every pair, so `is_behind` always has
      an answer; trichotomy asserted explicitly.
- [x] Three authority variants, so an adopted rule remembers its fork rather than becoming
      indistinguishable from one authored here; adopting twice is refused.
- [x] `authority_never_reaches_an_emitted_path_or_body` — emitters treat a cache exactly like a
      local rule, which is the point of caching it.
- [x] `adopt --on <date>`: the date is given, never read from a clock, and `tests/solo_mode.rs`
      now forbids clock reads so the decision cannot erode.

Carried, not decided here:

- [x] **Closed 2026-09-13 by `pull`.** Was: nothing creates a cached rule. A cache arrives today only by hand-writing the
      `authority` table; pulling belongs to a later phase, and until it exists `is_behind` has no
      upstream to ask about. The type and the refusal are deliberately in place first — the guard
      that arrives after the thing it guards is the guard somebody has to remember.
- [x] **Decided 2026-09-19.** Correct as it stands. One tag is one file, the name is derived rather than chosen, and every rule this repository holds round-trips through it. A tag-body collision across installs is hypothetical and would be caught by the duplicate-tag parse error before any file was written. Original entry: **`write_rule` names the file from the tag body.** Correct for every rule this repository
      holds, and it means adopting a rule whose file was named differently would write a second
      file rather than rewriting the first. A pull that records its own filename would settle it.

## Phase A2 — the org layer and mandated content (shipped 2026-09-13)

`Home::Org { name }`, `Origin::Mandated(Approval)`, and the federation exclusion — established
**before** anything can federate, which is the order that matters. No network, no new command.
Emission unchanged for all 52 rules (`verify` green, 63 files); the lint summary is byte-identical
because the corpus holds no mandate.

- [x] `Home::Org { name }` with `OrgName`, and `Federation` / `Home::federation` as an exhaustive
      match with no catch-all — plus `Rule::is_publishable` reading that one answer.
- [x] The compile-fail pin (`E0004` naming `Org`, asserted by its error rather than by failing)
      and the source-level wildcard check, probed in both directions.
- [x] `Origin::Mandated(Approval)` with `approval = { by, date, control }` required when and only
      when mandated — both halves refusals, neither a warning.
- [x] Mandates held out of both counter-metric numbers through one predicate, with the hold-out
      and the evidential denominator reported.
- [x] `arb_home` and a new `arb_origin` generate the new variants, so **every existing property**
      covers them rather than each needing its own copy.

Decisions this phase was forced to take, carried for confirmation rather than left implicit:

- [x] **Decided 2026-09-19.** Correct as it stands. A project home carries a filesystem path, which is a private identifier — the same reasoning that withholds `Org`. Publishing one would leak the shape of a machine, and the exhaustive match on `Home::federation` is what forced the question to be answered rather than defaulted. Original entry: **`Project` is `Withheld` alongside `Org`.** The exhaustive match required an answer for
      every variant; a project home carries a filesystem path (a private identifier) and is
      meaningless upstream, so withholding is the conservative direction and costs nothing while
      nothing federates. **B2 should confirm it** — contributing a project rule would mean
      re-homing it first, which is an authored act rather than a transfer.
- [x] **Decided 2026-09-19.** Correct as it stands. An organisation's principles bind every language and every tree inside it, so `Org` follows `Global`: always-resident, reaching its readers through the layers that are. There is no file extension that means "this belongs to the company". Original entry: **An `Org` layer is not emitted into `.claude/rules/`.** It follows `Global`: always-resident
      and reaching its readers through the skill, so emitting it there too would duplicate one rule
      into two Claude files. That site is a `matches!` rather than an exhaustive match, so the
      compiler did **not** ask — it is now written as a match precisely so the next variant cannot
      slip through it. An org wanting a project-scoped file is a different feature.

## Phase A1 — home is not scope (shipped 2026-09-13)

The first piece of the federated design (`docs/federated-relearn.md` §2) and deliberately the
only one that pays for itself with **no federation at all**: `applies_to` on a rule, `--scope`
on `build` / `verify` / `list`, and a `ScopeTag` witness. Nothing here touches the network,
contributions, caches or reports. The emitted tree did not change for any of the 52 rules —
`relearn verify` green, 63 files — and no rule file needed editing.

- [x] `ScopeTag` newtype, parsed at the perimeter; `applies_to` on `Rule`; `Rule::serves` holding
      both safety defaults where no caller can reimplement them.
- [x] `--scope` on `build`, `verify` and `list`, narrowing through one shared
      `restrict_to_scopes` so a build and its paired gate cannot disagree.
- [x] An unknown scope is a loud error for `build`/`verify`, an empty listing for `list`.
- [x] Enforcers shipped in the same change: `tests/scope_filter.rs`, four new property tests,
      parse/serialize unit pins, four `cli` tests. FEATURES rows and the decisions-log entry
      recording **why nesting was rejected**.

Discovered while building it, deliberately not decided here:

- [x] **Decided 2026-09-19.** Correct as it stands, and it is the same decision as the two above. `emit::claude_rules` emits project homes and known language domains only, because a domain with no glob table cannot be path-scoped and the alternative is always-resident. Four of five targets is the right answer, not a gap. Original entry: **A rule homed in `domain-low-latency` reaches four of the five targets — measured
      2026-09-14, no longer a prediction.** **Fifteen** such rules now exist (two when measured; re-counted 2026-09-19), and they land in
      `skills/domain-low-latency/SKILL.md`, fifteen `.cursor/rules/*.mdc` files, `.github/copilot-instructions.md`
      and `AGENTS.md` — and **not** in `.claude/rules/`. The motivating example of A1 is absent
      from exactly one layer, which is the decision below, now with a real rule behind it.
      `emit::claude_rules` emits project homes and *known language domains* only — a domain with
      no glob table is `LoadSemantics::OnRequest` and is skipped — so the motivating example of
      this very phase is absent from `.claude/rules/`. Skills, Cursor, Copilot and `AGENTS.md`
      all carry it. Either `low-latency` earns a glob set, or the rules layer needs a story for
      an always-loaded non-language domain. Not a regression: this predates scoping.
- [x] **A controlled scope vocabulary — answered without a controller, 2026-09-14.** `applies_to`
      is free strings bounded in shape, the corpus is the vocabulary, and `ScopeNearDuplicate`
      catches the drift a curator would have. Built once a second scope existed, because a drift
      detector over an empty vocabulary arrives already green and an artefact that cannot fail is
      one nobody notices is broken.
- [x] **Should `lint` flag a scope used by exactly one rule? — No, and the answer is in the
      finding.** A scope used once is flagged *only* when an established spelling sits a slip
      away from it. Flagging every singleton would fire on the first rule of every genuinely new
      audience, which is the ordinary way a vocabulary grows; and when two scopes are each used
      once there is no established spelling to have drifted from, so deciding which was intended
      is a judgement the check refuses to make. Pinned by
      `two_scopes_each_used_once_are_not_a_near_duplicate`.
- [x] **Decided 2026-09-19.** No. Scope is a build-time filter and the packs are already built per home, so a scoped variant would multiply the committed tree by the scope set to express something `--scope` already expresses at the moment of use. Original entry: **Should `copilot-pack/` and `claude-pack/` gain scoped variants?** They are built per home
      today and keep working untouched, because unscoped rules always emit. Changing nothing for
      now.
- [x] **Answered 2026-09-14, YES — a scoped rule announces its audience.** `> Written for the rust
      and java audiences.`, the recurrence-note shape exactly as this entry anticipated: third
      member of the `graduation_note` / `recurrence_note` family, `None` for an unscoped rule so
      not one byte moved for the other 52, and spliced **first** of the three because *is this
      mine* is the reader's first question. The byte-identity property was **split, not deleted**
      — `scope_never_reaches_an_emitted_path` keeps the load-bearing half, and
      `scope_reaches_a_body_only_through_the_audience_note` replaces the body half with a stronger
      claim: delete the note from a scoped build and the remainder is byte-identical to an
      unscoped one, so the announcement is provably the whole of the difference. Four emitted
      files changed for the two low-latency rules; `claude_rules` is unaffected because
      `low-latency` is not a known language domain, which is the gap already carried below.
- [x] **Answered 2026-09-14: two rules now declare one.** Reported in the A1 summary, **none applied**: deciding
      a rule's audience is a judgement about who is harmed by not seeing it, and that is George's.

## Phase J — federated relearn (design v2 filed 2026-09-13; **built A1 → C2 the same day**, two items open)

The design is [`docs/federated-relearn.md`](docs/federated-relearn.md). **v2 supersedes the v1
filed the same day** (then `docs/federated-recurrence.md`, in commit 522679f). v1's thesis was
*federate the signal, not the corpus* — a class catalogue and counts, no rule text ever. v2 lets
rules travel and moves the privacy problem to **contribution time, where a human is present**:
one home, many regenerable caches, and three flows with three different privacy models —
contribution **attributed**, cache carrying its author, recurrence report **anonymous, always**.
It is still the cross-install answer to the question Phase F answered locally, and still the only
route on the table to the n = 1 objection and to measuring the decay curve Paper 3 declines to
claim.

*Reconciled 2026-09-13, after C2.* **This section read "nothing built" while seven phases had
built it**, and every item below was marked `(Blocked)` — a ledger contradicted by its own tree,
in the file that grades this repository. It was filed beside `docs/recurrence-session-hook.md` as
*written down so it is not re-invented, deliberately not implemented*, and then the programme
implemented it without coming back here: a design document that becomes a plan, and a plan nobody
re-reads because its heading still says nothing was built. What each item became is now recorded
against the phase that delivered it; those phase sections are above and are not restated here.
**Two items were never built, and each says why.**

- [x] **Decide the six open questions (§12).** Answered 2026-09-13, the same day they were filed;
      §12 records each decision with its why, and the two that moved the design carried edits into
      §5 and §8.
  - [x] The catalogue stays dropped — a second identity space means per-install, per-rule entity
        resolution with no error-correcting feedback, and it was the sole source of v1's own
        hardest question. The real cost is a **biased** under-count, not a sparse one, and it is
        paid back by `adopt` (§5) and declared in the aggregate (§8).
  - [x] The contributor owns the scrub, mechanically, on their own machine; review checks only
        what a reviewer can see — duplication against the corpus, origin, parse. One maintainer
        plus a published checklist until there is volume.
  - [x] *k* = 5 (the standard cell-suppression floor, a benchmark a reader can look up) **and**
        counts publish as buckets — an exact count plus a month defeats pseudonym rotation.
  - [x] `control` kind is published, on the structural condition that it is a sealed enum with
        no free-text variant and no `Other(String)`. Never the control's name, hashed or not.
  - [x] The corpus is the scope vocabulary — one identity space, as in the catalogue answer —
        with a `ScopeNearDuplicate` finding instead of a curator. Scopes are audiences, not topics.
  - [x] A local attic suppresses, an upstream attic only warns; no new `Status` variant, because
        rule-replaces-rule is a version move on the same tag.
- [x] **Solo mode is gated before any federated line of code exists (§1).** `tests/solo_mode.rs`:
      no socket, no spawned process, no ambient state, no networking/TLS/async-runtime crate in
      the tree. It held through all seven phases, and every one of them had a standing reason to
      breach it. The two invariants it does *not* cover were closed by A1 and by the round trip:
  - [x] **No configuration means no filtering.** Closed by A1, **not** as designed: there is no
        `Audience::Everything` type, because `Rule::serves` reads an empty audience as "no
        filter" and a rule declaring no `applies_to` is emitted under every audience. A config
        file was then refused outright by programme Phase 0.1 rather than deferred, so the
        invariant is held by the absence of the thing it was guarding against.
        `tests/properties.rs::narrowing_never_touches_an_unscoped_rule` is the whole-space form.
  - [x] **Every federated field optional, absence being today's behaviour exactly.** Five fields
        arrived across the seven phases — `recurrence`, `applies_to`, `approval`, `authority`,
        `published_incident` — and not one rule file needed editing. Proved by
        `tests/corpus.rs::every_committed_rule_round_trips_byte_identically` over the real
        fifty-two, which is the round-trip assertion §13 asks for rather than the intent.
- [x] `applies_to` — **A1** (`3798521`). An audience, never a second home; absent means every
      audience, so adding a scope to one rule can never remove a different rule from a build.
- [x] `Home::Org { name }` and the structural exclusion — **A2**. `Home::federation` decides
      every variant with no catch-all arm, so a home added without deciding its federation
      behaviour is a compile error. `Project` is withheld too: its path is a private identifier.
- [x] `Authority::Local | Adopted | Cached` — **B1**. Three variants, not two: a cache that
      "became local" on adoption would be indistinguishable from a rule authored here. The edit
      path refuses `Cached` through the `EditableRule` witness, with a compile-fail pin.
- [x] `relearn adopt <tag>` — **B1**. The deliberate fork, with recorded provenance; refused for
      anything that is not a cache, and refused again on an already-adopted rule.
- [x] `Origin::Mandated` with an `approval` table, and mandates held out of every recurrence
      statistic — **A2**. No regulatory-alignment claim ships, with or without a signer.
- [x] `relearn contribute` — **B2**. `Contribution` is a projection that never *borrows* the raw
      incident or the recurrences, so no renderer can leak them. Prints what would leave, writes
      only on `--confirm`, transmits nothing.
  - [x] The scrub runs **here, not at review** (§12.2): `scrub::TermList` over the published
        incident **and the body**, reporting a location and a length and never the match.
        `--terms` is a required argument, so "ran with no list" is unconstructible rather than an
        exit code to remember — stronger than the script's exit 2.
- [x] `relearn report` — **B3**. Writes a file, publishes nothing. `Observation` **is** the field
      list, and `Month` has no day, so neither a sixth field nor a day-level date can be
      reintroduced by an edit. Only upstream tags are reportable.
  - [x] Counts serialize as **buckets**, `Control` is sealed at five values, and `k = 5` is a
        published constant both ends read — the producer and the aggregate, so neither can
        quietly lower it.
- [x] The poke, surfaced in `relearn lint` — **C2**. Reactive on and uncapped; broadcast capped
      by a number the operator passes (`--poke-cap`, not a config file — §6 amended in place),
      and two of the three off until named. It carries no severity and cannot change a verdict.
- [x] **Counter-metrics, not optional (§8) — and there are two.** Cross-install recurrence
      measures frequency *and* diligence, inseparably; and with the catalogue gone the aggregate
      counts only **classes somebody published a rule for**, a bias toward the cheap incidents
      rather than mere sparseness. Both sentences are a `const` written unconditionally by
      `Aggregate::to_toml`, including in an empty aggregate — not a flag, because the way they
      get dropped is by being droppable. The mined fraction is the counter-metric to the
      federation itself, and `lint::Tally` reports the evidential denominator beside it.
- [x] **`CachedRuleRetiredUpstream` (§12.6) — shipped 2026-09-13 as the fifth poke**, on
      George's call. §12.6 specifies a lint `Warning`, written before the poke existed; a
      `Warning` reachable only with `--upstream` is federation failing a run at the default
      `--deny warning`, which is the invariant C2 had just made structural. So it sits beside
      `cache-behind` as a broadcast poke, on by default, and cannot fire at all against an
      install that caches nothing. Everything §12.6 is *about* survives: an upstream attic only
      warns, the three resolutions are named with `adopt` first, and the suppression is
      **unreachable rather than declined** — only a local `Status` reaches
      `Status::emittability`, asserted over the real emitters. The deviation is recorded in
      §12.6 itself, in the programme's C2 section and in the decisions log, so the design does
      not go on specifying a shape the code deliberately does not have.
  - [ ] **The trigger has never fired against this corpus and cannot**: no rule here is a cache,
        and the clone it would read does not exist. Observed through the binary on a hand-built
        clone with a hand-added cached rule, then both removed. Same standing gap as the other
        four triggers, and the same second install closes it.

### Not built — the two, and why

- [x] **`ScopeNearDuplicate` (§12.5) — shipped 2026-09-14**, once the two low-latency rules
      gave the corpus a vocabulary and the detector could therefore fail. §12.5 specifies edit
      distance 2 and that alone is wrong in a way its own examples show: `rust` and `ruby` are
      two edits apart and are two languages, and any two two-letter scopes are two apart by
      arithmetic. Distance is read relative to length, and the deviation is recorded in §12.5
      itself. Levenshtein is twenty lines here rather than a dependency needing a price.
- [!] **A published report is not research consent.** A separate, recorded opt-in, and nothing
      records one. No report has been published, so nothing is exposed today — but the first one
      would be, and the consent is not a field, a flag, or a document yet.

## Phase C — instrumentation (parallel; lives in stochos-lab, not here)
- [x] Error-class recurrence — ~~partly exists in the ledger~~ **now modelled in the library itself** (Phase F, 2026-09-06): `[[recurrence]]` tables on the rule, an `UnheldRecurrence` lint finding, and an annotation in every emitted format. The stochos-lab ledger remains the place where recurrences are *noticed*; `rules/` is now the place they are *recorded*. What is still open there is the counter-metric (`origin`) and the graduation-date question, both carried under Phase F.
- [!] First-time-right capture on AI-assisted work
- [!] **Cold-surface / uncited report** (moved from Phase B): flag rules that runtime data shows are never invoked — candidates for the attic cut-list. Needs skill-invocation / hook-fire counts from the stochos-lab observability layers; once that feed exists, the report itself can live either here or as a relearn lint check fed by an exported dataset.
- [!] *(Deferred, needs a field site: rework rate, time-to-competence — study-design items, not build items)*

## Resolved decisions (2026-08-13 — see ARCHITECTURE decisions log)
- [x] Claude skill emitter: **one skill per home layer**, not per rule. Bounds the always-resident metadata index (P6); matches the existing `project-discipline`/`rust-typedd` skills that bundle rules by domain. Future escape hatch: an optional `skill_group` field *only if* one home ever needs more than one skill.
- [x] Cursor `globs`/`alwaysApply`: **derived from `Home`** via a shared domain→pattern table in `emit`; no `cursor.*` fields on the rule (keeps the format neutral, P2). If finer scope than `Home` expresses is ever needed, enrich `Home` so every emitter benefits — never a Cursor-only field.
- [x] **Emit-status filtering: suppress `attic`, emit `active` + `graduated` (annotated).** Policy is one exhaustive match (`Status::emittability`) applied once (`emit::emittable`); annotation via `emit::graduation_note`. Graduated kept because instruction-layer tuning (before generation) is a control distinct from the graduated-to hook (catches after); attic suppressed because withdrawn guidance must never enter an active instruction file. Whole-space property tests + unit pins; FEATURES row "Emission respects rule status".
