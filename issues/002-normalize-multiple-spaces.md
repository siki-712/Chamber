# Issue #002: Multiple spaces between notes not normalized

## Problem

When `normalize_note_spacing` is enabled, multiple consecutive spaces between notes should be collapsed to a single space.

Current behavior:
```
GAG   GAB | ABA  ABd
```

Expected behavior:
```
GAG GAB | ABA ABd
```

## Root Cause

The formatter's `emit_token` function was emitting trivia (including whitespace) verbatim without normalization.

## Solution

Added `emit_trivia_in_context()` method that normalizes whitespace when:
- Not in header section (`!self.in_header`)
- `normalize_note_spacing` is enabled

**Changes:**
- `crates/chamber_formatter/src/formatter.rs`:
  - Added `emit_trivia_in_context()` method
  - Modified `emit_token()` to use `emit_trivia_in_context()` for trivia
  - Whitespace-only trivia is collapsed to single space in body

## Status

- [x] Issue created
- [x] Fix implemented
- [x] Tests added (`test_normalize_multiple_spaces_between_notes`)
- [x] Verified (all tests pass)

## Closed

2024-12-28
