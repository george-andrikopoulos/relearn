# Registering the load log

Copy `relearn-load-log.sh` into `~/.claude/hooks/relearn/`, then merge the entries below into the `hooks` table of `~/.claude/settings.json`. relearn installs nothing: once registered, this hook writes a line on every skill and instruction-file load on the machine, to `$RELEARN_LOAD_LOG` or `~/.local/state/relearn/load-log.jsonl`.

`InstructionsLoaded` takes no matcher, so it sees every `CLAUDE.md` and rules file; `relearn loads` keeps only relearn's own homes.

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Skill",
        "hooks": [
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/relearn-load-log.sh" }
        ]
      }
    ],
    "InstructionsLoaded": [
      {
        "hooks": [
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/relearn-load-log.sh" }
        ]
      }
    ]
  }
}
```

<!-- relearn:generated v0.1.0 sha256=415747b27d75624de2de6f3130538095b1ecf2b002deddf4d4c7c8d26261e174 rules= -- DO NOT EDIT; regenerate with `relearn build` -->
