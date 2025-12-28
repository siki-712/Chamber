# Issue #001: `: |` not recognized as RepeatEnd

## Problem

When there is whitespace between `:` and `|` (e.g., `: |`), the lexer fails to recognize this as a RepeatEnd token. Instead, it tokenizes as:

1. `:` → `Colon` (and sets `in_header = true`!)
2. ` ` → `Whitespace`
3. `|` → `Bar`

This causes two problems:
- The formatter cannot normalize `: |` to `:|`
- The lexer incorrectly enters header mode, breaking subsequent parsing

## Expected Behavior

`: |` should be recognized as `RepeatEnd` (same as `:|`).

## Reproduction

```
AGF G3 : |   %ending
```

Current output: `AGF G3 : |% ending` (spaces preserved, wrong structure)
Expected: `AGF G3 :|` with `% ending` comment

## Root Cause

In `lexer.rs`, the `:` case does not check for `|` ahead with possible whitespace:

```rust
':' => {
    // Check for repeat markers like :|
    if self.peek() == Some('|') {
        self.advance();
        TokenKind::RepeatEnd
    } else {
        self.in_header = true;  // BUG: sets header mode!
        TokenKind::Colon
    }
}
```

## Solution

Added `has_bar_ahead()` lookahead function (similar to `has_digit_ahead()`) to detect `|` after `:` with optional whitespace.

**Changes:**
1. `crates/chamber_lexer/src/lexer.rs`:
   - Added `has_bar_ahead()` function
   - Modified `:` case to use `has_bar_ahead()` and consume whitespace before `|`

2. `crates/chamber_formatter/src/formatter.rs`:
   - Added `emit_bar_token()` to normalize bar line tokens
   - `RepeatEnd` tokens are always emitted as `:|` (removing internal whitespace)

## Status

- [x] Issue created
- [x] Fix implemented
- [x] Tests added (`test_repeat_end_with_space`, `test_repeat_end_spacing_normalization`)
- [x] Verified (all tests pass)

## Closed

2024-12-28
