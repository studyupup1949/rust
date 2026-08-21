use crate::algorithm::Printer;
use crate::heuristics;
use syn::File;

impl Printer {
    pub fn file(&mut self, file: &File) {
        self.cbox(0);
        if let Some(shebang) = &file.shebang {
            self.word(shebang.clone());
            self.hardbreak();
        }
        self.inner_attrs(&file.attrs);
        let mut prev: Option<&syn::Item> = None;
        for item in &file.items {
            if let Some(prev_item) = prev {
                if heuristics::should_blank_between_items(prev_item, item) {
                    self.hardbreak();
                }
            }
            self.item(item);
            prev = Some(item);
        }
        self.end();
    }
}
