/// Marks a data type as compatible with the `ocr()` function. 

pub trait Scannable {
    
    /// Converts the type into a String that can be read by `ocr()`. 
    /// '#' is considered part of a letter and all other chars are considered blank space.
    /// 
    /// To implement `normalize()` for your own data type, translate your data into this
    /// format:
    /// 
    /// ```text
    /// .##..###...##.
    /// #..#.#..#.#..#
    /// #..#.###..#...
    /// ####.#..#.#...
    /// #..#.#..#.#..#
    /// #..#.###...##.
    /// ```
    fn normalize(&self) -> String;
}

impl Scannable for &str {
    fn normalize(&self) -> String {
        self.to_string()
    }
}

impl Scannable for (&Vec<bool>, usize) {
    fn normalize(&self) -> String {
        let (bools, width) = self;
        let width = width.clone();
        let mut output = String::new();
        let height = bools.len() / width;
        for y in 0..height {
            for x in 0..width {
                output.push(if bools[x + y * width] { '#' } else { '.' });
            }
            output.push('\n');
        }
        output
    }
}

impl Scannable for (&Vec<char>, usize) {
    fn normalize(&self) -> String {
        let (chars, width) = self;
        let width = width.clone();
        let mut output = String::new();
        let height = chars.len() / width;
        for y in 0..height {
            for x in 0..width {
                output.push(chars[x + y * width]);
            }
            output.push('\n');
        }
        output
    }
}

impl Scannable for &Vec<Vec<bool>> {
    fn normalize(&self) -> String {
        let mut output = String::new();
        for row in self.iter() {
            for &cell in row.iter() {
                output.push(if cell { '#' } else { '.' });
            }
            output.push('\n');
        }
        output
    }
}

impl Scannable for &Vec<Vec<char>> {
    fn normalize(&self) -> String {
        let mut output = String::new();
        for row in self.iter() {
            for &cell in row.iter() {
                output.push(cell);
            }
            output.push('\n');
        }
        output
    }
}

