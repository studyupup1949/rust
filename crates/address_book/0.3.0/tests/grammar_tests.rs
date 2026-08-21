use address_book::{parse_phone_numbers};
#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test_parse_single_phone_number() -> Result<()> {
        let input = "123-456-7890";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_ok(),
            "Expected successful parsing for a single phone number"
        );
        Ok(())
    }

    #[test]
    fn test_parse_multiple_phone_numbers() -> Result<()> {
        let input = "123-456-7890, 555-123-4567";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_ok(),
            "Expected successful parsing for multiple phone numbers"
        );
        Ok(())
    }

    #[test]
    fn test_parse_identifier() -> Result<()> {
        let input = "abcde";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_ok(),
            "Expected successful parsing for a single identifier"
        );
        Ok(())
    }

    #[test]
    fn test_parse_date() -> Result<()> {
        let input = "2024-11-21";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_ok(),
            "Expected successful parsing for a valid date"
        );
        Ok(())
    }

    #[test]
    fn test_parse_mixed_fields() -> Result<()> {
        let input = "123-456-7890, abcde, 2024-11-21";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_ok(),
            "Expected successful parsing for mixed valid fields"
        );
        Ok(())
    }

    #[test]
    fn test_parse_with_whitespace() -> Result<()> {
        let input = "123-456-7890 ,    abcde ,\t2024-11-21";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_ok(),
            "Expected successful parsing for fields with whitespace"
        );
        Ok(())
    }

    #[test]
    fn test_parse_with_comment() -> Result<()> {
        let input = "123-456-7890, abcde # This is a comment";
        let result = parse_phone_numbers(input);
        assert!(
            result.is_ok(),
            "Expected successful parsing with a comment"
        );
        Ok(())
    }
    #[test]
fn test_parse_single_email() -> Result<()> {
    let input = "user@example.com";
    let result = parse_phone_numbers(input); 
    assert!(
        result.is_ok(),
        "Expected successful parsing for a single email"
    );
    Ok(())
}
#[test]
fn test_parse_single_url() -> Result<()> {
    let input = "http://www.example.com";
    let result = parse_phone_numbers(input); 
    assert!(
        result.is_ok(),
        "Expected successful parsing for a single URL"
    );
    Ok(())
}
}
