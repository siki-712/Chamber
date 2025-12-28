# Issue #004: Spaces inside tuplets not removed

## Problem

When `normalize_tuplets` is enabled, spaces inside tuplet content should be removed.

Current behavior:
```
(3  e d d   g d B
```

Expected behavior:
```
(3edd gdB
```

## Solution

Added `in_tuplet` flag to the formatter. When inside a tuplet with `normalize_tuplets` enabled, whitespace trivia is skipped entirely.

**Changes:**
- `crates/chamber_formatter/src/formatter.rs`:
  - Added `in_tuplet: bool` field to Formatter struct
  - Updated `emit_trivia_in_context()` to skip whitespace when `in_tuplet && config.normalize_tuplets`
  - Updated `format_tuplet()` to set `in_tuplet = true` while formatting children

## Status

- [x] Issue created
- [x] Fix implemented
- [x] Tests added (`test_tuplet_internal_spacing`)
- [x] Verified (all tests pass)

## Closed

2024-12-28
