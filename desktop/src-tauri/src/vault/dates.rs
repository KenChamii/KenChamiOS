const DAY_MS: i64 = 86_400_000;

/// Days since 1970-01-01 of a civil date.
pub fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year.rem_euclid(400);
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The civil date of a day count since 1970-01-01.
pub fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days.rem_euclid(146_097);
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

pub fn day_string(days: i64) -> String {
    let (year, month, day) = civil_from_days(days);
    format!("{year:04}-{month:02}-{day:02}")
}

/// The UTC day count of a unix time in milliseconds.
pub fn days_of(ms: i64) -> i64 {
    ms.div_euclid(DAY_MS)
}

/// Days since 1970-01-01 of `YYYY-MM-DD`, or `None` when it is not a real date.
pub fn parse_day(text: &str) -> Option<i64> {
    let mut parts = text.split('-');
    let (year, month, day) = (
        parts.next()?.parse::<i64>().ok()?,
        parts.next()?.parse::<i64>().ok()?,
        parts.next()?.parse::<i64>().ok()?,
    );
    let valid = parts.next().is_none()
        && text.len() == 10
        && (1..=12).contains(&month)
        && (1..=31).contains(&day);
    let days = days_from_civil(year, month, day);
    (valid && civil_from_days(days) == (year, month, day)).then_some(days)
}

/// ISO year and week number of a day count.
pub fn iso_week(days: i64) -> (i64, i64) {
    // 1970-01-01 was a Thursday; monday = 0
    let weekday = (days + 3).rem_euclid(7);
    let thursday = days - weekday + 3;
    let (year, _, _) = civil_from_days(thursday);
    let week = (thursday - days_from_civil(year, 1, 1)) / 7 + 1;
    (year, week)
}

/// The Monday of the ISO week that holds `days`.
pub fn monday_of(days: i64) -> i64 {
    days - (days + 3).rem_euclid(7)
}

pub fn day_start_ms(days: i64) -> i64 {
    days * DAY_MS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_dates_round_trip() {
        for days in [-1000, 0, 365, 20_777, 25_000] {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days_from_civil(y, m, d), days);
        }
        assert_eq!(day_string(20_777), "2026-11-20");
    }

    #[test]
    fn only_real_days_parse() {
        assert_eq!(parse_day("2026-11-20"), Some(20_777));
        assert_eq!(parse_day("2026-02-30"), None);
        assert_eq!(parse_day("2026-1-5"), None);
        assert_eq!(parse_day("nope"), None);
    }

    #[test]
    fn iso_weeks_follow_the_standard() {
        let week = |text: &str| iso_week(parse_day(text).expect("day"));
        assert_eq!(week("2026-01-01"), (2026, 1));
        assert_eq!(week("2024-12-30"), (2025, 1));
        assert_eq!(week("2021-01-03"), (2020, 53));
        assert_eq!(week("2026-11-16"), (2026, 47));
    }

    #[test]
    fn monday_is_the_start_of_the_week() {
        let monday = monday_of(parse_day("2026-11-19").expect("day"));
        assert_eq!(day_string(monday), "2026-11-16");
    }
}
