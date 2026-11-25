# Amni Dial
Every agent session on one giant dial, built in Helium graphics.
- Polar sectors: Grok Bot, Claude Code, Antigravity, OpenCode. Each session is a wedge.
- The active session sits at the top. New output enters at the rim and crawls toward the hub, shrinking and fading.
- Left/Right or the side arrows: next session in that platform. 1-4 or the chips: spin to a platform. `+` or Ctrl+N: new session. Ctrl+T: navy theme. Wheel/Up/Down: scroll back. Enter: send.
## Backends
| sector | adapter |
| --- | --- |
| Claude Code | `claude -p --output-format stream-json --verbose --permission-mode acceptEdits`, `--resume` per session |
| OpenCode | `opencode run --format json`, `-s` per session, model from `AMNI_DIAL_OPENCODE_MODEL` |
| Antigravity | `agy --print --output-format json`, `--conversation` per session |
| Grok Bot | Grok Bot has no public API; the sector drives the `grok` CLI when it is installed |
Sessions live in `~/.amni-dial/<platform>/<n>/` (`AMNI_DIAL_HOME` overrides). Agents run in the directory you start the dial from.
## Run
```
dial/build.sh
amni-dial
amni dial
amni-dial --shots out
```
