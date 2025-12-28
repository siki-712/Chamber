//! Example: Format an ABC tune
//!
//! Run with: cargo run -p chamber_formatter --example format_tune

use chamber_formatter::{format, FormatterConfig};

fn main() {
    // A messy ABC tune with inconsistent formatting
    let messy_tune = r#"X:1
T:The Kesh Jig
M:6/8
L:1/8
K:G
|:GAG GAB|ABA ABd|edd gdd|edB dBA|
GAG GAB|ABA ABd|edd gdB|AGF G3:|
"#;

    println!("=== Original (messy) ===");
    println!("{}", messy_tune);

    // Format with default settings
    println!("=== Formatted (default) ===");
    let formatted = format(messy_tune, &FormatterConfig::default());
    println!("{}", formatted);

    // Format with passthrough (no changes)
    println!("=== Passthrough ===");
    let passthrough = format(messy_tune, &FormatterConfig::passthrough());
    println!("{}", passthrough);

    // Format with minimal cleanup
    println!("=== Minimal ===");
    let minimal = format(messy_tune, &FormatterConfig::minimal());
    println!("{}", minimal);

    // Custom config: reorder headers
    println!("=== With header reordering ===");
    let wrong_order = "K:G\nT:Test\nX:1\nCDEF|";
    let config = FormatterConfig {
        normalize_header_order: true,
        ..FormatterConfig::default()
    };
    let reordered = format(wrong_order, &config);
    println!("Before: {}", wrong_order.replace('\n', " | "));
    println!("After:  {}", reordered.replace('\n', " | "));
}
