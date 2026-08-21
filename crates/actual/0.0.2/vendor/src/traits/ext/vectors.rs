/// Vec<T> -> String. Can't fail: everything that implements ToString
/// always produces *some* string, so there's no error case to handle here.
pub trait ToJoinedString {
    fn to_joined_string(&self, sep: &str) -> String;
}

impl<T: ToString> ToJoinedString for [T] {
    fn to_joined_string(&self, sep: &str) -> String {
        self.iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(sep)
    }
}
