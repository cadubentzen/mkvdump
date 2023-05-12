mod utils;

use mkvdump::{parse_element, parse_element_or_skip_corrupted, tree::build_element_trees, Element};
use serde::Serialize;
use wasm_bindgen::prelude::*;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub fn parse_mkv(input: &[u8]) -> JsValue {
    utils::set_panic_hook();

    log!("input size: {}", input.len());

    build_element_trees(&parse_elements(input))
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap()
}

fn parse_elements(input: &[u8]) -> Vec<Element> {
    let mut elements = Vec::<Element>::new();
    let mut read_buffer = input;

    loop {
        match parse_element(read_buffer) {
            Ok((new_read_buffer, element)) => {
                log!("new element: {:?}", element.header.id);
                elements.push(element);
                if new_read_buffer.is_empty() {
                    break;
                }
                read_buffer = new_read_buffer;
            }
            Err(e) => {
                log!("error: {}", e);
                break;
            }
        }
    }

    elements
}
