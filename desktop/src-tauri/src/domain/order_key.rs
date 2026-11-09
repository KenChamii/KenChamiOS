// fractional indexing: keys are base-62 strings that sort like fractions, so an
// item can always be placed between two neighbours without touching other rows
const DIGITS: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const BASE: usize = DIGITS.len();

fn digit_value(byte: u8) -> usize {
    DIGITS.iter().position(|&d| d == byte).unwrap_or(0)
}

/// Returns a key that sorts strictly between `before` and `after`.
/// An empty `before` means the start of the list, `None` for `after` the end.
pub fn between(before: &str, after: Option<&str>) -> String {
    midpoint(before.as_bytes(), after.map(str::as_bytes))
}

fn midpoint(before: &[u8], after: Option<&[u8]>) -> String {
    if let Some(after) = after {
        // missing digits in `before` count as zero
        let shared = after
            .iter()
            .enumerate()
            .take_while(|&(i, &b)| b == before.get(i).copied().unwrap_or(DIGITS[0]))
            .count();
        if shared > 0 {
            let prefix = String::from_utf8_lossy(&after[..shared]).into_owned();
            let rest_before = before.get(shared..).unwrap_or_default();
            return prefix + &midpoint(rest_before, Some(&after[shared..]));
        }
    }

    let low = before.first().map_or(0, |&b| digit_value(b));
    let high = after
        .and_then(|a| a.first())
        .map_or(BASE, |&b| digit_value(b));

    if high - low > 1 {
        return char::from(DIGITS[(low + high) / 2]).to_string();
    }
    match after {
        Some(after) if after.len() > 1 => char::from(after[0]).to_string(),
        _ => {
            let rest = before.get(1..).unwrap_or_default();
            char::from(DIGITS[low]).to_string() + &midpoint(rest, None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_key_is_in_the_middle() {
        assert_eq!(between("", None), "V");
    }

    #[test]
    fn appending_keeps_increasing() {
        let mut keys = vec![between("", None)];
        for _ in 0..200 {
            let next = between(keys.last().map(String::as_str).unwrap_or(""), None);
            keys.push(next);
        }
        assert!(keys.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn prepending_keeps_decreasing() {
        let mut keys = vec![between("", None)];
        for _ in 0..200 {
            let next = between("", keys.first().map(String::as_str));
            keys.insert(0, next);
        }
        assert!(keys.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn inserting_in_the_same_gap_never_collides() {
        let low = between("", None);
        let mut high = between(&low, None);
        for _ in 0..200 {
            let mid = between(&low, Some(&high));
            assert!(low < mid && mid < high, "{low} < {mid} < {high}");
            high = mid;
        }
    }

    #[test]
    fn handles_adjacent_digits() {
        let mid = between("a", Some("b"));
        assert!("a" < mid.as_str() && mid.as_str() < "b");
    }
}
