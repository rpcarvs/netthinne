#!/usr/bin/env python3
"""Strips content hashes from dx build asset filenames and patches all references.

dx build generates names like:  netthinne_bg-dxhABCDEF.wasm
This script renames them to:    netthinne_bg.wasm

Stable filenames let the service worker overwrite cached assets on each deploy
instead of accumulating stale copies in the cache.
"""
import re
import sys
from pathlib import Path

docs = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("docs")
assets = docs / "assets"

# Only rename .wasm and .js — CSS filenames are baked into the WASM binary at
# compile time via asset!() macros and cannot be patched after compilation.
HASH_PATTERN = re.compile(r"^(.+?)-dxh[0-9a-f]+(\.(wasm|js))$")

renames = {}
for path in assets.iterdir():
    m = HASH_PATTERN.match(path.name)
    if m:
        renames[path.name] = m.group(1) + m.group(2)

if not renames:
    print("No hashed assets found.")
    sys.exit(0)


def apply_renames(text):
    for old, new in renames.items():
        text = text.replace(old, new)
    return text


# 1. Patch index.html references
index_path = docs / "index.html"
index_path.write_text(apply_renames(index_path.read_text()))
print(f"Patched: {index_path}")

# 2. Patch JS loader contents — it contains a fetch() call with the WASM filename
for old_name in renames:
    if old_name.endswith(".js"):
        js_path = assets / old_name
        js_path.write_text(apply_renames(js_path.read_text()))
        print(f"Patched: {js_path}")

# 3. Rename all hashed files to stable names
for old_name, new_name in renames.items():
    (assets / old_name).rename(assets / new_name)
    print(f"Renamed: {old_name} -> {new_name}")
