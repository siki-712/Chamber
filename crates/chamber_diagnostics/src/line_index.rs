use chamber_text_size::TextSize;

/// Index for converting byte offsets to line/column positions.
#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offset of each line start.
    line_starts: Vec<TextSize>,
}

/// A line/column position (0-indexed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LineCol {
    /// Line number (0-indexed).
    pub line: u32,
    /// Column number (0-indexed, in bytes).
    pub col: u32,
}

impl LineCol {
    /// Creates a new line/column position.
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }

    /// Returns 1-indexed line number for display.
    pub fn line_display(&self) -> u32 {
        self.line + 1
    }

    /// Returns 1-indexed column number for display.
    pub fn col_display(&self) -> u32 {
        self.col + 1
    }
}

impl LineIndex {
    /// Creates a new line index for the given source.
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![TextSize::new(0)];

        for (offset, c) in source.char_indices() {
            if c == '\n' {
                line_starts.push(TextSize::new((offset + 1) as u32));
            }
        }

        Self { line_starts }
    }

    /// Returns the number of lines in the source.
    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }

    /// Converts a byte offset to a line/column position.
    pub fn line_col(&self, offset: TextSize) -> LineCol {
        let line = self
            .line_starts
            .partition_point(|&start| start <= offset)
            .saturating_sub(1);

        let line_start = self.line_starts[line];
        let col = offset.raw() - line_start.raw();

        LineCol {
            line: line as u32,
            col,
        }
    }

    /// Returns the byte offset of the start of a line.
    pub fn line_start(&self, line: u32) -> Option<TextSize> {
        self.line_starts.get(line as usize).copied()
    }

    /// Returns the byte offset of the end of a line (before newline).
    pub fn line_end(&self, line: u32, source: &str) -> Option<TextSize> {
        // Validate line exists
        let _ = self.line_start(line)?;
        let next_start = self.line_start(line + 1);

        match next_start {
            Some(next) => {
                // Line ends before the newline character
                let end = next.raw() - 1;
                Some(TextSize::new(end))
            }
            None => {
                // Last line - ends at source end
                Some(TextSize::new(source.len() as u32))
            }
        }
    }

    /// Returns the text of a specific line.
    pub fn line_text<'a>(&self, line: u32, source: &'a str) -> Option<&'a str> {
        let start = self.line_start(line)?.raw() as usize;
        let end = self.line_end(line, source)?.raw() as usize;

        // Handle case where end might include newline
        let text = &source[start..end.min(source.len())];
        Some(text.trim_end_matches(['\r', '\n']))
    }
}

impl std::fmt::Display for LineCol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line_display(), self.col_display())
    }
}
