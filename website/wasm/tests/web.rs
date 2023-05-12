//! Test suite for the Web and headless browsers.

#![cfg(target_arch = "wasm32")]

use mkvdump_wasm::parse;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {}
