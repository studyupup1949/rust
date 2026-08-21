pub trait StringExt {
  fn exclude(&self, st: &str) -> bool;
}

impl StringExt for String {
  fn exclude(&self, str: &str) -> bool {
    !self.as_str().contains(str)
  }
}

impl StringExt for &str {
  fn exclude(&self, str: &str) -> bool {
    !self.contains(str)
  }
}


// write test code
#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_exclude() {
    assert_eq!(true, "hello".exclude("world"));
    assert_eq!(false, "hello".exclude("hello"));

    assert_eq!(false, String::from("hello").exclude("hello"));
    assert_eq!(true, String::from("hello").exclude("world"));
  }
}
