pub use adf_template_macro::template;

pub mod prelude {
    pub use super::GenerateADF;
    pub use adf_template_macro::template;
}

#[doc(hidden)]
pub mod __private {
    pub use htmltoadf::convert_html_str_to_adf_str;
    pub use serde;
    pub use tinytemplate;
}

pub trait GenerateADF {
    fn to_adf(&self) -> Result<String, Box<dyn std::error::Error>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    #[template(path = "../tests/template.html")]
    struct TestStruct {
        name: String,
        age: u32,
    }

    #[test]
    fn test_generate_adf() {
        let test_struct = TestStruct {
            name: "John".to_string(),
            age: 20,
        };
        let adf = test_struct.to_adf().unwrap();
        assert_eq!(
            adf,
            r#"{"version":1,"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"John"},{"type":"text","text":"20"}]}]}"#
        );
    }
}
