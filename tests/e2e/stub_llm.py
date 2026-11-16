"""Tiny offline OpenAI/Ollama-compatible model server for end-to-end tests.

Turn 1 (last message from the user): asks for read_file(path=<file named in the prompt>).
Turn 2 (last message is the tool result): answers with the first line of that result.
If the request carries no `tools` (text tool protocol), the call is written as <tool_call> text.
Every request is appended to the log file given by STUB_LOG so tests can inspect them.
"""
import json, os, re, sys, time
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

MODEL = os.environ.get("STUB_MODEL", "stub-coder:1b")
LOG = os.environ.get("STUB_LOG")


class H(BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def _send(self, obj, code=200):
        b = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(b)))
        self.end_headers()
        self.wfile.write(b)

    def do_GET(self):
        if self.path.startswith("/api/tags"):
            return self._send({"models": [{"name": MODEL, "model": MODEL}]})
        if self.path.startswith("/v1/models"):
            return self._send({"object": "list", "data": [{"id": MODEL, "object": "model"}]})
        if self.path in ("/", "/health", "/api/version"):
            return self._send({"status": "ok", "version": "stub"})
        self._send({"error": "not found"}, 404)

    def do_POST(self):
        n = int(self.headers.get("Content-Length", 0))
        body = json.loads(self.rfile.read(n) or b"{}")
        if LOG:
            with open(LOG, "a") as f:
                f.write(json.dumps({"path": self.path, "body": body}) + "\n")
        if not self.path.startswith("/v1/chat/completions"):
            return self._send({"error": "not found"}, 404)
        msgs = body.get("messages", [])
        last = msgs[-1] if msgs else {}
        native = bool(body.get("tools"))
        tool_result = None
        if last.get("role") == "tool":
            tool_result = last.get("content", "")
        elif last.get("role") == "user" and "<tool_response>" in str(last.get("content", "")):
            tool_result = re.sub(r"</?tool_response>", "", last["content"]).strip()
        if tool_result is not None:
            time.sleep(float(os.environ.get("STUB_DELAY", "0")))
            first = tool_result.strip().splitlines()[0] if tool_result.strip() else "(empty)"
            msg = {"role": "assistant", "content": f"The file starts with: {first}"}
        else:
            text = str(last.get("content", ""))
            m = re.search(r"([\w./-]+\.(?:txt|md|rs|py|toml))", text)
            path = m.group(1) if m else "README.md"
            args = {"path": path}
            if native:
                msg = {"role": "assistant", "content": "", "tool_calls": [{"id": "call_1", "type": "function", "function": {"name": "read_file", "arguments": json.dumps(args)}}]}
            else:
                msg = {"role": "assistant", "content": "<tool_call>\n" + json.dumps({"name": "read_file", "arguments": args}) + "\n</tool_call>"}
        self._send({"id": "stub", "object": "chat.completion", "model": body.get("model"), "choices": [{"index": 0, "message": msg, "finish_reason": "stop"}]})


if __name__ == "__main__":
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 11555
    ThreadingHTTPServer(("127.0.0.1", port), H).serve_forever()
