use chrono::NaiveDate;

/// Parse date string in multiple common formats
pub fn parse_date(date_str: &str) -> Option<NaiveDate> {
    // Try ISO format first (YYYY-MM-DD)
    if let Ok(date) = NaiveDate::parse_from_str(date_str.trim(), "%Y-%m-%d") {
        return Some(date);
    }

    // Try other common formats
    let formats = [
        "%B %d, %Y", // "January 15, 2024"
        "%b %d, %Y", // "Jan 15, 2024"
        "%d/%m/%Y",  // "15/01/2024"
        "%m/%d/%Y",  // "01/15/2024"
        "%Y-%m-%d",  // "2024-01-15" (duplicate but ensures we try again)
    ];

    for format in &formats {
        if let Ok(date) = NaiveDate::parse_from_str(date_str.trim(), format) {
            return Some(date);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[test]
    fn test_date_parsing_iso_format() {
        assert!(parse_date("2024-01-15").is_some());
        assert!(parse_date("2023-12-31").is_some());
        assert!(parse_date("2020-02-29").is_some()); // Leap year
    }

    #[test]
    fn test_date_parsing_long_month_format() {
        assert!(parse_date("January 15, 2024").is_some());
        assert!(parse_date("February 28, 2023").is_some());
        assert!(parse_date("March 1, 2024").is_some());
    }

    #[test]
    fn test_date_parsing_short_month_format() {
        assert!(parse_date("Jan 15, 2024").is_some());
        assert!(parse_date("Feb 28, 2023").is_some());
        assert!(parse_date("Mar 1, 2024").is_some());
    }

    #[test]
    fn test_date_parsing_slash_formats() {
        assert!(parse_date("15/01/2024").is_some()); // DMY
        assert!(parse_date("01/15/2024").is_some()); // MDY
        assert!(parse_date("31/12/2023").is_some()); // DMY
        assert!(parse_date("12/31/2023").is_some()); // MDY
    }

    #[test]
    fn test_date_parsing_with_whitespace() {
        assert!(parse_date(" 2024-01-15 ").is_some());
        assert!(parse_date("\t2024-01-15\n").is_some());
        assert!(parse_date(" January 15, 2024 ").is_some());
    }

    #[test]
    fn test_invalid_date_formats() {
        assert!(parse_date("").is_none());
        assert!(parse_date("invalid date").is_none());
        assert!(parse_date("15-01-2024").is_none()); // Wrong dash format
        assert!(parse_date("2024/01/15").is_none()); // Wrong slash format
        assert!(parse_date("Jan 15 2024").is_none()); // Missing comma
        assert!(parse_date("15th January 2024").is_none()); // Ordinal suffix
    }

    #[test]
    fn test_invalid_dates() {
        assert!(parse_date("2024-02-30").is_none()); // February 30th doesn't exist
        assert!(parse_date("2024-13-01").is_none()); // Month 13 doesn't exist
        assert!(parse_date("2024-00-01").is_none()); // Month 0 doesn't exist
        assert!(parse_date("2024-01-00").is_none()); // Day 0 doesn't exist
        assert!(parse_date("2021-02-29").is_none()); // Non-leap year Feb 29
    }

    #[test]
    fn test_edge_case_dates() {
        assert!(parse_date("2024-01-01").is_some()); // Start of year
        assert!(parse_date("2024-12-31").is_some()); // End of year
        assert!(parse_date("0001-01-01").is_some()); // Minimum date
        assert!(parse_date("9999-12-31").is_some()); // Maximum reasonable date
    }

    #[test]
    fn test_parsing_returns_correct_date() {
        let date = parse_date("2024-01-15").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);

        let date = parse_date("January 15, 2024").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);

        let date = parse_date("15/01/2024").unwrap();
        assert_eq!(date.year(), 2024);
        assert_eq!(date.month(), 1);
        assert_eq!(date.day(), 15);
    }

    #[test]
    fn test_month_name_case_insensitive() {
        // Test that month names work regardless of case (chrono handles this)
        assert!(parse_date("january 15, 2024").is_some());
        assert!(parse_date("JANUARY 15, 2024").is_some());
        assert!(parse_date("January 15, 2024").is_some());

        assert!(parse_date("jan 15, 2024").is_some());
        assert!(parse_date("JAN 15, 2024").is_some());
        assert!(parse_date("Jan 15, 2024").is_some());
    }

    #[test]
    fn test_format_priority() {
        // ISO format should be tried first and work
        let iso_date = parse_date("2024-01-15").unwrap();

        // If date can be parsed in multiple ways, should still be valid
        let slash_date = parse_date("01/15/2024").unwrap();
        assert_eq!(iso_date.month(), slash_date.month());
        assert_eq!(iso_date.day(), slash_date.day());
        assert_eq!(iso_date.year(), slash_date.year());
    }

    #[test]
    fn test_single_digit_formats() {
        assert!(parse_date("2024-1-5").is_some()); // ISO without leading zeros
        assert!(parse_date("1/5/2024").is_some()); // MDY without leading zeros
        assert!(parse_date("5/1/2024").is_some()); // DMY without leading zeros
    }

    #[test]
    fn test_excessive_whitespace() {
        assert!(parse_date("  2024-01-15  ").is_some());
        assert!(parse_date("\t\n2024-01-15\n\t").is_some());
        assert!(parse_date("  January   15,  2024  ").is_some());
    }
}
