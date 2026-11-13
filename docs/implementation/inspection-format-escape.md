# Format-escape iteration: Unicode format characters in inspection stderr (backlog B-9, audit F15)

Status: **delivered on merge of its PR**, branch `impl/b9-format-escape` from `3c8c95c` (Schedule A).
Implemented by Claude (claude-opus-5-5, orchestrator, burn mode). Evidence lives in the
[checkpoint](inspection-progress.md).

## Scope

Host stderr escaping in `crates/wsr/src/commands/inspect.rs` escaped only `char::is_control`
characters, so Unicode format characters (category `Cf`: U+202E bidi override, U+2066–U+2069
isolates, U+200B zero-width space) from untrusted workflow keys and job identifiers reached the
terminal raw and could make a diagnostic read differently from its bytes. The escaper now escapes
every character `char::escape_debug` escapes, matching stdout's rendering, except quotes and
backslashes, which stay as written (identifiers already escaped by `wsr-gha` are not escaped twice).
Per-character escaping also escapes combining marks, which stdout's `str::escape_debug` leaves raw
except at the start of a string; this is a deliberate conservative difference for diagnostics.
No change to stdout, exit codes, categories or routes.

## Acceptance and expected red

CLI test with U+202E/U+200B in an unknown step key and U+202E/U+200B/U+2066/U+2069 in a job ID:
exact escaped diagnostic, no raw format character on stderr. Expected red: raw bytes on stderr.
Existing control-byte escaping tests unchanged.
