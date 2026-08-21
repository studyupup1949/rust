use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[wasm_bindgen]
pub fn hello_world() -> String {
    return String::from("Hello world!")
}

#[wasm_bindgen]
pub fn ping(number: usize) -> String {
    format!("pong: {}", number)
}

#[wasm_bindgen]
pub fn sanity_127() -> usize {
    127 as usize
}

#[wasm_bindgen]
pub fn reset_grid(width: usize, height: usize, default_possible: &str) -> String {
    let default_possible_parsed: Vec<crate::value::Value> = serde_json::from_str(default_possible).unwrap();
    let result = crate::reset_grid(width, height, default_possible_parsed);
    serde_json::to_string(&result).unwrap()
}

#[cfg_attr(feature = "serde_derive", derive(Serialize, Deserialize))]
pub struct WasmGrid {
    pub tiles: crate::Grid
}

#[wasm_bindgen]
pub fn propagate(grid: &str, x: usize, y: usize, max_recursions: usize) -> String {
    let grid_parsed: WasmGrid = serde_json::from_str(grid).unwrap();
    let mut mgrid = grid_parsed.tiles.clone();
    crate::propagate(&mut mgrid, x, y, max_recursions);
    return serde_json::to_string(&WasmGrid{
        tiles: mgrid
    }).unwrap();
}

#[wasm_bindgen]
pub fn collapse(grid: &str, x: usize, y: usize, rng_seed: Option<u64>, max_recursions: usize) -> String {
    let grid_parsed: WasmGrid = serde_json::from_str(grid).unwrap();
    let mut mgrid = grid_parsed.tiles.clone();
    crate::collapse(&mut mgrid, x, y, rng_seed, max_recursions);
    return serde_json::to_string(&WasmGrid{
        tiles: mgrid
    }).unwrap();
}

#[wasm_bindgen]
pub fn choose_collapsable(grid: &str, rng_seed: Option<u64>) -> String {
    let grid_parsed: WasmGrid = serde_json::from_str(grid).unwrap();
    let mgrid = grid_parsed.tiles.clone();
    let result = crate::choose_collapsable(&mgrid, rng_seed);
    return serde_json::to_string(&result).unwrap();
}

#[wasm_bindgen]
pub fn collapse_all(grid: &str, rng_seed: Option<u64>, max_recursions: usize) -> String {
    let grid_parsed: WasmGrid = serde_json::from_str(grid).unwrap();
    let mut mgrid = grid_parsed.tiles.clone();
    
    let mut tile = crate::choose_collapsable(&mgrid, rng_seed);
    while tile.is_some() {
        let t = tile.unwrap();
        crate::collapse(&mut mgrid, t.0, t.1, rng_seed, max_recursions);
        tile = crate::choose_collapsable(&mgrid, rng_seed);
    }

    return serde_json::to_string(&WasmGrid{
        tiles: mgrid
    }).unwrap();
}

#[wasm_bindgen]
pub fn render(grid: &str) -> String {
    let grid_parsed: WasmGrid = serde_json::from_str(grid).unwrap();
    let mgrid = grid_parsed.tiles.clone();
    crate::render(&mgrid)
}