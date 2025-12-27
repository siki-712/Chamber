# Chamber Examples

Example ABC notation files for testing and demonstration.

## Directory Structure

```
examples/
├── valid/           # Valid ABC files (no errors)
│   ├── simple.abc
│   ├── twinkle.abc
│   ├── modes.abc
│   ├── inline_fields.abc
│   └── decorations.abc
├── errors/          # Files with errors
│   ├── header/      # Header validation errors (H001-H012)
│   ├── music/       # Music body errors (M001-M013, L001)
│   └── structure/   # Structural errors (S001-S002)
└── warnings/        # Files with warnings only
    ├── missing_title.abc
    ├── empty_title.abc
    ├── unusual_octave.abc
    └── ...
```

## Usage

Check a single file:
```bash
chamber check examples/valid/simple.abc
```

Check all error examples:
```bash
for f in examples/errors/**/*.abc; do
  echo "=== $f ==="
  chamber check "$f"
done
```

## Error Categories

### Header Errors (H001-H012)
- `missing_x.abc` - H001: Missing X: field
- `missing_k.abc` - H002: Missing K: field
- `duplicate_x.abc` - H003: Duplicate X: field
- `wrong_order.abc` - H004: X: not first
- `invalid_meter.abc` - H005: Invalid M: value
- `invalid_tempo.abc` - H006: Invalid Q: value
- `invalid_length.abc` - H007: Invalid L: value
- `invalid_key.abc` - H008: Invalid K: value
- `empty_x.abc` - H011: Empty X: field
- `invalid_x_value.abc` - H012: Non-numeric X: value

### Music Errors (M001-M013)
- `unclosed_chord.abc` - M001: Missing ]
- `unclosed_slur.abc` - M002: Missing )
- `unclosed_grace.abc` - M003: Missing }
- `unexpected_brackets.abc` - M004/M005/M006: Unexpected closers
- `invalid_duration.abc` - M009: Zero denominator
- `unclosed_inline_field.abc` - M013: Unclosed inline field
- `unexpected_char.abc` - L001: Invalid character

### Structure Errors (S001-S002)
- `empty.abc` - S001: Empty file
- `field_in_body.abc` - S002: Header field in music body

### Warnings
- `missing_title.abc` - H009: No T: field
- `empty_title.abc` - H010: Empty T: field
- `unusual_octave.abc` - W001: Extreme octaves
- `suspicious_duration.abc` - W002: Very long notes
- `empty_chord.abc` - M010: Empty []
- `tuplet_mismatch.abc` - M012: Wrong note count
