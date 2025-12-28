# Issue #004: Tuplet formatting

## Problem (Original)

Originally wanted to remove all spaces inside tuplets:
```
(3  e d d   g d B  →  (3edd gdB
```

## Issue Found

Removing ALL spaces inside tuplets breaks the triplet boundary:
```
( 3  e d d   g d B  →  (3eddg d B   ← WRONG! "g" merged into triplet
```

The triplet `(3` affects exactly 3 notes. Removing all spaces makes it unclear where the triplet ends.

## Solution (Revised)

Only normalize the tuplet marker itself, keep normal spacing between notes:
```
( 3  e d d   g d B  →  (3 e d d g d B
```

- Marker `( 3` → `(3` (spaces in marker removed)
- Notes keep single-space separation (multiple spaces → single space)
- Triplet boundary remains clear

**Changes:**
- `crates/chamber_formatter/src/formatter.rs`:
  - `format_tuplet()` only normalizes the marker token
  - Notes inside use normal `emit_token()` with single-space normalization
  - Removed `in_tuplet` flag (not needed)

## Status

- [x] Issue created
- [x] Fix implemented (marker only)
- [x] Tests added (`test_tuplet_marker_normalization`)
- [x] Verified (all tests pass)

## Closed

2024-12-28
