use std::{
    fs::File,
    io::{self, Read},
};

use clap::{Parser, ValueEnum};

use mkvdump::{parse_buffer_to_end, parse_elements};
use serde::Serialize;

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

#[derive(ValueEnum, Clone, PartialEq, Eq)]
enum Format {
    Json,
    Yaml,
}

fn print_serialized<T: Serialize>(elements: &[T], format: &Format) {
    let serialized = match format {
        Format::Json => serde_json::to_string_pretty(elements).unwrap(),
        Format::Yaml => serde_yaml::to_string(elements).unwrap(),
    };
    println!("{}", serialized);
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.filename)?;

    // TODO(#8): read chunked to not load entire file in memory.
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;

    if args.linear_output {
        let elements = parse_elements(&buffer, args.show_element_positions);
        print_serialized(&elements, &args.format);
    } else {
        let element_trees = parse_buffer_to_end(&buffer, args.show_element_positions);
        print_serialized(&element_trees, &args.format);
    }

    Ok(())
}
