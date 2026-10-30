# Registering the generated hooks

Copy every `*.sh` in this directory except `test-detectors.sh` into `~/.claude/hooks/relearn/`, then merge the entries below into the `hooks` table of `~/.claude/settings.json`. relearn installs nothing: these files edit the layer every session on the machine loads, and no gate in this repository can see that layer.

Where a hand-written hook with the same name is registered today, replace its entry with the one below rather than adding a second: one hook per detector, so a block is never reported twice and a retired one is never left running.

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write",
        "hooks": [
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-case-collision.sh" }
        ]
      },
      {
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-anyhow-in-lib.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-count-in-heading.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-embedded-local-llm.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-expect-empty-msg.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-unwrap-in-src.sh" }
        ]
      }
    ]
  }
}
```

<!-- relearn:generated v0.1.0 sha256=5b60742768baf8c2a855d9b0b553e0cf343b4ff14b5fbaca74aead96d421b6b5 rules=R:case-collision,R:no-anyhow-in-libraries,R:no-unwrap-in-production,R:no-weak-model-for-judgment,R:revision-integrity -- DO NOT EDIT; regenerate with `relearn build` -->
