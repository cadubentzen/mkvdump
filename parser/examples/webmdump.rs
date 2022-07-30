use std::fs::File;
use std::io::{self, Read};

use clap::Parser;

use webm_parser::{parse_element, Element};

/// WebM dump
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the file to be dumped
    filename: String,

    /// Output in JSON format, rather than the default YAML
    #[clap(short, long, action = clap::ArgAction::SetTrue)]
    json: bool,
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut file = File::open(args.filename)?;

    // TODO: read chunked to not load entire video in memory.
    let mut buffer = Vec::<u8>::new();
    file.read_to_end(&mut buffer)?;

    let mut elements = Vec::<Element>::new();

    let mut read_buffer = &buffer[..];
    loop {
        match parse_element(read_buffer) {
            Ok((new_read_buffer, element)) => {
                elements.push(element);
                if new_read_buffer.is_empty() {
                    break;
                }
                read_buffer = new_read_buffer;
            }
            Err(nom::Err::Incomplete(needed)) => {
                println!(
                    "Needed: {:?}\nPartial result:\n{}",
                    needed,
                    serde_yaml::to_string(&elements).unwrap()
                );
                todo!("Partial reads not implemented")
            }
            Err(_) => {
                panic!("Something is wrong");
            }
        }
    }

    if args.json {
        println!("{}", serde_json::to_string_pretty(&elements).unwrap());
    } else {
        println!("{}", serde_yaml::to_string(&elements).unwrap());
    }

    Ok(())
}
