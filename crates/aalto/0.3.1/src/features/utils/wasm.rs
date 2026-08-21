use wasm_bindgen::prelude::*;
use crate::{features::utils, wasm::WasmGrid};

#[wasm_bindgen]
pub fn fill_tiles_from_existing_chunk(
    existing_chunk: &str,
    new_chunk: &str,
    offset_x: usize,
    offset_y: usize,
    crop_ux: usize,
    crop_uy: usize,
    crop_lx: usize,
    crop_ly: usize,
) -> String {
    let grid_parsed: WasmGrid = serde_json::from_str(existing_chunk).unwrap();
    let mgrid = grid_parsed.tiles.clone();

    let newgrid_parsed: WasmGrid = serde_json::from_str(new_chunk).unwrap();
    let mut mnewgrid = newgrid_parsed.tiles.clone();

    utils::chunks::fill_tiles_from_existing_chunk(&mut mnewgrid, &mgrid, offset_x, offset_y, crop_ux, crop_uy, crop_lx, crop_ly);

    serde_json::to_string(&mnewgrid).unwrap()
}

#[wasm_bindgen]
pub fn mirror(chunk: &str, mirror_x: bool, mirror_y: bool, default_possible: &str) -> String {
    let default_possible_parsed: Vec<crate::value::Value> = serde_json::from_str(default_possible).unwrap();

    let grid_parsed: WasmGrid = serde_json::from_str(chunk).unwrap();
    let mut mgrid = grid_parsed.tiles.clone();

    utils::chunks::mirror(&mut mgrid, mirror_x, mirror_y, default_possible_parsed);

    serde_json::to_string(&mgrid).unwrap()
}

#[wasm_bindgen]
pub fn propagate_chunk(to: &str, from: &str, direction: usize, default_possible: &str, max_recursions: usize) -> String {
    let default_possible_parsed: Vec<crate::value::Value> = serde_json::from_str(default_possible).unwrap();

    let to_grid_parsed: WasmGrid = serde_json::from_str(to).unwrap();
    let mut grid_to = to_grid_parsed.tiles.clone();

    let from_grid_parsed: WasmGrid = serde_json::from_str(from).unwrap();
    let grid_from = from_grid_parsed.tiles.clone();

    let direction_enum = match direction {
        0 => utils::chunks::PropagateDirection::Up,
        1 => utils::chunks::PropagateDirection::Right,
        2 => utils::chunks::PropagateDirection::Down,
        3 => utils::chunks::PropagateDirection::Left,
        _ => panic!("Invalid propagate direction")
    };

    utils::chunks::propagate_chunk(&mut grid_to, &grid_from, direction_enum, default_possible_parsed, max_recursions);

    serde_json::to_string(&grid_to).unwrap()
}