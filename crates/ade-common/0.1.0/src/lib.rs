pub const INVALID_KEY_SEQUENCE: &str =
    "Invalid key sequence: keys must be sequential integers from 0 to n-1";

#[cfg(feature = "test-helpers")]
#[macro_export]
macro_rules! assert_panics_with {
    ($expr:expr, $expected:expr $(,)?) => {{
        let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $expr));
        match res {
            Ok(_) => panic!(
                "expected panic with {:?}, but code did not panic",
                $expected
            ),
            Err(err) => {
                let msg = err
                    .downcast_ref::<String>()
                    .map(String::as_str)
                    .or_else(|| err.downcast_ref::<&'static str>().copied());
                assert_eq!(msg, Some($expected), "panic message mismatch");
            }
        }
    }};
}
