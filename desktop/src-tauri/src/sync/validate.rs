use super::schema::{Column, Kind, SyncSchema, Table};
use super::types::{Change, MAX_CHANGES, SyncError};
use serde_json::{Map, Value};

// a remote hlc this far ahead of the local clock is refused, see ADR 0005
const MAX_SKEW_MS: i64 = 60 * 60 * 1000;
const ID_MAX_LEN: usize = 64;
const HLC_COUNTER_BITS: u32 = 16;

fn reject<T>(message: impl Into<String>) -> Result<T, SyncError> {
    Err(SyncError::Rejected(message.into()))
}

pub fn is_valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= ID_MAX_LEN
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// Checks a whole batch; nothing is applied unless every change passes.
pub fn validate_changes(
    schema: &SyncSchema,
    changes: &[Change],
    now_ms: i64,
) -> Result<(), SyncError> {
    if changes.len() > MAX_CHANGES {
        return reject("too many changes in one batch");
    }
    for change in changes {
        let Some(table) = schema.table(&change.table) else {
            return reject("table is not synced");
        };
        validate_row(table, &change.row, now_ms)?;
    }
    Ok(())
}

fn validate_row(table: &Table, row: &Map<String, Value>, now_ms: i64) -> Result<(), SyncError> {
    if row.len() != table.columns.len() {
        return reject(format!("{}: wrong set of columns", table.name));
    }
    for column in &table.columns {
        let Some(value) = row.get(&column.name) else {
            return reject(format!("{}: missing column {}", table.name, column.name));
        };
        validate_value(table, column, value)?;
    }
    for key in ["id", "device_id"] {
        if !row
            .get(key)
            .and_then(Value::as_str)
            .is_some_and(is_valid_id)
        {
            return reject(format!("{}: invalid {key}", table.name));
        }
    }
    let hlc = row.get("updated_at").and_then(Value::as_i64).unwrap_or(-1);
    if hlc < 0 {
        return reject(format!("{}: invalid updated_at", table.name));
    }
    if (hlc >> HLC_COUNTER_BITS) > now_ms.saturating_add(MAX_SKEW_MS) {
        return reject(format!("{}: updated_at is too far ahead", table.name));
    }
    Ok(())
}

fn validate_value(table: &Table, column: &Column, value: &Value) -> Result<(), SyncError> {
    let bad = |what: &str| -> Result<(), SyncError> {
        reject(format!("{}.{}: {what}", table.name, column.name))
    };
    if value.is_null() {
        return if column.nullable { Ok(()) } else { bad("null") };
    }
    match column.kind {
        Kind::Integer => {
            let Some(number) = value.as_i64() else {
                return bad("not an integer");
            };
            if column
                .allowed_int
                .as_ref()
                .is_some_and(|allowed| !allowed.contains(&number))
            {
                return bad("value not allowed");
            }
        }
        Kind::Text => {
            let Some(text) = value.as_str() else {
                return bad("not a string");
            };
            if column
                .allowed_text
                .as_ref()
                .is_some_and(|allowed| !allowed.iter().any(|a| a == text))
            {
                return bad("value not allowed");
            }
            if column.string_array && serde_json::from_str::<Vec<String>>(text).is_err() {
                return bad("not a json array of strings");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::test_support::{fixture, rejected_cases};

    const NOW_MS: i64 = 1_800_000_000_000;

    fn changes_of(value: &Value) -> Vec<Change> {
        serde_json::from_value(value["changes"].clone()).expect("changes")
    }

    #[test]
    fn shared_fixtures_are_valid() {
        let schema = SyncSchema::parse().expect("schema");
        for name in ["push-valid", "pull-valid"] {
            let changes = changes_of(&fixture(name));
            validate_changes(&schema, &changes, NOW_MS).unwrap_or_else(|e| panic!("{name}: {e}"));
        }
    }

    #[test]
    fn shared_rejected_cases_are_refused() {
        let schema = SyncSchema::parse().expect("schema");
        for case in rejected_cases() {
            let changes = changes_of(&case);
            let result = validate_changes(&schema, &changes, NOW_MS);
            assert!(result.is_err(), "{} was accepted", case["name"]);
        }
    }

    #[test]
    fn hlc_skew_has_a_hard_edge() {
        let schema = SyncSchema::parse().expect("schema");
        let mut change = changes_of(&fixture("push-valid")).remove(0);
        let at = |ms: i64| Value::from(ms << HLC_COUNTER_BITS);

        change
            .row
            .insert("updated_at".into(), at(NOW_MS + MAX_SKEW_MS));
        assert!(validate_changes(&schema, std::slice::from_ref(&change), NOW_MS).is_ok());

        change
            .row
            .insert("updated_at".into(), at(NOW_MS + MAX_SKEW_MS + 1));
        assert!(validate_changes(&schema, std::slice::from_ref(&change), NOW_MS).is_err());
    }

    #[test]
    fn a_batch_over_the_limit_is_refused() {
        let schema = SyncSchema::parse().expect("schema");
        let one = changes_of(&fixture("push-valid")).remove(0);
        let batch = vec![one; MAX_CHANGES + 1];
        assert!(validate_changes(&schema, &batch, NOW_MS).is_err());
    }

    #[test]
    fn ids_are_limited_to_safe_characters() {
        assert!(is_valid_id("0190f0a2-7a1c-7c3e-9b52-1d3f6a8e4c10"));
        assert!(!is_valid_id(""));
        assert!(!is_valid_id("a b"));
        assert!(!is_valid_id(&"a".repeat(65)));
    }
}
