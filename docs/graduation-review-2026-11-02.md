# Graduation review — 2026-11-02

*A dated snapshot for one decision session. It states its own numbers; nothing else quotes
them. The decisions are George's; this sheet carries the evidence and a recommendation per
row. Answer per row — **accept**, **alternative**, or **hold** — or "accept all" with the
exceptions named.*

## What accepting a row costs

A status names a control, and `relearn controls` then checks that the control is wired. So
accepting is never only an edit to `status`:

- **A hook draft:**
  - copy `hooks-pack/hooks/<name>.sh` into `~/.claude/hooks/relearn/`;
  - register it in `settings.json` and in stochos-lab's `settings.hooks.json`, with a
    `test-hooks.sh` line;
  - set its `Install` cell in `hooks-pack/README.md` to `yes`;
  - add a `hook:<name>` entry to `controls.toml`;
  - set the rule to `partial`, naming the hook and the uncovered text below.

  A hook reaches Claude Code sessions on this machine only, so a hook-held rule is
  configuration-dependent, and its FEATURES row says so.
- **A repository gate:** add a `gate:<name>` entry to `controls.toml`, and an invoker that
  runs it (a `check.sh` stage or a repository's own hook). Until something invokes it, the
  resolver reads it as not wired.
- **An ADOPT-EXISTING proposal:** move its `controls.toml` entry from `[[proposed]]` to
  `[[control]]`, then set the rule to `partial`. The control is already wired.

No row below recommends `graduated`. None of these controls claims a whole error class.

## How the evidence was measured

- **Fixtures:** the detector's own cases, all passing in `relearn check` and through
  `hooks-pack/hooks/test-detectors.sh`, on Windows and, since today, on Linux.
- **Real code:** every tracked file in four local repositories (relearn, stochos-lab,
  mesh-watchdog, win-health-mcp). Each file was prefiltered with the rule's own pattern, then
  passed whole through the **generated hook script** as a `Write` payload, which is the
  production path. Each pattern found candidate files, so a zero means the hook judged them
  clean, not that it saw nothing.
- **Caveat:** none of the four repositories holds Java, so the two Java detectors have
  fixture evidence only.

## 1. Hook drafts

| Detector | Rule | Action | Fixtures | Real code: block / warn | Recommendation |
|---|---|---|---|---|---|
| `no-embedded-local-llm` | `[R:no-weak-model-for-judgment]` | warn | 6 | 0 / 0 (12 candidates) | **Install; `partial`.** Uncovered: routing judgment to a cheaper hosted tier, and local runtimes the pattern does not name. |
| `no-count-in-heading` | `[R:revision-integrity]` | warn | 8 | 0 / 4 | **Install as `warn`; `partial`.** Of the four headings, one or two are the class (*Three things to know*); *The five files* is a definition and *Two rules written…* is history. `warn` is the right grade, and `block` would be wrong. Uncovered: antecedents, cross-references, counts in body text, terms used before definition. |
| `no-tool-attribution` | `[R:no-tool-attribution-in-public-artefacts]` | block | 7 | 0 / 0 (3 candidates) | **Install; `partial`.** It also has a `git-pre-commit` event, so the commit dispatcher can carry it into a repository. Uncovered: trailers passed inline to `git commit`, the tool's own setting, document metadata, history already pushed. |
| `no-public-repo-create` | `[R:repository-private-by-default]` | block | 7 | 0 / 0 (6 candidates) | **Install; `partial`.** Uncovered: a create at a shell prompt, through the web UI or through the REST API. Note that the hook reads a *written file*: a `gh repo create --public` typed as a Bash command is not in its scope. |
| `no-string-identity-compare` | `[R:identity-is-not-equality-for-boxes]` | block | 6 | no Java to test | **Hold** until a Java repository is to hand, as for the deferred Java linter (Q5). Fixture-only evidence for a `block` is thin. |
| `no-serializable-domain-type` | `[R:serializable-is-a-second-constructor]` | block | 5 | no Java to test | **Hold**, same reason. |
| `no-string-error` | `[R:errors-name-what-failed]` | block | 8 | **1** / 0 | **Fix first, then install.** It blocks `OriginError::Unknown(String)` in `src/rule/origin.rs`, a correct variant that carries the unrecognised value and prints it, which is what the rule asks for. The `(Other\|Unknown\|Generic\|Misc)(String)` arm cannot tell a catch-all from a variant naming its value. Alternatives: drop `Unknown` from the arm, or ship it as `warn`. Uncovered: field-less variants, and variants omitting the identifying value. |
| `no-let-underscore-local` | `[R:no-let-underscore-on-a-local]` | block | 6 | 0 / 0 (12 candidates) | **Install; `partial`.** Uncovered: underscore-prefixed bindings. |
| `no-free-validator` | `[R:parse-dont-validate]` | block | 6 | 0 / 0 (4 candidates) | **Install; `partial`.** Uncovered: re-checking parsed data at interior call sites. |
| `no-constructed-target-path` | `[R:xplat-fixtures]` | block | 5 | 0 / 0 (6 candidates) | **Install; `partial`.** Uncovered: uncanonicalised path comparison, stray CRs in tool output, interpreter stubs. |

## 2. Repository gates and the ledger detector, not yet in TODO.md

`relearn lint` lists these as graduation candidates too: each rule carries a control whose
fixtures pass, and its status does not name it. In the "real repositories" column, *clean*
means exit 0 on all four repositories.

| Control | Rule | Runs where today | Real repositories | Recommendation |
|---|---|---|---|---|
| `gate:executable-text-eol-pinned` | `[R:pin-eol-for-executable-text]` | relearn's `check.sh`, over relearn | clean | **`partial`.** It is the only gate here that is already invoked. Uncovered: executable text with no shebang outside the declared globs, and every repository that does not run the gate. |
| `gate:five-root-documents` | `[R:five-files-no-more]` | fixtures only | clean | **Add a `check.sh` stage over relearn, then `partial`.** Uncovered: documents sprawling below the root, non-`.md` documents, and repositories without the gate. |
| `gate:one-home-paragraph` (warn) | `[R:one-home-per-rule]` | fixtures only | relearn 7 lines of warnings, win-health-mcp 3 | **Hold.** Triage today's warnings first. Even wired, it finds only verbatim copies, and paraphrase is where the rule says second homes hide, so `partial` would claim little. |
| `gate:pass-by-value-lint-on` | `[R:borrow-in-signatures]` | fixtures only | **fails all four**, stochos-lab included, which holds no Rust | **Hold.** No repository enables `needless_pass_by_value` (relearn included, the open *dogfood `lint-enabled` here?* item), and the gate reports a finding where it cannot apply (new TODO item). Decide the dogfooding first. |
| `hook:` + `gate:ledger-entry-names-artefact` (sections, level 3) | `[R:features-ledger-names-its-artefact]` | not installed | not measured here | **Hold** with the item already deferred to this review: at level 2 the pilot flags stochos-lab's group headings, not its entries. Decide which ledger shape it is for before installing. |
| `gate:listed-file-staged` | `[R:no-secrets-in-config-repo]` | **live** in stochos-lab's `pre-commit` (commit `455864b`), with the list configured | — | **`partial`**, after the `controls.toml` entry; check with `relearn controls --root stochos-lab=…` that it resolves as wired. Uncovered: a secret inside a tracked, whitelisted file, a secret-bearing file not on the list, and `--no-verify`. |

## 3. ADOPT-EXISTING proposals

All four resolve as `RESOLVED_AND_WIRED` today, through `relearn controls` with stochos-lab
as a root. The `uncovered` text is the disposition table's, in ARCHITECTURE.md.

| Rule | Control | Recommendation |
|---|---|---|
| `[R:price-every-dependency]` | `test:tests/dependencies.rs` | **Accept.** Uncovered: every repository but this one, and a refused dependency, which leaves no artefact. |
| `[R:employer-identity-not-in-public-artefacts]` | `gate:scripts/no-banned-names.sh` | **Accept.** Uncovered: role titles and names not on the list, and any repository without the gate. |
| `[R:names-travel-with-the-quote]` | `test:…::every_authored_field_that_travels_is_offered_to_the_scan` | **Accept.** Uncovered: identifiers not on the list. |
| `[R:reconcile-wiring-at-start]` | `hook:session-wiring-check` (stochos-lab) | **Accept.** Uncovered: the drill that proves the alarm fires, and any machine but this one. |

Not proposed: `[R:verify-tracked-after-move]`. Its control, stochos-lab's `verify-deploy.sh`,
is still not wired. stochos-lab's `pre-push` leaves it out on purpose, and its comment says
why, so the rule stays `active`.

## If every recommendation is accepted

- **Install seven hooks** and set their rules to `partial`: `no-embedded-local-llm`,
  `no-count-in-heading`, `no-tool-attribution`, `no-public-repo-create`,
  `no-let-underscore-local`, `no-free-validator` and `no-constructed-target-path`.
- **Fix `no-string-error`'s pattern**, then install it.
- **Set three rules to `partial`:** `pin-eol-for-executable-text` and `no-secrets-in-config-repo`
  now, and `five-files-no-more` once its gate has a `check.sh` stage.
- **Move the four proposals** to `partial`.
- **Hold five:** the two Java detectors, `one-home-paragraph`, `pass-by-value-lint-on`, and
  the ledger detector.
