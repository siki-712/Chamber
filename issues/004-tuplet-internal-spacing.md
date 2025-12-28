# Issue #004: Tuplet formatting

## Problem

Tuplet `( 3  e d d   g d B` should become `(3edd g d B`:
- Inside tuplet: no spaces between notes
- Outside tuplet: normal spacing
- Support `(3`, `(4`, `(5`, etc.

## Bugs Found & Fixed

### Bug 1: Hardcoded tuplet count
Parser had `count = 3` hardcoded. Fixed to extract number from marker.

### Bug 2: Trailing whitespace lost
Last note's trailing whitespace was skipped, merging following notes.
Fixed by emitting space after tuplet if trailing whitespace existed.

## Solution

**Parser changes** (`crates/chamber_parser/src/cst_parser.rs`):
- Added `source: &str` field to parser
- Extract digit from marker text: `(4` → count=4

**Formatter changes** (`crates/chamber_formatter/src/formatter.rs`):
- Track `had_trailing_whitespace` in `format_tuplet()`
- Emit space after tuplet if needed

## Result

```
( 3  e d d   g d B  →  (3edd g d B
(4 a b c d e f      →  (4abcd e f
```

## Status

- [x] Issue created
- [x] Fix implemented
- [x] Tests added (`test_tuplet_normalization_3`, `test_tuplet_normalization_4`)
- [x] Verified (all tests pass)

## Closed

2024-12-28
