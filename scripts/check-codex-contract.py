#!/usr/bin/env python3
"""Inspect actual Codex request tools against loopback, with no auth or inference."""
import http.server
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading

root = Path(__file__).resolve().parent.parent
captures = []


class Capture(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        data = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
        captures.append({"path": self.path, "tools": data.get("tools", [])})
        # Always reject: this server cannot infer or call tools.
        body = b'{"error":{"message":"Synthetic contract capture complete","type":"invalid_request_error"}}'
        self.send_response(400)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *args):
        pass


with tempfile.TemporaryDirectory(prefix="ramo-codex-contract-") as folder:
    config = json.loads(subprocess.check_output([
        "cargo", "run", "--quiet", "--manifest-path", "src-tauri/Cargo.toml",
        "--no-default-features", "--locked", "--offline", "--example", "codex_contract",
        "--", str(Path(folder) / "message.txt")
    ], cwd=root))
    version = subprocess.check_output([config["binary"], "--version"], text=True).strip()
    if version != "codex-cli " + config["version"]:
        raise SystemExit("Requires the exact adapter-tested Codex CLI version")
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Capture)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    env = dict(os.environ)
    env["CODEX_HOME"] = folder
    for key in ("OPENAI_API_KEY", "CODEX_API_KEY", "ANTHROPIC_API_KEY"):
        env.pop(key, None)
    provider = {"name": "Ramo synthetic capture", "base_url": f"http://127.0.0.1:{server.server_port}",
                "wire_api": "responses", "requires_openai_auth": False, "supports_websockets": False,
                "request_max_retries": 0, "stream_max_retries": 0}
    table = "{" + ", ".join(k + " = " + json.dumps(v) for k, v in provider.items()) + "}"
    args = config["args"] + ["-c", 'model_provider="ramo_mock"', "-c", "model_providers.ramo_mock=" + table]
    try:
        result = subprocess.run([config["binary"], *args], input=b"Synthetic offline request only.",
                                cwd=folder, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=30)
    finally:
        server.shutdown()
        server.server_close()
    if not captures or any(item["tools"] for item in captures):
        raise SystemExit("FAILED: no captured request or model tools remained enabled")
    print(json.dumps({"cliVersion": config["version"], "requestsCaptured": len(captures),
                      "modelTools": [], "home": "isolated empty temporary directory",
                      "credentials": "none", "inference": "none; local server rejects every request",
                      "result": "passed; actual adapter arguments exposed zero model tools"}, indent=2))
