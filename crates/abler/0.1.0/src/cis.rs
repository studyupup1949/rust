pub struct Cis<'a>(pub &'a str);

impl PartialEq for Cis<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(other.0)
    }
}

impl Eq for Cis<'_> {}

impl phf::PhfHash for Cis<'_> {
    fn phf_hash<H: core::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.0.len());
        self.0.chars().filter_map(|c| c.to_lowercase().next()).for_each(|c| state.write_u32(c.into()));
    }
}

impl<'a, 'b: 'a> phf_shared::PhfBorrow<Cis<'a>> for Cis<'b> {
    fn borrow(&self) -> &Cis<'a> {
        self
    }
}

impl core::hash::Hash for Cis<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        phf::PhfHash::phf_hash(self, state);
    }
}

impl phf_shared::FmtConst for Cis<'_> {
    fn fmt_const(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "cis::Cis(\"{}\")", self.0)
    }
}