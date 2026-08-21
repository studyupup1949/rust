
macro_rules! assert_diff_lt {
    ( $a:expr, $b:expr, $delta:expr) => {
        assert!(($a-$b).abs() < $delta, "|{}-{}| < {}", $a, $b, $delta)
    }
}

pub (super) use assert_diff_lt;
