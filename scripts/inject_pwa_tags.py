#!/usr/bin/env python3
"""Injects PWA meta tags into the dx-generated index.html after build."""
import sys

path = sys.argv[1] if len(sys.argv) > 1 else "docs/index.html"

with open(path) as f:
    html = f.read()

pwa_tags = (
    '    <meta name="theme-color" content="#1a4a58">\n'
    '    <meta name="apple-mobile-web-app-capable" content="yes">\n'
    '    <meta name="apple-mobile-web-app-status-bar-style" content="black-translucent">\n'
    '    <meta name="apple-mobile-web-app-title" content="Netthinne">\n'
    '    <link rel="manifest" href="manifest.json">\n'
    '    <link rel="apple-touch-icon" href="icons/icon-192.png">\n'
    '    <script>if("serviceWorker"in navigator)'
    'navigator.serviceWorker.register("./service-worker.js");</script>\n'
)

marker = '<meta charset="UTF-8">'
if marker not in html:
    print(f"ERROR: marker not found in {path}", file=sys.stderr)
    sys.exit(1)

html = html.replace(marker, marker + "\n" + pwa_tags, 1)

with open(path, "w") as f:
    f.write(html)

print(f"PWA tags injected into {path}")
