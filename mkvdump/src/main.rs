#![doc = include_str!("../../README.md")]

use std::{
    fs::File,
    io::{self, Read},
};

use clap::{Parser, ValueEnum};
use mkvparser::{parse_element_or_skip_corrupted, tree::build_element_trees, Body, Element};
use serde::Serialize;

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

// TODO: decide where to place this helper. Currently duplicated.
#[doc(hidden)]
fn parse_elements(input: &[u8], show_position: bool) -> Vec<Element> {
    let mut elements = Vec::<Element>::new();
    let mut read_buffer = input;
    let mut position = show_position.then_some(0);

    while let Ok((new_read_buffer, mut element)) = parse_element_or_skip_corrupted(read_buffer) {
        element.header.position = position;
        position = position.map(|p| {
            if let Body::Master = element.body {
                p + element.header.header_size
            } else {
                // It's safe to unwrap because all non-Master elements have a set size
                p + element.header.size.unwrap()
            }
        });
        elements.push(element);
        if new_read_buffer.is_empty() {
            break;
        }
        read_buffer = new_read_buffer;
    }
    elements
}

#[doc(hidden)]
fn print_serialized<T: Serialize>(elements: &[T], format: &Format) {
    let serialized = match format {
        Format::Json => serde_json::to_string_pretty(elements).unwrap(),
        Format::Yaml => serde_yaml::to_string(elements).unwrap(),
    };
    println!("{}", serialized);
}

#[doc(hidden)]
fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.filename)?;

    // TODO(#8): read chunked to not load entire file in memory.
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;

    let elements = parse_elements(&buffer, args.show_element_positions);
    if args.linear_output {
        print_serialized(&elements, &args.format);
    } else {
        let element_trees = build_element_trees(&elements);
        print_serialized(&element_trees, &args.format);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use mkvparser::{elements::Id, parse_element};

    use super::*;

    #[test]
    fn test_show_position() {
        const INPUT: &[u8] = include_bytes!("../tests/inputs/matroska-test-suite/test7.mkv");
        let elements = parse_elements(INPUT, true);
        for element in elements {
            // Corrupted elements won't match as we ignore their ID due to invalid content.
            if element.header.id == Id::Corrupted {
                continue;
            }
            let (_, element_at_position) =
                parse_element(&INPUT[element.header.position.unwrap() as usize..]).unwrap();
            assert_eq!(element_at_position.header.id, element.header.id);
        }
    }
}
