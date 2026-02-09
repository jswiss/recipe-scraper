//! ISO 8601 duration parsing for recipe times.
//!
//! Parses durations in the format PT[n]H[n]M[n]S to minutes.
//! Examples:
//! - "PT15M" -> 15 minutes
//! - "PT1H30M" -> 90 minutes
//! - "PT2H" -> 120 minutes

/// Parses an ISO 8601 duration string to minutes.
///
/// Supports the format `PT[n]H[n]M[n]S` where:
/// - P = period designator (required)
/// - T = time designator (required for time components)
/// - H = hours
/// - M = minutes
/// - S = seconds (converted to minutes, rounded)
///
/// Returns None if the string is not a valid ISO 8601 duration.
pub fn parse_iso8601_duration(duration: &str) -> Option<u32> {
    let s = duration.trim();

    // Must start with PT
    if !s.starts_with("PT") {
        return None;
    }

    let s = &s[2..]; // Remove "PT" prefix

    // Empty after PT is invalid
    if s.is_empty() {
        return None;
    }

    let mut total_minutes: u32 = 0;
    let mut current_num = String::new();
    let mut found_any = false;

    for c in s.chars() {
        if c.is_ascii_digit() || c == '.' {
            current_num.push(c);
        } else {
            // Must have a number before the unit
            if current_num.is_empty() {
                return None;
            }
            let num: f64 = current_num.parse().ok()?;
            current_num.clear();
            found_any = true;

            match c {
                'H' => total_minutes += (num * 60.0) as u32,
                'M' => total_minutes += num as u32,
                'S' => total_minutes += (num / 60.0).round() as u32,
                _ => return None, // Invalid character
            }
        }
    }

    // If there's a remaining number without a unit, it's invalid
    if !current_num.is_empty() {
        return None;
    }

    // Must have found at least one time component
    if !found_any {
        return None;
    }

    Some(total_minutes)
}

/// Attempts to parse a duration from various formats.
///
/// Tries ISO 8601 first, then falls back to plain number parsing (assumed minutes).
pub fn parse_duration_flexible(duration: &str) -> Option<u32> {
    let s = duration.trim();

    // Try ISO 8601 first
    if let Some(minutes) = parse_iso8601_duration(s) {
        return Some(minutes);
    }

    // Try plain number (assumed minutes)
    if let Ok(minutes) = s.parse::<u32>() {
        return Some(minutes);
    }

    // Try extracting numbers from strings like "15 minutes" or "1 hour"
    let lower = s.to_lowercase();
    let numbers: Vec<u32> = s
        .chars()
        .filter(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
        .into_iter()
        .collect();

    if let Some(&num) = numbers.first() {
        if lower.contains("hour") {
            return Some(num * 60);
        }
        if lower.contains("min") {
            return Some(num);
        }
        // Just a number, assume minutes
        return Some(num);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minutes_only() {
        assert_eq!(parse_iso8601_duration("PT15M"), Some(15));
        assert_eq!(parse_iso8601_duration("PT30M"), Some(30));
        assert_eq!(parse_iso8601_duration("PT5M"), Some(5));
    }

    #[test]
    fn test_parse_hours_only() {
        assert_eq!(parse_iso8601_duration("PT1H"), Some(60));
        assert_eq!(parse_iso8601_duration("PT2H"), Some(120));
    }

    #[test]
    fn test_parse_hours_and_minutes() {
        assert_eq!(parse_iso8601_duration("PT1H30M"), Some(90));
        assert_eq!(parse_iso8601_duration("PT2H15M"), Some(135));
    }

    #[test]
    fn test_parse_with_seconds() {
        assert_eq!(parse_iso8601_duration("PT30S"), Some(1)); // 30s rounds to 1min
        assert_eq!(parse_iso8601_duration("PT1M30S"), Some(2)); // 1m + 30s rounds to 2min
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(parse_iso8601_duration(""), None);
        assert_eq!(parse_iso8601_duration("15"), None);
        assert_eq!(parse_iso8601_duration("P15M"), None); // Missing T
        assert_eq!(parse_iso8601_duration("PT"), None); // No value
        assert_eq!(parse_iso8601_duration("PTM"), None); // No number
    }

    #[test]
    fn test_flexible_iso8601() {
        assert_eq!(parse_duration_flexible("PT15M"), Some(15));
        assert_eq!(parse_duration_flexible("PT1H30M"), Some(90));
    }

    #[test]
    fn test_flexible_plain_number() {
        assert_eq!(parse_duration_flexible("15"), Some(15));
        assert_eq!(parse_duration_flexible("30"), Some(30));
    }

    #[test]
    fn test_flexible_text() {
        assert_eq!(parse_duration_flexible("15 minutes"), Some(15));
        assert_eq!(parse_duration_flexible("1 hour"), Some(60));
    }
}
