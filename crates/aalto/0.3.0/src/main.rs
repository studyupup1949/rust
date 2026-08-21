use aalto::{
    reset_grid,
    choose_collapsable,
    collapse,
    render,
    rules
};

fn count_collapsable_amount(grid: &aalto::Grid) -> usize {
    let mut count: usize = 0;
    for col in grid {
        for t in col {
            if t.possible.len() > 1 {
                count += 1;
            }
        }
    }

    count
}

fn main() {
    extraction_demo();
}

fn _solve_demo() {
    let verbose: bool = false;
    let h = 16;
    let w = 64;
    let mut grid = reset_grid(w, h, rules::dungeon());

    let mut tile = choose_collapsable(&grid, None);
    while tile.is_some() {
        if verbose {
            println!("{} tile(s) left", count_collapsable_amount(&grid));
        }
        let t = tile.unwrap();
        collapse(&mut grid, t.0, t.1, None, 500);
        tile = choose_collapsable(&grid, None);
    }
    
    println!("{}", render(&grid));
    //println!("{}", serde_json::to_string_pretty(&rules::checkers()).unwrap());
}

fn extraction_demo() {
    let image = aalto::features::extractor::load_local_image("tests/images/3Bricks.png".to_string()).unwrap();
    aalto::features::extractor::extract_from_image(image, 3);
}