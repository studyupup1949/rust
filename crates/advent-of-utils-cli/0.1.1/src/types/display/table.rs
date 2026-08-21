pub trait Table {
    fn table_constructor(&self) -> TableStruct;
    fn reduced_table_constructor(&self) -> TableStruct;
    fn table(&self) {
        self.table_constructor().display();
    }

    fn reduced_table(&self) {
        self.reduced_table_constructor().display();
    }
}

pub struct TableStruct {
    contents: Vec<Vec<String>>,
    cell_widths: Vec<usize>,
    width: usize,
    height: usize,
}

impl TableStruct {
    pub fn new(contents: Vec<Vec<String>>) -> Self {
        let mut cell_widths: Vec<usize> = vec![];
        for x in 0..contents[0].len() {
            let mut width_max = 0;
            for y in contents.iter() {
                let width = y[x].len();
                if width > width_max {
                    width_max = width;
                }
            }
            cell_widths.push(width_max);
        }
        Self {
            width: contents[0].len(),
            height: contents.len(),
            contents,
            cell_widths,
        }
    }
    pub fn display(&self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let cell_width = self.cell_widths[x];
                let corner = match x {
                    0 => match y {
                        0 => " ┌",
                        _ => " ├",
                    },
                    _ => match y {
                        0 => "─┬",
                        _ => "─┼",
                    },
                };
                print!("{}─{:─^cell_width$}", corner, "");
            }
            let end = match y {
                0 => '┐',
                _ => '┤',
            };
            println!("─{end}");
            for x in 0..self.width {
                let cell_width = self.cell_widths[x];
                let content = self.contents[y][x].clone();
                print!(" │ {:^cell_width$}", content);
            }
            println!(" │")
        }
        for x in 0..self.width {
            let cell_width = self.cell_widths[x];
            let corner = match x {
                0 => " └",
                _ => "─┴",
            };
            print!("{}─{:─^cell_width$}", corner, "");
        }
        println!("─┘")
    }
}
