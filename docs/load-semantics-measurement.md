# How the `.claude/rules/` load model was measured (2026-08-22)

The `LoadSemantics` decision in `ARCHITECTURE.md` rests on a measurement, not on
documentation: **`paths: []` loads at `session_start`** — an empty glob list means
*always*, not *never*. This note records how that was established so the claim can
be re-checked rather than taken on trust. The rig itself was a throwaway directory,
deleted once the result was recorded.

## Why it needed measuring

Anthropic's docs describe `paths:` front-matter and say rules without it load
unconditionally. They do not say what an **empty** list does. The two readings are
opposites — "matches nothing, so never loads" versus "no constraint, so always
loads" — and the emitter had to pick one. Guessing would have decided, silently,
whether a rule meant to be reachable by description became one resident in every
session on the machine.

## The instrument

Claude Code exposes an `InstructionsLoaded` hook that fires per instruction file.
In a throwaway git repo containing only `index.html`, `style.css` and `app.tsx`:

`.claude/settings.json` — registered on all five documented matchers
(`session_start`, `path_glob_match`, `nested_traversal`, `include`, `compact`)
rather than a `".*"` wildcard, so an empty log could not be a false negative from
an unhonoured regex:

```json
{ "hooks": { "InstructionsLoaded": [
  { "matcher": "session_start", "hooks": [
    { "type": "command",
      "command": "powershell -NoProfile -ExecutionPolicy Bypass -File .claude/log-instructions.ps1" } ] }
] } }
```

`.claude/log-instructions.ps1` — writes **one file per event** into `loaded.d/`:

```powershell
$json = [Console]::In.ReadToEnd()
$dir = 'loaded.d'
if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
Set-Content -Path (Join-Path $dir ([guid]::NewGuid().Guid + '.json')) -Value $json -Encoding utf8
```

One file per event is not fastidiousness. The first version appended to a single
log and **captured 3 of 18 events** — ~18 hooks fire concurrently at session start
and `Add-Content` raced, losing writes. Read as-is it would have reported most
rules as not loading.

Each run: `claude -p "<prompt>" --permission-mode acceptEdits`, then read `loaded.d/`.

## Sentinels

Each rule file carried a fresh UUID so nothing else could produce the string. The
pair matters: one file alone cannot distinguish *"`paths:` scoping works"* from
*"everything in a rules directory loads regardless"*.

| Sentinel | Front-matter | Result |
|---|---|---|
| match | `paths: ["**/*.html"]` | loaded, `path_glob_match`, trigger `index.html` |
| nomatch | `paths: ["**/*.nonexistent-xyz"]` | never loaded |
| real subdirectory | `paths: ["**/*.html"]`, in `rules/realsub/` | loaded — recursion works |
| junction | `paths: ["**/*.html"]`, via a Windows directory junction | **never loaded** |
| empty | `paths: []` | **loaded, `session_start`** |

## Results

The payload field is **`load_reason`**, not the documented `reason`, and carries
undocumented `memory_type`, `globs[]` and `trigger_file_path`. Writing the hook
against the documented field name would have logged nothing useful.

1. **`paths: []` loads at `session_start`.** The empty list is unconditional. This
   is the finding `Globs`' non-empty-by-construction invariant exists to respect.
2. **The load model is two-state**: front-matter absent → always; `paths: [globs]`
   → on reading a match. There is no spelling for "reachable by description only",
   which is why `emit::claude_rules` skips `LoadSemantics::OnRequest` rather than
   writing it unscoped.
3. **User-scope `paths:` works.** `~/.claude/rules/ecc/rust/*.md` all fired with
   `memory_type: "User"`, `load_reason: "path_glob_match"` on reading a `.rs`
   file. A prior belief that user-level scoped rules never load was refuted; the
   `/context` readings behind it were taken before any matching file had been read,
   which is the documented lazy-load behaviour.
4. **Windows directory junctions are not traversed** by rules discovery, in a run
   where a matching file *was* read and both sibling rules fired.

## One confound worth knowing

The first junction sentinel was written with PowerShell 5.1
`Set-Content -Encoding utf8`, which prepends a UTF-8 BOM ahead of the `---`. That
alone can break front-matter parsing. The sentinel was rewritten BOM-free and the
negative held. **Write sentinels with a BOM-free editor**, or a null result means
nothing.
