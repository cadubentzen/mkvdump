#![doc = include_str!("../README.md")]

use clap::{Parser, ValueEnum};
use mkvdump::parse_elements_from_file;
use mkvparser::tree::build_element_trees;
use serde::Serialize;
use std::io::Write;

const DEFAULT_BUFFER_SIZE: u64 = 8192;

#[doc(hidden)]
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the MKV/WebM file to be parsed
    filename: String,

    /// Output format
    #[clap(value_enum, short, long, default_value = "yaml")]
    format: Format,

    /// Add element positions in the output
    #[clap(short = 'p', long)]
    show_element_positions: bool,

    /// Show output as a sequence, rather than a tree
    #[clap(short = 'l', long)]
    linear_output: bool,
}

#[doc(hidden)]
#[derive(ValueEnum, Clone, PartialEq, Eq)]
enum Format {
    Json,
    Yaml,
}

#[doc(hidden)]
fn print_serialized<T: Serialize>(elements: &[T], format: &Format) -> anyhow::Result<()> {
    let serialized = match format {
        Format::Json => serde_json::to_string_pretty(elements).unwrap(),
        Format::Yaml => serde_yaml::to_string(elements).unwrap(),
    };
    // BrokenPipe errors are ok, as they can come from piping the output
    // into other unix tools like less/head etc.
    // https://github.com/rust-lang/rust/issues/46016#issuecomment-1242039016
    match writeln!(std::io::stdout(), "{}", serialized) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::BrokenPipe => Ok(()),
        Err(e) => Err(e),
    }?;
    Ok(())
}

#[doc(hidden)]
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let elements = parse_elements_from_file(&args.filename, args.show_element_positions)?;

    if args.linear_output {
        print_serialized(&elements, &args.format)?;
    } else {
        let element_trees = build_element_trees(&elements);
        print_serialized(&element_trees, &args.format)?;
    }

    Ok(())
}
