mod utils;

use mkvdump::{parse_element_or_skip_corrupted, tree::build_element_trees, Element};
use serde::Serialize;
use wasm_bindgen::prelude::*;

macro_rules! dlog {
    ( $( $t:tt )* ) => {
        #[cfg(debug_assertions)]
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

macro_rules! error {
    ( $( $t:tt )* ) => {
        web_sys::console::error_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub fn parse_mkv(input: &[u8]) -> Result<JsValue, JsValue> {
    utils::set_panic_hook();

    Ok(build_element_trees(&parse_elements(input)).serialize(
        &serde_wasm_bindgen::Serializer::json_compatible()
            // u64 doesn't fit into JSON so need to use BigInt
            .serialize_large_number_types_as_bigints(true),
    )?)
}

fn parse_elements(input: &[u8]) -> Vec<Element> {
    let mut elements = Vec::<Element>::new();
    let mut read_buffer = input;

    loop {
        match parse_element_or_skip_corrupted(read_buffer) {
            Ok((new_read_buffer, element)) => {
                dlog!("element: {:?}", element);
                elements.push(element);
                if new_read_buffer.is_empty() {
                    break;
                }
                read_buffer = new_read_buffer;
            }
            Err(e) => {
                error!("error: {}", e);
                break;
            }
        }
    }

    elements
}
