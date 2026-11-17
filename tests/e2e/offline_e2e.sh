#!/usr/bin/env bash
# Offline end-to-end test. Runs Amni-Code, two stub model servers and headless Chromium inside a
# network namespace that has ONLY loopback (no route out), plus a dead HTTPS proxy for good measure.
# usage: sudo tests/e2e/offline_e2e.sh [path/to/amni] [out-dir]     (needs root for unshare -n)
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
BIN="${1:-$HERE/../../target/debug/amni}"
OUT="${2:-$(mktemp -d)}"
if [ "${AMNI_E2E_IN_NS:-}" != "1" ]; then
  exec env AMNI_E2E_IN_NS=1 unshare -n "$0" "$(readlink -f "$BIN")" "$OUT"
fi
python3 "$HERE/lo_up.py" >/dev/null
WORK="$(mktemp -d)"; mkdir -p "$WORK/home" "$WORK/ws"
printf '# Demo workspace\nhello from the offline workspace\n' > "$WORK/ws/notes.md"
cat > "$WORK/home/config.seed" <<JSON
{"provider":"ollama","model":"stub-coder:1b","api_key":"","base_url":"http://127.0.0.1:11434","auto_approve":false,"mode":"edit","working_dir":"$WORK/ws","model_dir":"","offline":true}
JSON
mkdir -p "$WORK/home/.amni" && mv "$WORK/home/config.seed" "$WORK/home/.amni/config.json"
STUB_LOG="$WORK/ollama.log" STUB_MODEL="stub-coder:1b" python3 "$HERE/stub_llm.py" 11434 & P1=$!
STUB_DELAY=1.5 STUB_LOG="$WORK/adam.log" STUB_MODEL="adam:8b" python3 "$HERE/stub_llm.py" 7700 & P2=$!
trap 'kill $P1 $P2 ${P3:-} 2>/dev/null || true' EXIT
for i in $(seq 1 50); do curl -s -o /dev/null http://127.0.0.1:11434/api/tags && curl -s -o /dev/null http://127.0.0.1:7700/v1/models && break; sleep 0.1; done
START=$(date +%s%N)
( cd "$WORK/ws" && HOME="$WORK/home" HTTPS_PROXY=http://127.0.0.1:9 HTTP_PROXY=http://127.0.0.1:9 NO_PROXY=127.0.0.1,localhost \
  AMNI_CODE_PORT=3199 AMNI_CODE_HOST=127.0.0.1 "$BIN" --serve --offline "$WORK/ws" > "$WORK/amni.log" 2>&1 ) & P3=$!
for i in $(seq 1 50); do curl -s -o /dev/null http://127.0.0.1:3199/health && break; sleep 0.2; done
echo "startup: $(( ($(date +%s%N) - START) / 1000000 )) ms"
grep -E "Offline mode|Local model server" "$WORK/amni.log" || true
# Cloud providers must fail fast and clearly while offline (no timeout).
curl -s -X POST -H 'Content-Type: application/json' -d '{"provider":"anthropic","model":"claude-opus-5-5","api_key":"dummy"}' http://127.0.0.1:3199/api/config >/dev/null
T0=$(date +%s%N)
MSG=$(curl -s -N -X POST -H 'Content-Type: application/json' -d '{"message":"hi"}' http://127.0.0.1:3199/api/chat | grep -o 'Offline mode is on[^"]*' | head -1)
echo "cloud while offline: $(( ($(date +%s%N) - T0) / 1000000 )) ms -> ${MSG:0:60}"
[ -n "$MSG" ] || { echo "FAIL: no offline message"; exit 1; }
curl -s -X POST -H 'Content-Type: application/json' -d '{"provider":"ollama","model":"stub-coder:1b","base_url":"http://127.0.0.1:11434"}' http://127.0.0.1:3199/api/config >/dev/null
node "$HERE/offline_ui.js" http://127.0.0.1:3199 "$OUT"
python3 - "$WORK" <<'PY'
import json, sys
w = sys.argv[1]
ol = [json.loads(l) for l in open(w + "/ollama.log")]
ad = [json.loads(l) for l in open(w + "/adam.log")]
assert len(ol) == 2 and len(ad) == 2, (len(ol), len(ad))
assert "tools" not in ol[0]["body"], "small local model should use the text tool protocol"
assert "<tool_call>" in ol[0]["body"]["messages"][0]["content"]
names = [t["function"]["name"] for t in ad[0]["body"]["tools"]]
assert "read_file" in names and "web_search" not in names and "web_fetch" not in names, names
assert ad[1]["body"]["messages"][-1]["role"] == "tool"
print("stub checks ok: ollama text-protocol x2, adam native tools x2 (web tools filtered offline)")
PY
node "$HERE/ui_click.js" http://127.0.0.1:3199 "$OUT"
echo "PASS (screenshots in $OUT)"
