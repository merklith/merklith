#!/usr/bin/env python3
"""
MERKLITH CORS Proxy - Basit ve Hızlı
Tüm MERKLITH node'larına CORS desteği ekler
"""

import http.server
import socketserver
import urllib.request
import json
import sys

PORT = 9999

# Node haritalaması
NODES = {
    "node1": "http://localhost:8545",
    "node2": "http://localhost:8547",
    "node3": "http://localhost:8549",
    "default": "http://localhost:8545",
}


class CORSHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        # Sessiz mod
        pass

    def do_OPTIONS(self):
        # CORS preflight
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def do_GET(self):
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()

        # Health check
        response = {
            "status": "ok",
            "service": "MERKLITH CORS Proxy",
            "port": PORT,
            "nodes": list(NODES.keys()),
        }
        self.wfile.write(json.dumps(response).encode())

    def do_POST(self):
        try:
            # URL path'e göre hedef node belirle
            path = self.path.strip("/")
            target_url = NODES.get(path, NODES["default"])

            # Request body'yi oku
            content_length = int(self.headers.get("Content-Length", 0))
            post_data = self.rfile.read(content_length)

            # MERKLITH node'a istek gönder
            req = urllib.request.Request(
                target_url,
                data=post_data,
                headers={
                    "Content-Type": "application/json",
                    "Accept": "application/json",
                },
                method="POST",
            )

            with urllib.request.urlopen(req, timeout=10) as response:
                response_data = response.read()

                # CORS header'ları ekle ve yanıtı gönder
                self.send_response(200)
                self.send_header("Content-Type", "application/json")
                self.send_header("Access-Control-Allow-Origin", "*")
                self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
                self.send_header("Access-Control-Allow-Headers", "Content-Type")
                self.end_headers()
                self.wfile.write(response_data)

        except urllib.error.URLError as e:
            self.send_response(503)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            error = {
                "jsonrpc": "2.0",
                "error": {"code": -32000, "message": f"Node unreachable: {str(e)}"},
                "id": 1,
            }
            self.wfile.write(json.dumps(error).encode())

        except Exception as e:
            self.send_response(500)
            self.send_header("Content-Type", "application/json")
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            error = {
                "jsonrpc": "2.0",
                "error": {"code": -32603, "message": f"Proxy error: {str(e)}"},
                "id": 1,
            }
            self.wfile.write(json.dumps(error).encode())


def main():
    print(f">> MERKLITH CORS Proxy starting...")
    print(f">> Port: {PORT}")
    print(f">> Nodes:")
    for name, url in NODES.items():
        print(f"   {name}: {url}")
    print(f"\n>> Web UI configuration:")
    print(f"   RPC URL: http://localhost:{PORT}")
    print(f"\n>> Press Ctrl+C to stop")
    print("-" * 50)

    with socketserver.ThreadingTCPServer(("", PORT), CORSHandler) as httpd:
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n\n>> Proxy stopped.")
            sys.exit(0)


if __name__ == "__main__":
    main()
