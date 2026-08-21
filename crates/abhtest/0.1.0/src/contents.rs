/// Iterator over a finite range from 0 to n.
/// 
/// # Examples
///
/// ```
/// let r = abhtest::Arange::new(10);
/// for i in r {}
/// ```
pub struct Arange {
    pub n: i32,
    pub i: i32,
}

impl Arange {
    pub fn new(n: i32) -> Self {
        Self { n, i: 0 }
    }
}

impl Iterator for Arange {
    type Item = i32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.n {
            return None;
        }
        self.i += 1;
        Some(self.i-1)
    }
}
