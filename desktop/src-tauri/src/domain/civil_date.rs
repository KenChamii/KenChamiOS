use crate::error::{AppError, AppResult};

const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// Unix ms of midnight UTC for a `YYYY-MM-DD` date; `None` when it is not a real calendar day.
pub fn parse(text: &str) -> Option<i64> {
    let mut parts = text.trim().split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=9999).contains(&year) || !(1..=12).contains(&month) {
        return None;
    }
    if day < 1 || day > days_in_month(year, month) {
        return None;
    }
    Some(days_from_civil(year, month, day) * DAY_MS)
}

/// A user or ui supplied day in the canonical `YYYY-MM-DD` form, so unique keys match across devices.
pub fn clean(day: &str) -> AppResult<String> {
    parse(day)
        .map(format)
        .ok_or(AppError::Invalid("day must be a date like 2026-11-10"))
}

/// The `YYYY-MM-DD` date (UTC) of a unix ms value.
pub fn format(ms: i64) -> String {
    let (year, month, day) = civil_from_days(ms.div_euclid(DAY_MS));
    format!("{year:04}-{month:02}-{day:02}")
}

fn days_in_month(year: i64, month: i64) -> i64 {
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

// days since 1970-01-01 (Howard Hinnant's civil calendar algorithm)
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip() {
        assert_eq!(parse("1970-01-01"), Some(0));
        assert_eq!(parse("2024-02-29"), Some(1_709_164_800_000));
        for text in ["1999-12-31", "2000-02-29", "2026-11-09", "0001-01-01"] {
            assert_eq!(format(parse(text).expect(text)), text);
        }
    }

    #[test]
    fn impossible_dates_are_rejected() {
        for text in [
            "",
            "2023-02-29",
            "2024-13-01",
            "2024-00-10",
            "2024-04-31",
            "0000-00-00",
            "2024-1",
            "2024-01-01-01",
            "abcd-01-01",
        ] {
            assert_eq!(parse(text), None, "{text}");
        }
    }

    #[test]
    fn times_before_the_epoch_format_correctly() {
        assert_eq!(format(-1), "1969-12-31");
    }
}
