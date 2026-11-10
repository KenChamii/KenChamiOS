use crate::error::{AppError, AppResult};
use serde_json::Value;

const TABLES: &str = include_str!("../../../../shared/schema/tables.schema.json");
const ENUMS: &str = include_str!("../../../../shared/schema/enums.schema.json");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Integer,
    Text,
}

#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub kind: Kind,
    pub nullable: bool,
    pub allowed_text: Option<Vec<String>>,
    pub allowed_int: Option<Vec<i64>>,
    pub string_array: bool,
}

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub columns: Vec<Column>,
    pub local_columns: Vec<String>,
    pub unique_live: Vec<String>,
    pub upsert_sql: String,
    pub export_sql: String,
}

impl Table {
    /// Columns that travel on the wire and are written on receive.
    pub fn shared_columns(&self) -> impl Iterator<Item = &Column> {
        self.columns
            .iter()
            .filter(|c| !self.local_columns.contains(&c.name))
    }
}

/// The tables that sync and their columns, read from `shared/schema` so both apps agree.
#[derive(Debug)]
pub struct SyncSchema {
    tables: Vec<Table>,
}

impl SyncSchema {
    pub fn parse() -> AppResult<Self> {
        let tables: Value = serde_json::from_str(TABLES)?;
        let enums: Value = serde_json::from_str(ENUMS)?;
        let defs = tables["$defs"].as_object().ok_or_else(invalid)?;
        let mut synced = Vec::new();
        for (name, def) in defs {
            if def["x-synced"].as_bool() == Some(true) {
                synced.push(parse_table(name, def, &enums)?);
            }
        }
        Ok(Self { tables: synced })
    }

    pub fn table(&self, name: &str) -> Option<&Table> {
        self.tables.iter().find(|t| t.name == name)
    }

    #[cfg(test)]
    pub fn tables(&self) -> &[Table] {
        &self.tables
    }
}

fn invalid() -> AppError {
    AppError::Sync("the bundled sync schema is malformed".to_string())
}

fn parse_table(name: &str, def: &Value, enums: &Value) -> AppResult<Table> {
    let properties = def["properties"].as_object().ok_or_else(invalid)?;
    let columns = properties
        .iter()
        .map(|(column, property)| parse_column(column, property, enums))
        .collect::<AppResult<Vec<_>>>()?;
    let local_columns = string_list(&def["x-local-columns"]);
    let mut table = Table {
        name: name.to_string(),
        columns,
        local_columns,
        unique_live: string_list(&def["x-unique-live"]),
        upsert_sql: String::new(),
        export_sql: String::new(),
    };
    table.upsert_sql = upsert_sql(&table);
    table.export_sql = export_sql(&table);
    Ok(table)
}

fn string_list(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn parse_column(name: &str, property: &Value, enums: &Value) -> AppResult<Column> {
    let variants: Vec<&Value> = match property.get("anyOf").and_then(Value::as_array) {
        Some(any_of) => any_of.iter().collect(),
        None => vec![property],
    };
    let mut column = Column {
        name: name.to_string(),
        kind: Kind::Text,
        nullable: false,
        allowed_text: None,
        allowed_int: None,
        string_array: property["x-encoding"].as_str() == Some("json-string-array"),
    };
    for variant in variants {
        if let Some(reference) = variant["$ref"].as_str() {
            let id = reference.rsplit('/').next().ok_or_else(invalid)?;
            column.allowed_text = Some(string_list(&enums["$defs"][id]["enum"]));
        }
        let types: Vec<&str> = match &variant["type"] {
            Value::String(t) => vec![t.as_str()],
            Value::Array(items) => items.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        for kind in types {
            match kind {
                "null" => column.nullable = true,
                "integer" => column.kind = Kind::Integer,
                "string" => column.kind = Kind::Text,
                _ => return Err(invalid()),
            }
        }
        if let Some(values) = variant["enum"].as_array() {
            column.allowed_int = Some(values.iter().filter_map(Value::as_i64).collect());
        }
    }
    Ok(column)
}

// local columns go out as null, each device keeps its own
fn export_sql(table: &Table) -> String {
    let pairs: Vec<String> = table
        .columns
        .iter()
        .map(|c| {
            if table.local_columns.contains(&c.name) {
                format!("'{}', NULL", c.name)
            } else {
                format!("'{0}', {0}", c.name)
            }
        })
        .collect();
    format!(
        "SELECT json_object({}) FROM {} WHERE id = ?",
        pairs.join(", "),
        table.name
    )
}

// last writer wins: the incoming row replaces the stored one only when it is newer
fn upsert_sql(table: &Table) -> String {
    let columns: Vec<&str> = table.shared_columns().map(|c| c.name.as_str()).collect();
    let placeholders = vec!["?"; columns.len()].join(", ");
    let assignments: Vec<String> = columns
        .iter()
        .filter(|c| **c != "id")
        .map(|c| format!("{c} = excluded.{c}"))
        .collect();
    format!(
        "INSERT INTO {name} ({cols}) VALUES ({placeholders})
         ON CONFLICT(id) DO UPDATE SET {assignments}
         WHERE excluded.updated_at > {name}.updated_at
            OR (excluded.updated_at = {name}.updated_at AND excluded.device_id > {name}.device_id)",
        name = table.name,
        cols = columns.join(", "),
        assignments = assignments.join(", "),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_the_thirteen_synced_tables() {
        let schema = SyncSchema::parse().expect("schema");
        let mut names: Vec<&str> = schema.tables().iter().map(|t| t.name.as_str()).collect();
        names.sort_unstable();
        assert_eq!(
            names,
            [
                "activity",
                "checklist_items",
                "habit_checks",
                "habits",
                "media_entries",
                "media_items",
                "media_progress_log",
                "project_docs",
                "projects",
                "prompts",
                "save_points",
                "tasks",
                "time_entries"
            ]
        );
    }

    #[test]
    fn local_columns_stay_out_of_the_upsert() {
        let schema = SyncSchema::parse().expect("schema");
        let items = schema.table("media_items").expect("media_items");
        assert!(!items.upsert_sql.contains("cover_cache_path"));
        assert!(items.upsert_sql.contains("cover_url"));
    }

    #[test]
    fn reads_enums_flags_and_nullable_columns() {
        let schema = SyncSchema::parse().expect("schema");
        let tasks = schema.table("tasks").expect("tasks");
        let column = |name: &str| tasks.columns.iter().find(|c| c.name == name).expect(name);
        assert_eq!(
            column("status").allowed_text.as_ref().map(Vec::len),
            Some(4)
        );
        assert_eq!(column("deleted").allowed_int, Some(vec![0, 1]));
        assert!(column("due_at").nullable);
        assert!(column("recurrence").nullable);
        assert!(column("tags").string_array);
    }
}
