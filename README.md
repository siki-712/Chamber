# Chamber

> **An opinionated language environment for ABC notation.**

**chamber** is a fast, partial-safe language toolkit for **writing, checking, and refining ABC notation**.
Inspired by tools like **Biome**, chamber treats ABC not as a file format, but as a **language** — one that deserves modern tooling.

Just like chamber music, it focuses on **structure, clarity, and interaction**, rather than scale or spectacle.

---

## Installation

### npm (Browser/WASM)

```bash
npm install chamber-abc
```

### CLI

```bash
cargo install --path crates/chamber_cli
```

---

## Quick Start

### Browser

```javascript
import init, { parse, analyze, tokenize, format_default } from 'chamber-abc';

await init();

const source = `X:1
T:My Tune
M:4/4
L:1/8
K:C
CDEF GABc|`;

// Parse
const result = parse(source);
console.log(result.tune);        // AST
console.log(result.diagnostics); // Parse errors

// Analyze (lint)
const analysis = analyze(result.tune);
console.log(analysis.diagnostics); // Warnings

// Tokenize (for syntax highlighting)
const tokens = tokenize(source);
// [{ kind: "FieldLabel", range: { start: 0, end: 1 } }, ...]

// Format
const formatted = format_default(source);
```

### CLI

```bash
chamber check tune.abc
```

---

## Features

### Partial-safe parsing

chamber never crashes on unfinished input.

```abc
K:
C D E F | G
```

Still produces:
- an AST
- diagnostics
- tokens for highlighting

Editing always comes first.

---

### Structured diagnostics

30+ diagnostic codes with rich context:

```
error[H002]: missing key field (K:)
 --> tune.abc:1:1
  |
1 | X:1
  | ^^^ tune must have a K: field

warning[W003]: bar length mismatch
 --> tune.abc:5:1
  |
5 | CDEFG|
  | ^^^^^ bar has 5/8, expected 4/4
```

---

### Lint rules

| Rule | Code | Description |
|------|------|-------------|
| UnknownDecoration | M014 | Unknown decoration name (with suggestions) |
| UnusualOctave | W001 | Notes in extreme octaves |
| SuspiciousDuration | W002 | Very long note durations |
| BarLengthMismatch | W003 | Bar length doesn't match time signature |

---

### Formatter

```abc
C  D   E|F G
```

↓

```abc
C D E | F G
```

13 configurable options:
- `normalize_note_spacing`
- `space_around_bars`
- `trim_trailing_whitespace`
- `normalize_header_order`
- and more...

---

### Syntax highlighting (tokenizer)

```javascript
const tokens = tokenize(source);
// [
//   { kind: "FieldLabel", range: { start: 0, end: 1 } },
//   { kind: "Note", range: { start: 10, end: 11 } },
//   { kind: "Bar", range: { start: 20, end: 21 } },
//   ...
// ]
```

Token kinds: `Note`, `Rest`, `FieldLabel`, `Comment`, `Decoration`, `Bar`, `Sharp`, `Flat`, `Tuplet`, etc.

---

## Architecture

```
source text
    ↓
  lexer → tokens (for highlighting)
    ↓
  parser (partial-safe)
    ↓
   AST
    ↓
 ┌──┴──┐
 │     │
analyzer  formatter
 │
diagnostics
```

### Crates

| Crate | Description |
|-------|-------------|
| `chamber_lexer` | Tokenizer |
| `chamber_parser` | Partial-safe parser |
| `chamber_ast` | AST types |
| `chamber_analyzer` | Lint rules |
| `chamber_formatter` | Code formatter |
| `chamber_diagnostics` | Error/warning types |
| `chamber_wasm` | WASM bindings |
| `chamber_cli` | CLI tool |

---

## What chamber is *not*

- A score renderer
- A playback engine
- A full ABC specification reference

chamber focuses on **language tooling**, not visualization.

---

## Roadmap

- [x] Partial-safe parser
- [x] Structured diagnostics (30+ codes)
- [x] Lint rules (4 rules)
- [x] Formatter (13 options)
- [x] WASM bindings
- [x] npm package (`chamber-abc`)
- [x] Tokenizer for syntax highlighting
- [ ] LSP server
- [ ] More lint rules
- [ ] Auto-fix suggestions

---

## Inspiration

- [Biome](https://biomejs.dev/)
- ESLint / Prettier
- rust-analyzer

Applied to **music notation**, not programming languages.

---

## License

MIT

---

> **chamber is the environment where ABC notation is written, examined, and refined — like chamber music itself.**
