#!/usr/bin/env python3
from __future__ import annotations

import os
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlparse


ROOT = Path(__file__).resolve().parent
OUT = Path(os.environ.get("OUT_DIR", ROOT / "out"))
ENV_OUT = OUT / "oauth_callback.env"


class Handler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path.rstrip("/") != "/callback":
            self.send_response(404)
            self.end_headers()
            self.wfile.write(b"Not found")
            return

        query = parse_qs(parsed.query)
        code = query.get("code", [""])[0]
        state = query.get("state", [""])[0]
        error = query.get("error", [""])[0]
        error_description = query.get("error_description", [""])[0]

        OUT.mkdir(parents=True, exist_ok=True)
        lines = [
            f"TIKTOK_AUTH_CODE='{code}'",
            f"TIKTOK_CALLBACK_STATE='{state}'",
            f"TIKTOK_CALLBACK_ERROR='{error}'",
            f"TIKTOK_CALLBACK_ERROR_DESCRIPTION='{error_description}'",
        ]
        ENV_OUT.write_text("\n".join(lines) + "\n")

        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.end_headers()
        if error:
            body = f"""
            <h1>TikTok OAuth returned an error</h1>
            <p><code>{error}</code></p>
            <p>{error_description}</p>
            <p>Details written to <code>{ENV_OUT}</code>.</p>
            """
        else:
            body = f"""
            <h1>TikTok OAuth code captured</h1>
            <p>Details written to <code>{ENV_OUT}</code>.</p>
            <p>You can close this tab and run <code>bash tests/tiktok/oauth_exchange.sh</code>.</p>
            """
        self.wfile.write(body.encode("utf-8"))

    def log_message(self, format: str, *args: object) -> None:
        return


def main() -> None:
    host = os.environ.get("TIKTOK_CALLBACK_HOST", "localhost")
    port = int(os.environ.get("TIKTOK_CALLBACK_PORT", "3455"))
    server = HTTPServer((host, port), Handler)
    print(f"Listening on http://{host}:{port}/callback/")
    print("Run oauth_start.sh, open the auth URL, then complete login.")
    print("Press Ctrl+C to stop.")
    server.serve_forever()


if __name__ == "__main__":
    main()
