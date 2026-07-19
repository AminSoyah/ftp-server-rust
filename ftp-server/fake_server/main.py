#!/usr/bin/env python3
"""Simple FTP test server"""

from pathlib import Path
from pyftpdlib.authorizers import DummyAuthorizer
from pyftpdlib.handlers import FTPHandler
from pyftpdlib.servers import FTPServer

HOST = "0.0.0.0"
PORT = 2121
ROOT = Path(__file__).parent / "ftp_data"

ROOT.mkdir(exist_ok=True)

authorizer = DummyAuthorizer()
authorizer.add_user("user", "password", str(ROOT), perm="elradfmw")
authorizer.add_anonymous(str(ROOT), perm="elr")

handler = FTPHandler
handler.authorizer = authorizer

server = FTPServer((HOST, PORT), handler)

print(f"""
🌲 FTP Test Server
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📍 ftp://localhost:{PORT}
📁 {ROOT. absolute()}

👤 User:      testuser / testpass
🌐 Anonymous: anonymous / (empty)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
""")

try:
    server.serve_forever()
except KeyboardInterrupt: 
    print("\n✅ Server stopped")