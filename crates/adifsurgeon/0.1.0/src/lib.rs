use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};

pub fn parse_timestamp(input: &str) -> Result<DateTime<Utc>, &'static str> {
    match input.len() {
        // Format: YYYYMMDD
        8 => {
            let year = input[0..4].parse::<i32>().map_err(|_| "Invalid year")?;
            let month = input[4..6].parse::<u32>().map_err(|_| "Invalid month")?;
            let day = input[6..8].parse::<u32>().map_err(|_| "Invalid day")?;
            
            match NaiveDate::from_ymd_opt(year, month, day) {
                Some(date) => {
                    match date.and_hms_opt(0, 0, 0) {
                        Some(datetime) => Ok(Utc.from_utc_datetime(&datetime)),
                        None => Err("Invalid time creation"),
                    }
                },
                None => Err("Invalid date"),
            }
        },
        
        // Format: YYYYMMDDhhmmss
        14 => {
            let year = input[0..4].parse::<i32>().map_err(|_| "Invalid year")?;
            let month = input[4..6].parse::<u32>().map_err(|_| "Invalid month")?;
            let day = input[6..8].parse::<u32>().map_err(|_| "Invalid day")?;
            let hour = input[8..10].parse::<u32>().map_err(|_| "Invalid hour")?;
            let minute = input[10..12].parse::<u32>().map_err(|_| "Invalid minute")?;
            let second = input[12..14].parse::<u32>().map_err(|_| "Invalid second")?;
            
            let date = NaiveDate::from_ymd_opt(year, month, day)
                .ok_or("Invalid date")?;
            let time = chrono::NaiveTime::from_hms_opt(hour, minute, second)
                .ok_or("Invalid time")?;
            
            let dt = NaiveDateTime::new(date, time);
            Ok(Utc.from_utc_datetime(&dt))
        },
        
        _ => Err("Invalid timestamp format"),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;
    use chrono::Timelike;

    #[test]
    fn test_valid_date_only() {
        let result = parse_timestamp("20230415").unwrap();
        assert_eq!(result.year(), 2023);
        assert_eq!(result.month(), 4);
        assert_eq!(result.day(), 15);
        assert_eq!(result.hour(), 0);
        assert_eq!(result.minute(), 0);
        assert_eq!(result.second(), 0);
    }

    #[test]
    fn test_valid_full_timestamp() {
        let result = parse_timestamp("20230415123045").unwrap();
        assert_eq!(result.year(), 2023);
        assert_eq!(result.month(), 4);
        assert_eq!(result.day(), 15);
        assert_eq!(result.hour(), 12);
        assert_eq!(result.minute(), 30);
        assert_eq!(result.second(), 45);
    }

    #[test]
    fn test_invalid_length() {
        assert!(parse_timestamp("2023").is_err());
        assert!(parse_timestamp("202304151230").is_err());
    }

    #[test]
    fn test_invalid_date_components() {
        // Invalid month (13)
        assert!(parse_timestamp("20231315").is_err());
        // Invalid day (32)
        assert!(parse_timestamp("20230132").is_err());
        // Invalid day for month (April 31)
        assert!(parse_timestamp("20230431").is_err());
        // Invalid February 29 in non-leap year
        assert!(parse_timestamp("20230229").is_err());
    }

    #[test]
    fn test_invalid_time_components() {
        // Invalid hour (25)
        assert!(parse_timestamp("20230415250045").is_err());
        // Invalid minute (60)
        assert!(parse_timestamp("20230415126045").is_err());
        // Invalid second (60)
        assert!(parse_timestamp("20230415123060").is_err());
    }

    #[test]
    fn test_non_numeric_input() {
        assert!(parse_timestamp("2023041a").is_err());
        assert!(parse_timestamp("2023041512304a").is_err());
    }
    
    #[test]
    fn test_leap_year() {
        // February 29 in leap year (2024)
        let result = parse_timestamp("20240229").unwrap();
        assert_eq!(result.year(), 2024);
        assert_eq!(result.month(), 2);
        assert_eq!(result.day(), 29);
    }
}
