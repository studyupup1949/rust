#![doc = include_str!("../README.md")]
pub use paste::paste;

/// Utility macro to declare advent of rust tests.
///
/// See crate-level documentation for more information
///
/// # Examples
///
/// ```ignore
/// advent_of_rust::advent_of_rust! {
///   1 => 1234, "Advent of Rust!";
///   2 => "hello";
/// }
/// ```
#[macro_export]
macro_rules! advent_of_rust {
    ($($day:expr => $part_1:expr $(, $part_2:expr)?);* $(;)?) => {
        use $crate::paste;

        paste! {
            $(mod [<day_ $day>];)*

            pub fn ignore_clippy_unused_errors() {
                $(
                    let _ = [<day_ $day>]::part_1;
                    $(
                        let _ = [<day_ $day>]::part_2;
                        let _ = $part_2;
                    )?
                )*
            }

            #[cfg(test)]
            mod tests {
                use super::*;
                $(
                    const [<DAY_ $day _INPUT>]: &str = include_str!(concat!("day_", $day, ".txt"));

                    #[test]
                    fn [<day_ $day _part_1>]() {
                        assert_eq!([<day_ $day>]::part_1([<DAY_ $day _INPUT>]), $part_1);
                    }

                    $(
                        #[test]
                        fn [<day_ $day _part_2>]() {
                            assert_eq!([<day_ $day>]::part_2([<DAY_ $day _INPUT>]), $part_2);
                        }
                    )?
                )*
            }
        }
    };
}
