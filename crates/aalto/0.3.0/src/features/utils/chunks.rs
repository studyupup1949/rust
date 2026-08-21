use crate::Grid;

pub fn fill_tiles_from_existing_chunk(
    new_chunk: &mut Grid,
    existing_chunk: &Grid,
    offset_x: usize,
    offset_y: usize,
    crop_ux: usize,
    crop_uy: usize,
    crop_lx: usize,
    crop_ly: usize,
) {
    let mut y = 0;
    for col in existing_chunk {
        let mut x = 0;
        for tile in col {
            if y < crop_ux && x < crop_uy && y >= crop_ly && x >= crop_lx {
                new_chunk[y + offset_y][x + offset_x] = tile.clone();
            }
            x += 1;
        }
        y += 1;
    }
}
pub fn mirror(chunk: &Grid, mirror_x: bool, mirror_y: bool, default_possible: Vec<crate::value::Value>) -> Grid {
    let empty_vec = [].to_vec();
    let h = chunk.len();
    let w = chunk.get(0).unwrap_or_else(|| &empty_vec).len();

    let mut mirrored_chunk = crate::reset_grid(w, h, default_possible);
    for y in 0..h {
        let my = if mirror_y { h - 1 - y } else { y };
        for x in 0..w {
            let mx = if mirror_x { w - 1 - x } else { x };

            mirrored_chunk[my][mx] = chunk[y][x].clone();
        }
    }

    mirrored_chunk
}

#[derive(Debug)]
pub enum PropagateDirection {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}
pub fn propagate_chunk(to: &mut Grid, from: &Grid, direction: PropagateDirection, default_possible: Vec<crate::value::Value>, max_recursions: usize) {
    let empty_vec = [].to_vec();
    let h = from.len();
    let w = from.get(0).unwrap_or_else(|| &empty_vec).len();

    // Vary offset depending on the orientation of the chunks
    let (offset_x, offset_y, mirror_x, mirror_y) = match direction {
        PropagateDirection::Up => (0, h, false, false),
        PropagateDirection::Right => (w, 0, false, false),
        PropagateDirection::Down => (0, h, false, true),
        PropagateDirection::Left => (w, 0, true, false),
    };
    // Vary size as well
    let mut working_copy = crate::reset_grid(w + offset_x, h + offset_y, default_possible.clone());

    fill_tiles_from_existing_chunk(
        &mut working_copy,
        &mirror(&to, mirror_x, mirror_y, default_possible.clone()),
        0,
        0,
        usize::MAX,
        usize::MAX,
        0,
        0,
    );
    fill_tiles_from_existing_chunk(
        &mut working_copy,
        &mirror(from, mirror_x, mirror_y, default_possible.clone()),
        offset_x,
        offset_y,
        usize::MAX,
        usize::MAX,
        0,
        0,
    );

    for y in 0..h {
        for x in 0..w {
            crate::propagate(
                &mut working_copy,
                x + offset_x,
                y + offset_y,
                max_recursions,
            );
        }
    }

    // Copy changes back to the original chunk
    let mirrored = mirror(&working_copy, mirror_x, mirror_y, default_possible);
    fill_tiles_from_existing_chunk(to, &mirrored, 0, 0, w, h, 0, 0);
}