# CSQLite

SQLite **3.54.1** (2026-10-16), the amalgamation published on
[sqlite.org/download.html](https://www.sqlite.org/download.html):

| File | From |
|---|---|
| `sqlite3.c` | `sqlite-amalgamation-3540100.zip` |
| `include/sqlite3.h` | the same archive |

SHA3-256 of the archive, checked against the download page:
`92281c86e2957f9b9305890f9ac04204caa7b677cc868dc326d942689df28e30`.

One comment of `sqlite3.c` is reworded (it cited where an explanation came from); no line
of code differs from the archive. `update.sh` does the same.

The compile options are in `Package.swift` (foreign keys on by default, no extension
loading, no deprecated API, strict double-quoted strings). SQLite is in the public domain.

To move to a newer release: `./update.sh` downloads the latest amalgamation, checks its
SHA3-256 against sqlite.org, and replaces the two files. Then update the version here and
in `Package.swift`, and run `swift test --package-path ios/DuovaKit`.
