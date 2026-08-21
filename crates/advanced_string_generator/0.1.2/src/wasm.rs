use wasm_bindgen::prelude::*;
use super::RegexGenerator;

#[wasm_bindgen]
pub struct WasmRegexGenerator {
    generator: RegexGenerator,
}

#[wasm_bindgen]
impl WasmRegexGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new(pattern: &str, increment_value: Option<String>, array_values: Option<Vec<JsValue>>) -> WasmRegexGenerator {
        let array_values = array_values.map(|arr| {
            arr.into_iter().filter_map(|js_val| js_val.as_string()).collect()
        });

        WasmRegexGenerator {
            generator: RegexGenerator::new(pattern, increment_value, array_values),
        }
    }

    #[wasm_bindgen]
    pub fn generate(&mut self) -> String {
        self.generator.generate()
    }
}
