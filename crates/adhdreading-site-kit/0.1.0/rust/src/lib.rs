pub const BASE_URL: &str = "https://adhdreading.org";
pub const CHROME_WEB_STORE_URL: &str =
    "https://chromewebstore.google.com/detail/adhd-reading/dgihjimekmhphkbnnnomcbemhinmhmeg";

pub fn home_url() -> &'static str {
    BASE_URL
}

pub fn page_url(slug: &str) -> String {
    let clean = slug.trim_matches('/');
    if clean.is_empty() {
        BASE_URL.to_string()
    } else {
        format!("{BASE_URL}/{clean}")
    }
}

pub fn features_url() -> String { page_url("features") }

pub fn download_url() -> String { page_url("download") }

pub fn blog_url() -> String { page_url("blog") }

pub fn pricing_url() -> String { page_url("pricing") }

pub fn faq_url() -> String { page_url("faq") }

pub fn chrome_url() -> &'static str { CHROME_WEB_STORE_URL }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_links() {
        assert_eq!(home_url(), "https://adhdreading.org");
        assert_eq!(features_url(), "https://adhdreading.org/features");
        assert_eq!(download_url(), "https://adhdreading.org/download");
        assert_eq!(blog_url(), "https://adhdreading.org/blog");
        assert_eq!(pricing_url(), "https://adhdreading.org/pricing");
        assert_eq!(faq_url(), "https://adhdreading.org/faq");
        assert_eq!(
            chrome_url(),
            "https://chromewebstore.google.com/detail/adhd-reading/dgihjimekmhphkbnnnomcbemhinmhmeg"
        );
    }
}
