use crate::error::{AppError, AppResult};

const UNITS: [&str; 4] = ["daily", "weekly", "monthly", "weekdays"];
const MAX_INTERVAL: u32 = 999;

/// True for `unit:interval`, like `weekly:2`; see `shared/fixtures/recurrence/rules.json`.
pub fn is_valid(rule: &str) -> bool {
    let Some((unit, interval)) = rule.split_once(':') else {
        return false;
    };
    let digits_only = !interval.is_empty() && interval.bytes().all(|b| b.is_ascii_digit());
    if !UNITS.contains(&unit) || !digits_only || interval.starts_with('0') {
        return false;
    }
    let Ok(interval) = interval.parse::<u32>() else {
        return false;
    };
    interval <= MAX_INTERVAL && (unit != "weekdays" || interval == 1)
}

/// A blank rule means no recurrence.
pub fn clean(rule: Option<String>) -> AppResult<Option<String>> {
    match rule.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some(text) if is_valid(text) => Ok(Some(text.to_string())),
        Some(_) => Err(AppError::Invalid("unknown repeat rule")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn shared_rules() -> Value {
        let path = format!(
            "{}/../../shared/fixtures/recurrence/rules.json",
            env!("CARGO_MANIFEST_DIR")
        );
        serde_json::from_str(&std::fs::read_to_string(path).expect("read")).expect("parse")
    }

    fn list(value: &Value) -> Vec<&str> {
        value
            .as_array()
            .expect("array")
            .iter()
            .map(|v| v.as_str().expect("rule"))
            .collect()
    }

    #[test]
    fn accepts_and_rejects_the_shared_rules() {
        let rules = shared_rules();
        for rule in list(&rules["valid"]) {
            assert!(is_valid(rule), "{rule}");
        }
        for rule in list(&rules["invalid"]) {
            assert!(!is_valid(rule), "{rule:?}");
        }
        assert!(!is_valid("daily:+3"));
    }

    #[test]
    fn clean_trims_and_treats_blank_as_none() {
        assert_eq!(clean(None).expect("none"), None);
        assert_eq!(clean(Some(" ".into())).expect("blank"), None);
        assert_eq!(
            clean(Some(" weekly:2 ".into())).expect("rule").as_deref(),
            Some("weekly:2")
        );
        assert!(matches!(
            clean(Some("yearly:1".into())),
            Err(AppError::Invalid(_))
        ));
    }
}
