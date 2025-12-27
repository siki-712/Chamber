# Chamber Diagnostic Codes

This document lists all diagnostic codes used by Chamber's ABC notation parser.

## Code Format

Codes follow the pattern `XNNN` where:
- `X` = Category letter
- `NNN` = Three-digit number

### Categories

| Prefix | Category | Description |
|--------|----------|-------------|
| L | Lexer | Tokenization errors |
| H | Header | Header field validation |
| M | Music | Music body parsing |
| S | Structural | Overall structure issues |
| W | Warnings | General warnings |

### Severity Levels

| Level | Description |
|-------|-------------|
| Error | Must be fixed; parsing may produce incorrect results |
| Warning | Should be reviewed; parsing continues normally |

---

## Lexer Errors (L001-L099)

| Code | Name | Severity | Description |
|------|------|----------|-------------|
| L001 | UnexpectedCharacter | Error | Character not recognized in ABC notation |

**Example:**
```abc
X:1
K:C
C@D#E
  ^ L001: unexpected character '@'
```

---

## Header Errors (H001-H099)

| Code | Name | Severity | Description |
|------|------|----------|-------------|
| H001 | MissingReferenceNumber | Error | No `X:` field found |
| H002 | MissingKeyField | Error | No `K:` field found |
| H003 | DuplicateReferenceNumber | Error | Multiple `X:` fields |
| H004 | InvalidFieldOrder | Warning | `X:` must be first header field |
| H005 | InvalidMeterValue | Error | Invalid `M:` value (e.g., `M:abc`) |
| H006 | InvalidTempo | Error | Invalid `Q:` value |
| H007 | InvalidUnitNoteLength | Error | Invalid `L:` value |
| H008 | InvalidKeySignature | Error | Invalid `K:` value |
| H009 | MissingTitle | Warning | No `T:` field found |
| H010 | EmptyTitle | Warning | `T:` field has no value |
| H011 | EmptyReferenceNumber | Error | `X:` field has no value |
| H012 | InvalidReferenceNumber | Error | `X:` value is not a positive integer |

**Examples:**
```abc
T:My Tune
K:C
CDEF
^ H001: missing reference number field (X:)

X:1
K:C
CDEF
^ H009: missing title field (T:)

X:
T:Test
K:C
^ H011: empty reference number field

X:abc
T:Test
K:C
^ H012: invalid reference number (must be a positive integer)

T:Test
X:1
K:C
^ H004: X: should be the first field in header
```

---

## Music Body Errors (M001-M099)

| Code | Name | Severity | Description |
|------|------|----------|-------------|
| M001 | UnclosedChord | Error | `[` without matching `]` |
| M002 | UnclosedSlur | Error | `(` without matching `)` |
| M003 | UnclosedGraceNotes | Error | `{` without matching `}` |
| M004 | UnexpectedClosingBracket | Error | `]` without matching `[` |
| M005 | UnexpectedClosingParen | Error | `)` without matching `(` |
| M006 | UnexpectedClosingBrace | Error | `}` without matching `{` |
| M007 | InvalidNoteName | Error | Invalid note character |
| M008 | InvalidAccidental | Error | Invalid accidental syntax |
| M009 | InvalidDuration | Error | Invalid duration syntax |
| M010 | EmptyChord | Warning | `[]` with no notes |
| M011 | EmptyTuplet | Warning | Tuplet with no notes |
| M012 | TupletNoteMismatch | Warning | Tuplet note count differs from ratio |

**Examples:**
```abc
X:1
K:C
[CEG
    ^ M001: unclosed chord, missing ']'

X:1
K:C
(CDE
    ^ M002: unclosed slur, missing ')'

X:1
K:C
{gab
    ^ M003: unclosed grace notes, missing '}'

X:1
K:C
CDE]F
   ^ M004: unexpected ']' without matching '['

X:1
K:C
[]
^ M010: empty chord

X:1
K:C
(3CD|
   ^ M012: tuplet note count does not match ratio (expected 3, got 2)
```

---

## Structural Errors (S001-S099)

| Code | Name | Severity | Description |
|------|------|----------|-------------|
| S001 | EmptyTune | Error | No content in tune |
| S002 | UnexpectedToken | Error | Token not expected at this position |

---

## Warnings (W001-W099)

| Code | Name | Severity | Description |
|------|------|----------|-------------|
| W001 | UnusualOctave | Warning | Note in extremely high or low octave |
| W002 | SuspiciousDuration | Warning | Very large duration value |

**Examples:**
```abc
X:1
K:C
C''''
    ^ W001: unusual octave (very high or very low)

X:1
K:C
C128
   ^ W002: suspicious duration (very large)
```

---

## Implementation Status

| Code | Implemented | Tested |
|------|-------------|--------|
| L001 | Yes | Yes |
| H001 | Yes | Yes |
| H002 | Yes | Yes |
| H003 | No | - |
| H004 | Yes | Yes |
| H005 | No | - |
| H006 | No | - |
| H007 | No | - |
| H008 | No | - |
| H009 | Yes | Yes |
| H010 | Yes | Yes |
| H011 | Yes | Yes |
| H012 | Yes | Yes |
| M001 | Yes | Yes |
| M002 | Yes | Yes |
| M003 | Yes | Yes |
| M004 | Yes | Yes |
| M005 | Yes | Yes |
| M006 | Yes | Yes |
| M007 | No | - |
| M008 | No | - |
| M009 | No | - |
| M010 | Yes | Yes |
| M011 | Yes | Yes |
| M012 | Yes | Yes |
| S001 | No | - |
| S002 | No | - |
| W001 | No | - |
| W002 | No | - |
