pub(crate) struct MultiHashIterator {
    a: u64,
    b: u64,
    c: u64,
}

impl MultiHashIterator {
    pub(crate) fn new(a: u64, b: u64) -> Self {
        Self {
            a,
            b,
            c: Default::default(),
        }
    }
}

impl Iterator for MultiHashIterator {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = self.a;
        self.a = self.a.wrapping_add(self.b);
        self.b = self.b.wrapping_add(self.c);
        self.c += self.c.wrapping_add(1);
        Some(ret)
    }
}
