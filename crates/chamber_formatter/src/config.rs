//! Formatter configuration options.

/// Configuration options for the ABC formatter.
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// Whether to normalize spacing between notes (single space).
    pub normalize_note_spacing: bool,

    /// Whether to ensure space around bar lines.
    pub space_around_bars: bool,

    /// Whether to align header field values.
    pub align_header_values: bool,

    /// Whether to remove trailing whitespace from lines.
    pub trim_trailing_whitespace: bool,

    /// Whether to ensure a newline at end of file.
    pub ensure_final_newline: bool,

    /// Whether to normalize header field order (X, T, C, M, L, Q, K).
    pub normalize_header_order: bool,

    /// Whether to normalize header field spacing (remove spaces around colon).
    /// `T : Value` -> `T:Value`
    pub normalize_header_spacing: bool,

    /// Whether to remove empty lines in the header section.
    pub remove_empty_header_lines: bool,

    /// Maximum line width for music lines (0 = no limit).
    pub max_line_width: usize,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            normalize_note_spacing: true,
            space_around_bars: true,
            align_header_values: false,
            trim_trailing_whitespace: true,
            ensure_final_newline: true,
            normalize_header_order: false,
            normalize_header_spacing: true,
            remove_empty_header_lines: true,
            max_line_width: 0,
        }
    }
}

impl FormatterConfig {
    /// Creates a new formatter config with all options disabled (passthrough mode).
    pub fn passthrough() -> Self {
        Self {
            normalize_note_spacing: false,
            space_around_bars: false,
            align_header_values: false,
            trim_trailing_whitespace: false,
            ensure_final_newline: false,
            normalize_header_order: false,
            normalize_header_spacing: false,
            remove_empty_header_lines: false,
            max_line_width: 0,
        }
    }

    /// Creates a config with minimal formatting (just cleanup).
    pub fn minimal() -> Self {
        Self {
            normalize_note_spacing: false,
            space_around_bars: false,
            align_header_values: false,
            trim_trailing_whitespace: true,
            ensure_final_newline: true,
            normalize_header_order: false,
            normalize_header_spacing: false,
            remove_empty_header_lines: false,
            max_line_width: 0,
        }
    }
}
