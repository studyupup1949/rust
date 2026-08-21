use crate::Pagination;

/// Configuration for pagination
#[derive(Debug, Clone, Copy)]
pub struct PaginationConfig {
    pub(crate) page_name: &'static str,
    pub(crate) per_page_name: &'static str,
    pub(crate) default_per_page: i32,
    pub(crate) max_per_page: i32,
}

impl PaginationConfig {
    pub(crate) const DEFAULT_PER_PAGE: u32 = 20;
    pub(crate) const DEFAULT: Self = PaginationConfig {
        page_name: "page",
        per_page_name: "per_page",
        default_per_page: Self::DEFAULT_PER_PAGE as _,
        max_per_page: 100,
    };

    pub(crate) fn pagination(&self) -> Pagination {
        Pagination {
            per_page: self.default_per_page as _,
            page: 0,
        }
    }

    pub(crate) fn parse_page(&self, text: &str) -> i32 {
        text.parse().unwrap_or(1).max(1) - 1
    }

    pub(crate) fn parse_per_page(&self, text: &str) -> i32 {
        text.parse()
            .unwrap_or(self.default_per_page)
            .max(self.default_per_page)
            .min(self.max_per_page)
    }

    /// Set query name for `page`, default `"page"`
    pub fn page_name(mut self, page_name: &'static str) -> Self {
        self.page_name = page_name;
        self
    }

    /// Set query name for `per_page`, default `"per_page"`
    pub fn per_page_name(mut self, per_page_name: &'static str) -> Self {
        self.per_page_name = per_page_name;
        self
    }

    /// Set default `per_page`, default `20`
    pub fn default_per_page(mut self, default_per_page: u32) -> Self {
        debug_assert!(default_per_page >= Self::DEFAULT_PER_PAGE);
        self.default_per_page = default_per_page as _;
        self
    }

    /// Set max `per_page`, default `100`
    pub fn max_per_page(mut self, max_per_page: u32) -> Self {
        debug_assert!(max_per_page >= Self::DEFAULT_PER_PAGE);
        self.max_per_page = max_per_page as _;
        self
    }
}

impl Pagination {
    /// Create [`PaginationConfig`] for pagination
    pub fn config() -> PaginationConfig {
        PaginationConfig::DEFAULT
    }
}
