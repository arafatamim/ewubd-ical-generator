use wasm_bindgen::prelude::*;

use crate::generate_ics as generate_ics_impl;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn generate_ics(doc: &str) -> Result<String, String> {
    generate_ics_impl(doc).map_err(|e| e.to_string())
}
