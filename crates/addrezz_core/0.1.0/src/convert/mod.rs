#[cfg(feature = "url")]
mod url_impl;

#[cfg(feature = "http")]
mod http_impl;

// reqwest::Url is a re-export of url::Url, so when both features are on
// the `url` impls already cover reqwest. Only add reqwest-specific impls
// when the `url` feature is off.
#[cfg(all(feature = "reqwest", not(feature = "url")))]
mod reqwest_impl;
