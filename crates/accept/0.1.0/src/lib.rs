use std::fmt::Debug;

pub fn literal<'a, T: PartialEq + 'a, E: Debug>(
    to_take: &'a [T],
) -> impl Fn(&'a [T]) -> PResult<'a, &[T], T, E> + 'a {
    move |input| {
        if input.starts_with(to_take) {
            Ok((&input[to_take.len()..], to_take))
        } else {
            Err(CombinatorError::MissingTag)
        }
    }
}

pub fn any_of<'a, T: PartialEq, Value, E: Debug>(
    to_try: &'a [&'a dyn Fn(&[T]) -> PResult<'a, Value, T, E>],
) -> impl Fn(&[T]) -> PResult<'a, Value, T, E> {
    move |input| {
        for func in to_try {
            if let Ok(result) = func(input) {
                return Ok(result);
            }
        }
        Err(CombinatorError::NoFuncOk)
    }
}
/// Takes tokens until the inputted function returns an Ok result, consuming the `until` token
pub fn take_until<'a, T: Clone + 'a, UntilValue, E: Debug>(
    until: impl Fn(&'a [T]) -> PResult<UntilValue, T, E>,
) -> impl Fn(&'a [T]) -> (&[T], &[T]) {
    move |input| {
        for i in 0..input.len() {
            let slice = &input[i..];
            if let Ok((remainder, _)) = until(slice) {
                return (remainder, &input[..i]);
            }
        }
        (&[], input)
    }
}

pub fn take_n<T, E: Debug>(amount: usize) -> impl Fn(&[T]) -> PResult<&[T], T, E> {
    move |input| {
        let taken = input
            .get(..amount)
            .ok_or(CombinatorError::NotEnoughTokens)?;
        let remainder = input.get(amount..).unwrap_or(&[]);
        Ok((remainder, taken))
    }
}

pub fn split<'a, T: 'a, E: Debug, Value, DelimiterValue>(
    item: impl Fn(&'a [T]) -> PResult<'a, Value, T, E> + 'a,
    delimited_by: impl Fn(&'a [T]) -> PResult<'a, DelimiterValue, T, E> + 'a,
) -> impl Fn(&'a [T]) -> ((&[T], Vec<Value>), bool) + 'a {
    move |mut input| {
        let mut values = Vec::new();
        let mut has_seen_delimiter = false;
        let has_trailing = loop {
            input = match item(input) {
                Ok((remainder, value)) => {
                    values.push(value);
                    remainder
                }
                Err(_) => {
                    // if we are breaking on an input instead of a delimiter, that means we have a trailing delimiter
                    if has_seen_delimiter {
                        break true;
                    }
                    break false;
                }
            };
            input = match delimited_by(input) {
                Ok((remainder, _value)) => {
                    has_seen_delimiter = true;
                    remainder
                }
                Err(_) => break false,
            };
        };
        ((input, values), has_trailing)
    }
}
pub fn map_ok<'a, Value, T: 'a, E: Debug, MapTo>(
    parser: impl Fn(&'a [T]) -> PResult<Value, T, E> + 'a,
    predicate: impl Fn(Value) -> MapTo + 'a,
) -> impl Fn(&'a [T]) -> PResult<MapTo, T, E> {
    move |input| parser(input).map(|(remainder, value)| (remainder, predicate(value)))
}

/// Parses until error
pub fn many<T, Value, E: Debug>(
    parser: impl Fn(&[T]) -> PResult<Value, T, E>,
) -> impl Fn(&[T]) -> Vec<Value> {
    move |mut input| {
        let mut values = Vec::new();
        while let Ok((remainder, value)) = parser(input) {
            input = remainder;
            values.push(value);
        }
        values
    }
}
pub type PResult<'a, Value, Token, Error = ()> =
    Result<(&'a [Token], Value), CombinatorError<Error>>;
#[derive(Debug)]
pub enum CombinatorError<T: Debug> {
    Custom(T),
    MissingTag,
    NoFuncOk,
    NotEnoughTokens,
    UnexpectedTrailing,
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tag_works() {
        assert_eq!(
            literal::<i32, ()>(&[1_i32, 2])(&[1, 2, 3, 4]).unwrap(),
            ([3, 4].as_ref(), [1, 2].as_ref())
        )
    }
    #[test]
    fn tag_fails() {
        assert!(literal::<i32, ()>(&[1_i32, 100])(&[1, 2, 3, 4]).is_err())
    }
    #[test]
    fn test_take() {
        let chars_to_check = "abcd".chars().collect::<Vec<char>>();
        assert_eq!(
            take_n::<char, ()>(2)(&chars_to_check).unwrap().1,
            &['a', 'b']
        );
    }
    #[test]
    fn test_split() {
        let take_1 = take_n(1);
        let comma = literal(&[',']);
        let chars_to_check = "a,b,c,d".chars().collect::<Vec<char>>();
        let splitted = split::<'_, char, (), _, _>(take_1, comma);
        let ((tokens, parts), has_trailing) = splitted(&chars_to_check);
        println!("{parts:?} left over {tokens:?}");
        assert_eq!(parts[0][0], 'a');
        assert_eq!(parts[1][0], 'b');
        assert_eq!(parts[2][0], 'c');
        assert_eq!(parts[3][0], 'd');
        assert!(!has_trailing);
    }

    #[test]
    fn test_take_until() {
        assert_eq!(
            take_until::<_, _, ()>(literal(&[1]))(&[2, 3, 4, 1, 5]),
            (&[5][..], &[2, 3, 4][..])
        )
    }
}
