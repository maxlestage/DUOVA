#!/usr/bin/env bash
# Replaces sqlite3.c and include/sqlite3.h with the latest amalgamation of sqlite.org,
# after checking its SHA3-256 against the one the download page publishes.
set -euo pipefail
cd "$(dirname "$0")"

page=$(curl -fsSL https://www.sqlite.org/download.html)
line=$(grep -m1 -oE 'PRODUCT,[0-9.]+,[0-9]+/sqlite-amalgamation-[0-9]+\.zip,[0-9]+,[0-9a-f]+' <<<"$page")
IFS=, read -r _ version path _ sha3 <<<"$line"
echo "SQLite $version ($path)"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
curl -fsSL -o "$work/amalgamation.zip" "https://www.sqlite.org/$path"
actual=$(python3 -I -c 'import hashlib, sys; print(hashlib.sha3_256(open(sys.argv[1], "rb").read()).hexdigest())' "$work/amalgamation.zip")
if [ "$actual" != "$sha3" ]; then
  echo "SHA3-256 mismatch: expected $sha3, got $actual" >&2
  exit 1
fi
unzip -q "$work/amalgamation.zip" -d "$work"
cp "$work"/sqlite-amalgamation-*/sqlite3.c sqlite3.c
# One comment of the amalgamation cites where an explanation came from: reworded, no
# code changes (the same as in the copy of this folder).
perl -0pi -e 's/, or so \w+\n(\s*\*\*) tells me\./.\n$1/g' sqlite3.c
cp "$work"/sqlite-amalgamation-*/sqlite3.h include/sqlite3.h
echo "Updated to SQLite $version, SHA3-256 $sha3."
