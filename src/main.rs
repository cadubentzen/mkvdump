use std::{
    fs::File,
    io::{self, Read},
};

use clap::{Parser, ValueEnum};

use mkvdump::{parse_buffer_to_end, ElementTree};

/// Parse Matroska file and display result in serialized format.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the MKV/WebM file to be parsed
    filename: String,

    /// Output format
    #[clap(value_enum, short, long, default_value = "yaml")]
    format: Format,
}

#[derive(ValueEnum, Clone, PartialEq, Eq)]
enum Format {
    Json,
    Yaml,
}

fn print_element_trees(element_trees: &[ElementTree], format: &Format) {
    let serialized = match format {
        Format::Json => serde_json::to_string_pretty(element_trees).unwrap(),
        Format::Yaml => serde_yaml::to_string(element_trees).unwrap(),
    };
    println!("{}", serialized);
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.filename)?;

    // TODO(#8): read chunked to not load entire file in memory.
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;

    let element_trees = parse_buffer_to_end(&buffer);

    print_element_trees(&element_trees, &args.format);

    Ok(())
}
