//! Format an ABC file from command line
//!
//! Usage:
//!   cargo run -p chamber_formatter --example format_file -- <file.abc>
//!   cargo run -p chamber_formatter --example format_file -- <file.abc> --write

use chamber_formatter::{format, FormatterConfig};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.abc> [--write] [--passthrough]", args[0]);
        eprintln!();
        eprintln!("Options:");
        eprintln!("  --write        Write formatted output back to file");
        eprintln!("  --passthrough  Preserve original formatting");
        eprintln!("  --minimal      Minimal cleanup only");
        std::process::exit(1);
    }

    let file_path = &args[1];
    let write_back = args.contains(&"--write".to_string());
    let passthrough = args.contains(&"--passthrough".to_string());
    let minimal = args.contains(&"--minimal".to_string());

    // Read file
    let source = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file: {}", e);
            std::process::exit(1);
        }
    };

    // Choose config
    let config = if passthrough {
        FormatterConfig::passthrough()
    } else if minimal {
        FormatterConfig::minimal()
    } else {
        FormatterConfig::default()
    };

    // Format
    let formatted = format(&source, &config);

    if write_back {
        // Write back to file
        match fs::write(file_path, &formatted) {
            Ok(_) => println!("Formatted: {}", file_path),
            Err(e) => {
                eprintln!("Error writing file: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Print to stdout
        println!("=== Original ===");
        println!("{}", source);
        println!("=== Formatted ===");
        println!("{}", formatted);

        // Show diff
        if source != formatted {
            println!("=== Changes ===");
            for (i, (orig, fmt)) in source.lines().zip(formatted.lines()).enumerate() {
                if orig != fmt {
                    println!("Line {}: {:?} -> {:?}", i + 1, orig, fmt);
                }
            }
        } else {
            println!("(No changes)");
        }
    }
}
