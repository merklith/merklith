#!/usr/bin/env python3
"""
MERKLITH CORS Proxy Server
Proxies requests to MERKLITH nodes with CORS headers
"""

import http.server
import socketserver
import urllib.request
import urllib.error
import json

PORT = 8541
MERKLITH_NODES = {
    "node1": "http://localhost:8545",
    "node2": "http://localhost:8547",
    "node3": "http://localhost:8549",
}


class CORSProxyHandler(http.server.BaseHTTPRequestHandler):
    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def do_POST(self):
        # Get target node from path or default to node1
        path = self.path.strip("/")
        target_url = MERKLITH_NODES.get(path, MERKLITH_NODES["node1"])

        content_length = int(self.headers["Content-Length"])
        post_data = self.rfile.read(content_length)

        try:
            req = urllib.request.Request(
                target_url,
                data=post_data,
                headers={"Content-Type": "application/json"},
                method="POST",
            )

            with urllib.request.urlopen(req, timeout=10) as response:
                response_data = response.read()

                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                self.send_header("Access-Control-Allow-Headers", "Content-Type")
                self.end_headers()
                self.wfile.write(response_data)

        except urllib.error.URLError as e:
            self.send_response(502)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            error_response = json.dumps(
                {
                    "jsonrpc": "2.0",
                    "error": {"code": -32000, "message": f"Node unreachable: {str(e)}"},
                    "id": 1,
                }
            ).encode()
            self.wfile.write(error_response)

    def do_GET(self):
        # Health check endpoint
        if self.path == "/health":
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            health = {
                "status": "ok",
                "proxy": "MERKLITH CORS Proxy",
                "nodes": list(MERKLITH_NODES.keys()),
            }
            self.wfile.write(json.dumps(health).encode())
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, format, *args):
        # Suppress default logging
        pass


if __name__ == "__main__":
    with socketserver.TCPServer(("", PORT), CORSProxyHandler) as httpd:
        print(f"MERKLITH CORS Proxy Server running on http://localhost:{PORT}")
        print(f"Proxying to nodes:")
        for name, url in MERKLITH_NODES.items():
            print(f"  {name}: {url}")
        print("\\nUse this proxy in your Web UI instead of direct node URLs")
        print("Press Ctrl+C to stop")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\\nShutting down...")
