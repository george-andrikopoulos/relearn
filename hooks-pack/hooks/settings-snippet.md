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
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-constructed-target-path.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-count-in-heading.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-embedded-local-llm.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-expect-empty-msg.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-free-validator.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-let-underscore-local.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-public-repo-create.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-serializable-domain-type.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-string-error.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-string-identity-compare.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-tool-attribution.sh" },
          { "type": "command", "command": "bash ~/.claude/hooks/relearn/no-unwrap-in-src.sh" }
        ]
      }
    ]
  }
}
```

<!-- relearn:generated v0.1.0 sha256=8df2cc0e7454b99509e1e098191e4ea0e697d785de5bfe43ee7a5404191cbf73 rules=R:case-collision,R:errors-name-what-failed,R:identity-is-not-equality-for-boxes,R:no-anyhow-in-libraries,R:no-let-underscore-on-a-local,R:no-tool-attribution-in-public-artefacts,R:no-unwrap-in-production,R:no-weak-model-for-judgment,R:parse-dont-validate,R:repository-private-by-default,R:revision-integrity,R:serializable-is-a-second-constructor,R:xplat-fixtures -- DO NOT EDIT; regenerate with `relearn build` -->
