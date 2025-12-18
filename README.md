# Chamber

> **An opinionated language environment for ABC notation.**

**chamber** is a fast, partial-safe language toolkit for **writing, checking, and refining ABC notation**.
Inspired by tools like **Biome**, chamber treats ABC not as a file format, but as a **language** — one that deserves modern tooling.

Just like chamber music, it focuses on **structure, clarity, and interaction**, rather than scale or spectacle.

---

## Why chamber?

Most existing ABC tools are designed for **rendering or conversion**.
They assume the notation is already complete and correct.

**chamber is built for editing.**

* 🎼 Designed for *writing* ABC, not just compiling it
* 🧠 Partial-safe: works on incomplete or broken notation
* ⚡ Fast and deterministic (Rust core)
* 🧩 One shared AST for all features
* 🧭 Opinionated about how ABC should be written

If you are editing ABC in a browser or editor, **chamber is the missing foundation**.

---

## Core features

### ✅ Partial-safe parsing

chamber never crashes on unfinished input.

```abc
K:
C D E F | G
```

Still produces:

* an AST
* syntax highlights
* diagnostics

Editing always comes first.

---

### ✅ Single source of truth: AST

All functionality is built on a shared, typed AST:

* syntax highlighting
* diagnostics
* formatting
* lint rules
* future transformations

No duplicated logic. No regex-only hacks.

---

### ✅ Structured diagnostics

Errors are not just strings.

```text
abc/missing-key
Missing `K:` field.
```

Diagnostics are:

* span-aware
* severity-based (error / warning)
* rule-driven
* designed for future auto-fixes

---

### ✅ Opinionated formatting

```abc
C  D   E|F G
```

↓

```abc
C D E | F G
```

Formatting in chamber is:

* AST-based
* idempotent
* minimal-diff
* rule-oriented

---

### ✅ Editor-friendly syntax highlighting

chamber outputs **span-based highlights**, not colors.

This makes it editor-agnostic and suitable for:

* CodeMirror
* Monaco
* custom editors

---

## Philosophy

chamber follows three core principles:

### 1. Editing-first

The user is always in the middle of writing.
Incomplete input is the norm, not an error case.

---

### 2. Opinionated, but transparent

There *is* a recommended way to write ABC.
chamber encodes these opinions as rules and explains them clearly.

---

### 3. One core, many frontends

The same engine powers:

* WASM (browser)
* CLI
* LSP (planned)

---

## What chamber is *not*

* ❌ A score renderer
* ❌ A playback engine
* ❌ A full ABC specification reference

chamber focuses on **language tooling**, not visualization.

---

## Architecture (high level)

```
source text
    ↓
  lexer
    ↓
  parser (partial-safe)
    ↓
   AST
    ↓↓↓↓↓
 highlight · diagnostics · formatter · rules
```

---

## Planned usage

### Browser (WASM)

```ts
const result = chamber.analyze(source)

result.highlights
result.diagnostics
result.formatted
```

---

### CLI

```sh
chamber check tune.abc
chamber format tune.abc
```

---

## Status

🚧 **Work in progress**

Initial focus:

* core AST
* partial-safe parser
* diagnostics
* syntax highlighting

---

## Inspiration

* [Biome](https://biomejs.dev/)
* ESLint / Prettier
* rust-analyzer

Applied to **music notation**, not programming languages.

---

## License

MIT

---

## One-line pitch

> **chamber is the environment where ABC notation is written, examined, and refined — like chamber music itself.**

---
