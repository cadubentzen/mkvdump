use std::{
    fs::File,
    io::{self, Read},
};

use clap::Parser;

use mkvdump::{parse_buffer_to_end, ElementTree};

/// mkvdump
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to be parsed
    filename: String,

    /// Output format
    #[clap(short, long, default_value = "yaml")]
    format: String,
}

fn print_element_trees(element_trees: &[ElementTree], format: &str) {
    let serialized = if format == "json" {
        serde_json::to_string_pretty(element_trees).unwrap()
    } else {
        serde_yaml::to_string(element_trees).unwrap()
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
