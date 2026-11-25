# Amni Dial
Every agent session on one giant dial: a GPU coding console built in Helium graphics and written in Gold (`dial.gold` builds `dial.amni`).
- Polar sectors: Grok Bot, Claude Code, Antigravity, OpenCode. Each session is a wedge; the active one fills the bottom of the screen.
- New output enters at the rim and crawls up into a lensed black hole (`shaders/hole.frag`): it shrinks, redshifts, spirals and falls past the event horizon.
- Markdown renders as a console: three heading levels, nested bullets, numbered and task lists, quotes, rules, tables with aligned columns, fenced code on a code plate, inline `code` pills, **bold**, *italic*, ~~strikethrough~~ and links. Math in `$…$`, `$$…$$`, `\(…\)` and `\[…\]` is converted from LaTeX to Unicode (Greek, operators, fractions and roots) and tinted; exponents and indices use Unicode super- and subscripts when they can and otherwise draw as smaller raised or lowered runs. Text uses the system TrueType font, with DejaVu Sans filling in missing glyphs.
- Tool calls show their target (file, command, pattern), failed tool results show in red, thinking shows as a dim italic line, and each finished turn shows its time, cost and turn count.
- Indicators: while an agent works, the prompt track and HUD show a spinner with what it is doing (thinking, running a tool, writing) and the elapsed time. HUD chips show the model, the context used against the model's window (also drawn as a meter on the rim), spend, turns, output tokens and the permission mode.
- Scrolling is eased: the wheel, touchpad, Up/Down (hold to glide), PageUp/PageDown, Home and End all move smoothly.
- HDR scene with bloom, chromatic edges, vignette and film grain (`shaders/post.frag`). Dark space theme by default; Ctrl+T switches to ivory paper.
- Left/Right or the side arrows: next session in that platform. 1-4 or the tabs: spin to a platform. `+` or Ctrl+N: new session. Wheel/Up/Down/PageUp/PageDown: scroll back, End: newest. Ctrl+V: paste. Enter: send.
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
