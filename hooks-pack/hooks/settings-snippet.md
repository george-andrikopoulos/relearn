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
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-public-repo-create.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-serializable-domain-type.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-string-identity-compare.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-tool-attribution.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-unwrap-in-src.sh" }
        ]
      }
    ]
  }
}
```

<!-- relearn:generated v0.1.0 sha256=591c2cb53efce8c3e40d04f161d53fcbd93a169d307910d0e017191ac07aac83 rules=R:case-collision,R:identity-is-not-equality-for-boxes,R:no-anyhow-in-libraries,R:no-tool-attribution-in-public-artefacts,R:no-unwrap-in-production,R:no-weak-model-for-judgment,R:repository-private-by-default,R:revision-integrity,R:serializable-is-a-second-constructor -- DO NOT EDIT; regenerate with `relearn build` -->
