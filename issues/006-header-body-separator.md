# Issue #006: Remove blank line between header and body

## Problem

ABC 2.1 spec says the tune body "follows immediately after" the K: field. Empty lines between header and body should be removed to conform to the spec.

Before:
```
X:1
T:Test
K:G

|: GAG GAB |
```

After:
```
X:1
T:Test
K:G
|: GAG GAB |
```

## Solution

Added `remove_header_body_separator` config option (default: true).

**Changes:**

1. `crates/chamber_formatter/src/config.rs`:
   - Added `remove_header_body_separator: bool` option

2. `crates/chamber_formatter/src/formatter.rs`:
   - Added `at_body_start: bool` flag to track start of body
   - `format_header()` calls `trim_trailing_blank_lines()` after formatting
   - `emit_text()` and `emit()` reset `at_body_start` when emitting content
   - `emit_trivia_in_context()` skips whitespace/newlines at body start
   - `emit_trivia_in_bar_context()` skips whitespace/newlines at body start

## Tests

- `test_remove_header_body_separator` - Empty line removed with default config
- `test_preserve_header_body_separator` - Empty line preserved with passthrough config
- `test_remove_header_body_separator_with_note_start` - Works when body starts with note

## Status

- [x] Issue created
- [x] Fix implemented
- [x] Tests added
- [x] Verified (all 37 tests pass)

## Closed

2024-12-28
