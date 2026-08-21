use lazy_static::lazy_static;
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

pub use rust_decimal::RoundingStrategy;

lazy_static! {
    /// The default list of abbreviation units.
    pub static ref ABBREVIATIONS: [&'static str; 7] = ["", "k", "M", "B", "T", "P", "E"];
}

/// The options for abbreviating a number.
#[derive(Debug, Default, Copy, Clone)]
pub struct Options<'a> {
    /// The precision of the result. `1` by default.
    pub precision: Option<u32>,
    /// A list of custom abbreviation units. [ABBREVIATIONS] is used by default.
    pub abbreviations: Option<[&'a str; 7]>,
    /// The [RoundingStrategy] to use on the result.
    /// [RoundingStrategy::MidpointNearestEven] is used by default.
    pub rounding_strategy: Option<RoundingStrategy>,
}

/// Abbreviates the given number into a human-friendly format according to specified
/// options.
///
/// # Arguments
///
/// * `number` - The integer to be abbreviated.
/// * `options` - An optional parameter specifying the [Options] for abbreviation.
///
/// # Returns
///
/// `Some(value)`, a string representation of the abbreviated form of the number.
/// Returns `None` if the number is out of bounds or cannot be abbreviated using the
/// provided abbreviations.
///
/// # Examples
///
/// ```
/// use abbrev_num::{abbrev_num, Options};
///
/// let options = Options {
///     precision: Some(3),
///     ..Default::default()
/// };
///
/// assert_eq!(abbrev_num(10_500, Some(options)), Some("10.5k".to_string()));
/// ```
pub fn abbrev_num(number: isize, options: Option<Options>) -> Option<String> {
    if number == 0 {
        return Some("0".to_string());
    }

    let options = options.unwrap_or_default();
    let absolute = number.unsigned_abs();
    let level = absolute.ilog10() / 3;
    let sign = if number.is_negative() { "-" } else { "" };
    let precision = options.precision.unwrap_or(1);

    let abbreviation = if let Some(abbreviations) = options.abbreviations {
        abbreviations.get(level as usize).copied()
    } else {
        ABBREVIATIONS.get(level as usize).copied()
    }?;

    if level == 0 {
        return Some(format!("{sign}{absolute}{abbreviation}"));
    }

    let result = absolute as f64 / 10_f64.powi(level as i32 * 3);
    let result = Decimal::from_f64(result)?.round_dp_with_strategy(
        precision,
        options
            .rounding_strategy
            .unwrap_or(RoundingStrategy::MidpointNearestEven),
    );

    Some(format!("{sign}{}{abbreviation}", result.normalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_abbreviate_numbers() {
        let fixtures: Vec<(isize, &str)> = vec![
            (0, "0"),
            (-0, "0"),
            (1, "1"),
            (10, "10"),
            (150, "150"),
            (999, "999"),
            (1_000, "1k"),
            (1_200, "1.2k"),
            (10_000, "10k"),
            (150_000, "150k"),
            (1_000_000, "1M"),
            (4_500_000, "4.5M"),
            (-10, "-10"),
            (-1_500, "-1.5k"),
        ];

        fixtures.iter().for_each(|(case, expected)| {
            let result = abbrev_num(*case, None);
            assert_eq!(result, Some(expected.to_string()));
        });
    }

    #[test]
    fn can_abbreviate_upto_a_certain_precision() {
        let result = abbrev_num(
            1_566_450,
            Some(Options {
                precision: Some(3),
                ..Default::default()
            }),
        );
        assert_eq!(result, Some("1.566M".to_string()));

        // Zero precision
        let result = abbrev_num(
            1_566_450,
            Some(Options {
                precision: Some(0),
                ..Default::default()
            }),
        );
        assert_eq!(result, Some("2M".to_string()));
    }

    #[test]
    fn can_abbreviate_using_custom_units() {
        let units: [&str; 7] = ["_c0", "_c1", "_c2", "_c3", "_c4", "_c5", "_c6"];
        let fixtures: Vec<(isize, &str)> = vec![
            (0, "0"),
            (10, "10_c0"),
            (1_000, "1_c1"),
            (1_000_000, "1_c2"),
            (1_000_000_000, "1_c3"),
            (1_000_000_000_000, "1_c4"),
            (1_000_000_000_000_000, "1_c5"),
            (1_000_000_000_000_000_000, "1_c6"),
        ];

        fixtures.iter().for_each(|(case, expected)| {
            let result = abbrev_num(
                *case,
                Some(Options {
                    abbreviations: Some(units),
                    ..Default::default()
                }),
            );
            assert_eq!(result, Some(expected.to_string()));
        });
    }
}
