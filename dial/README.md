# Amni Dial
Every agent session on one giant dial: a GPU coding console built in Helium graphics and written in Gold (`dial.gold` builds `dial.amni`).
- Polar sectors: Grok Bot, Claude Code, Antigravity, OpenCode. Each session is a wedge; the active one fills the bottom of the screen.
- New output enters at the rim and crawls up into a lensed black hole (`shaders/hole.frag`): it shrinks, redshifts, spirals and falls past the event horizon.
- Markdown renders as a console: headings, bullets, rules, tables with aligned columns, fenced code on a code plate, inline `code` and **bold**. Tool calls show their target (file, command, pattern), failed tool results show in red, and each finished turn shows its time, cost and turn count.
- HDR scene with bloom, chromatic edges, vignette and film grain (`shaders/post.frag`). Dark space theme by default; Ctrl+T switches to ivory paper.
- Left/Right or the side arrows: next session in that platform. 1-4 or the tabs: spin to a platform. `+` or Ctrl+N: new session. Wheel/Up/Down: scroll back. Ctrl+V: paste. Enter: send.
## Backends
| sector | adapter |
| --- | --- |
| Claude Code | `claude -p --output-format stream-json --verbose --permission-mode acceptEdits`, `--resume` per session |
| OpenCode | `opencode run --format json`, `-s` per session, model from `AMNI_DIAL_OPENCODE_MODEL` |
| Antigravity | `agy --print --output-format json`, `--conversation` per session |
| Grok Bot | Grok Bot has no public API; the sector drives the `grok` CLI when it is installed |
Sessions live in `~/.amni-dial/<platform>/<n>/` (`AMNI_DIAL_HOME` overrides). Agents run detached, so they keep working after the dial closes, in the directory you start the dial from.
## Run
```
dial/build.sh
amni-dial
amni-dial --ivory
amni-dial --send 2 "summarize the repo"
amni-dial --shots out
amni-dial --shots out --frames
```
`--shots` renders stills into the folder; `--frames` also writes every animation frame to `out/frames`.
## Edit
Change `dial.gold`, then `gold build dial.gold` and `dial/build.sh`. Commit `dial.gold`, `dial.gold.lock` and the generated `dial.amni` together.
