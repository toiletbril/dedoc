#!/bin/env python3

import ssl
import sys

from http.server import HTTPServer, SimpleHTTPRequestHandler

if len(sys.argv) != 5:
  print(f"USAGE: {sys.argv[0]} <host> <port> <key.pem> <cert.pem>")
  sys.exit(1)

host, port, keyfile, certfile = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]

httpd = HTTPServer((host, port), SimpleHTTPRequestHandler)
ssl_context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ssl_context.load_cert_chain(certfile=certfile, keyfile=keyfile)
httpd.socket = ssl_context.wrap_socket(httpd.socket, server_side=True)

httpd.serve_forever()
