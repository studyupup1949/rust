use ad_event_log_parser::*;
use anyhow::Result;

mod date {
    use super::*;
    use pest::Parser;

    #[test]
    fn valid_date() -> Result<()> {
        let pair = AdEventLogParser::parse(Rule::date, "2023-07-20")?
            .next()
            .unwrap();
        assert_eq!(pair.as_str(), "2023-07-20");
        Ok(())
    }

    #[test]
    fn invalid_date_format() {
        let parse_result = AdEventLogParser::parse(Rule::date, "20-07-2023");
        assert!(parse_result.is_err());
    }
}

mod time {
    use super::*;
    use pest::Parser;

    #[test]
    fn valid_time() -> Result<()> {
        let pair = AdEventLogParser::parse(Rule::time, "12:34:56")?
            .next()
            .unwrap();
        assert_eq!(pair.as_str(), "12:34:56");
        Ok(())
    }

    #[test]
    fn invalid_time_format() {
        let parse_result = AdEventLogParser::parse(Rule::time, "12:34");
        assert!(parse_result.is_err());
    }
}

mod uuid {
    use super::*;
    use pest::Parser;

    #[test]
    fn valid_uuid() -> Result<()> {
        let pair = AdEventLogParser::parse(Rule::uuid, "123e4567-e89b-12d3-a456-426614174000")?
            .next()
            .unwrap();
        assert_eq!(pair.as_str(), "123e4567-e89b-12d3-a456-426614174000");
        Ok(())
    }

    #[test]
    fn invalid_uuid_format() {
        let parse_result =
            AdEventLogParser::parse(Rule::uuid, "123e4567-zzzz-12d3-a456-426614174000");
        assert!(parse_result.is_err());
    }
}

mod ip {
    use super::*;
    use pest::Parser;

    #[test]
    fn valid_ip() -> Result<()> {
        let pair = AdEventLogParser::parse(Rule::ip, "192.168.1.1")?
            .next()
            .unwrap();
        assert_eq!(pair.as_str(), "192.168.1.1");
        Ok(())
    }
}

mod price {
    use super::*;
    use pest::Parser;

    #[test]
    fn valid_price() -> Result<()> {
        let pair = AdEventLogParser::parse(Rule::price, "$19.99")?
            .next()
            .unwrap();
        assert_eq!(pair.as_str(), "$19.99");
        Ok(())
    }

    #[test]
    fn invalid_price_format() {
        let parse_result = AdEventLogParser::parse(Rule::price, "p19.99");
        assert!(parse_result.is_err());
    }
}

#[test]
fn test_complete_log() {
    let log = "2023-07-20 12:34:56 123e4567-e89b-12d3-a456-426614174000 192.168.1.1 $19.99";
    let parsed = parse_ad_event(log).expect("Failed to parse log");
    assert_eq!(parsed.date, "2023-07-20");
    assert_eq!(parsed.time, "12:34:56");
    assert_eq!(parsed.uuid, "123e4567-e89b-12d3-a456-426614174000");
    assert_eq!(parsed.ip, "192.168.1.1");
    assert_eq!(parsed.price, 19.99);
}
