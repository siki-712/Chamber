# Issue #003: `| :` not recognized as RepeatStart

## Problem

When there is whitespace between `|` and `:` (e.g., `| :`), the lexer fails to recognize this as a RepeatStart token.

Current tokenization:
1. `|` → `Bar`
2. ` ` → `Whitespace`
3. `:` → `Colon` (sets `in_header = true`!)

## Expected Behavior

`| :` should be recognized as `RepeatStart` (same as `|:`).

## Reproduction

```
| : GAG GAB |
```

Current: Parsed as Bar + Whitespace + Colon (header mode!)
Expected: Parsed as RepeatStart

## Solution

Added `has_colon_ahead_for_repeat()` lookahead function to detect `:` after `|` with optional whitespace.

**Changes:**
1. `crates/chamber_lexer/src/lexer.rs`:
   - Added `has_colon_ahead_for_repeat()` function
   - Modified `bar_line()` to check for `:` ahead and consume whitespace

2. `crates/chamber_formatter/src/formatter.rs`:
   - Updated `emit_bar_token()` to normalize `RepeatStart` to `|:`

## Status

- [x] Issue created
- [x] Fix implemented
- [x] Tests added (`test_repeat_start_with_space`, `test_repeat_start_spacing_normalization`)
- [x] Verified (all tests pass)

## Closed

2024-12-28
