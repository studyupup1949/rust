/// Pagination extractor
#[derive(Debug, Clone, Copy)]
pub struct Pagination {
    pub(crate) page: u32,
    pub(crate) per_page: u32,
}

impl Pagination {
    #[inline]
    pub fn offset(&self) -> u32 {
        self.page * self.per_page
    }

    #[inline]
    pub fn per_page(&self) -> u32 {
        self.per_page
    }

    #[inline]
    pub fn page(&self) -> u32 {
        self.page
    }
}
