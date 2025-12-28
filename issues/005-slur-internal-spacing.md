# Issue #005: Spaces inside slurs not removed

## Problem

Spaces inside slur content should be removed (slurred notes are typically written without spaces).

Current behavior:
```
(  C D E )
```

Expected behavior:
```
(CDE)
```

## Solution

Added `normalize_slurs` config option and `in_slur` flag to the formatter. When inside a slur with `normalize_slurs` enabled, whitespace trivia is skipped entirely.

**Changes:**
1. `crates/chamber_formatter/src/config.rs`:
   - Added `normalize_slurs: bool` option (default: true)

2. `crates/chamber_formatter/src/formatter.rs`:
   - Added `in_slur: bool` field to Formatter struct
   - Updated `emit_trivia_in_context()` to skip whitespace when `in_slur && config.normalize_slurs`
   - Updated `format_slur()` to set `in_slur = true` while formatting children

## Status

- [x] Issue created
- [x] Fix implemented
- [x] Tests added (`test_slur_internal_spacing`)
- [x] Verified (all tests pass)

## Closed

2024-12-28
