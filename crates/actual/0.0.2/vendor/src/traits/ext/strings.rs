use std::str::FromStr;
pub trait FromJoinedStr<T> {
    fn parse_joined(&self, sep: &str) -> Vec<T>;
}

impl<T: FromStr> FromJoinedStr<T> for str {
    fn parse_joined(&self, sep: &str) -> Vec<T> {
        self.split(sep)
            .map(|piece| {
                piece
                    .trim()
                    .parse::<T>()
            })
            .collect::<Result<Vec<T>, _>>()
            .unwrap_or_default()
    }
}
