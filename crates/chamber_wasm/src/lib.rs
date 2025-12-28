//! WASM bindings for Chamber ABC notation toolkit.
//!
//! This crate provides JavaScript/TypeScript bindings for the Chamber
//! parser, analyzer, and formatter via WebAssembly.

use wasm_bindgen::prelude::*;

/// Parse ABC notation source code.
///
/// Returns a ParseResult containing the AST and any diagnostics.
#[wasm_bindgen]
pub fn parse(source: &str) -> JsValue {
    let result = chamber_parser::parse_with_diagnostics(source);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Analyze a tune for semantic issues.
///
/// Takes a Tune object (from parse result) and returns diagnostics.
#[wasm_bindgen]
pub fn analyze(tune_js: JsValue) -> JsValue {
    let tune: chamber_ast::Tune = match serde_wasm_bindgen::from_value(tune_js) {
        Ok(t) => t,
        Err(_) => return JsValue::NULL,
    };
    let result = chamber_analyzer::Analyzer::new().analyze(&tune);
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

/// Format ABC notation source code with custom configuration.
#[wasm_bindgen]
pub fn format(source: &str, config_js: JsValue) -> String {
    let config: chamber_formatter::FormatterConfig = match serde_wasm_bindgen::from_value(config_js) {
        Ok(c) => c,
        Err(_) => chamber_formatter::FormatterConfig::default(),
    };
    chamber_formatter::format(source, &config)
}

/// Format ABC notation source code with default configuration.
#[wasm_bindgen]
pub fn format_default(source: &str) -> String {
    chamber_formatter::format(source, &chamber_formatter::FormatterConfig::default())
}

/// Format ABC notation source code with minimal changes (cleanup only).
#[wasm_bindgen]
pub fn format_minimal(source: &str) -> String {
    chamber_formatter::format(source, &chamber_formatter::FormatterConfig::minimal())
}

/// Format ABC notation source code without any changes (passthrough).
#[wasm_bindgen]
pub fn format_passthrough(source: &str) -> String {
    chamber_formatter::format(source, &chamber_formatter::FormatterConfig::passthrough())
}

/// Get line and column information for a byte offset.
#[wasm_bindgen]
pub fn get_line_col(source: &str, offset: u32) -> JsValue {
    use chamber_diagnostics::LineIndex;
    use chamber_text_size::TextSize;

    let line_index = LineIndex::new(source);
    let line_col = line_index.line_col(TextSize::new(offset));

    serde_wasm_bindgen::to_value(&line_col).unwrap_or(JsValue::NULL)
}

/// Tokenize ABC notation source code for syntax highlighting.
///
/// Returns an array of tokens with kind and range.
#[wasm_bindgen]
pub fn tokenize(source: &str) -> JsValue {
    let tokens = chamber_lexer::Lexer::new(source).tokenize();
    serde_wasm_bindgen::to_value(&tokens).unwrap_or(JsValue::NULL)
}
