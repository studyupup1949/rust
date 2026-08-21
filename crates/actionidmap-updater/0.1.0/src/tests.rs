#[cfg(test)]
use {crate::ActionIDMap, std::time::Duration};
#[cfg(test)]
static TEST_URL: &str = "https://gist.githubusercontent.com/sigaloid/e8319cf32d04a89c6b429513f62120c9/raw/ea2f9f11de1b26463a90d2da6a9163edaf6f05e4/gistfile1.txt";

#[test]
fn test_expiry_not_needed() {
    let actionmap = ActionIDMap::new(TEST_URL.to_string(), 5).unwrap();
    assert!(!actionmap.needs_refresh());
}
#[test]
fn test_expiry_needed() {
    let actionmap = ActionIDMap::new(TEST_URL.to_string(), 0).unwrap();
    std::thread::sleep(Duration::from_secs(1));
    assert!(actionmap.needs_refresh());
}
