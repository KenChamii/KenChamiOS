// the shared json schema must describe the tables the migrations really create
use super::open_in_memory;
use serde_json::Value;
use std::collections::BTreeSet;

fn schema(file: &str) -> Value {
    let path = format!("{}/../../shared/schema/{file}", env!("CARGO_MANIFEST_DIR"));
    let text = std::fs::read_to_string(path).expect("read schema");
    serde_json::from_str(&text).expect("parse schema")
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("array")
        .iter()
        .map(|v| v.as_str().expect("string").to_string())
        .collect()
}

// the values of `column IN ('a', 'b')` in a create table statement
fn check_values(create_sql: &str, column: &str) -> Vec<String> {
    let marker = format!("CHECK ({column} IN (");
    let start = create_sql.find(&marker).expect("check constraint") + marker.len();
    let end = start + create_sql[start..].find(')').expect("closing bracket");
    create_sql[start..end]
        .split(',')
        .map(|v| v.trim().trim_matches('\'').to_string())
        .collect()
}

#[tokio::test]
async fn tables_schema_matches_the_migrations() {
    let pool = open_in_memory().await.expect("open db");
    let tables = schema("tables.schema.json");
    let enums = schema("enums.schema.json");

    let in_db: BTreeSet<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'
            AND name != '_sqlx_migrations'",
    )
    .fetch_all(&pool)
    .await
    .expect("list tables")
    .into_iter()
    .collect();
    let in_schema: BTreeSet<String> = tables["$defs"]
        .as_object()
        .expect("defs")
        .keys()
        .cloned()
        .collect();
    assert_eq!(in_db, in_schema);

    for (table, def) in tables["$defs"].as_object().expect("defs") {
        let columns: Vec<(String, String, bool, bool)> = sqlx::query_as(
            "SELECT name, type, \"notnull\" != 0, pk != 0 FROM pragma_table_info(?)",
        )
        .bind(table)
        .fetch_all(&pool)
        .await
        .expect("table info");
        let properties = def["properties"].as_object().expect("properties");
        let required = strings(&def["required"]);
        assert_eq!(
            columns.iter().map(|c| c.0.clone()).collect::<BTreeSet<_>>(),
            properties.keys().cloned().collect::<BTreeSet<_>>(),
            "{table} columns"
        );

        let create_sql: String = sqlx::query_scalar("SELECT sql FROM sqlite_master WHERE name = ?")
            .bind(table)
            .fetch_one(&pool)
            .await
            .expect("create sql");
        for (name, sql_type, not_null, primary) in columns {
            let property = &properties[&name];
            assert_eq!(
                required.contains(&name),
                not_null || primary,
                "{table}.{name} required"
            );
            if let Some(reference) = property.get("$ref") {
                let id = reference
                    .as_str()
                    .expect("ref")
                    .rsplit('/')
                    .next()
                    .expect("id");
                assert_eq!(
                    strings(&enums["$defs"][id]["enum"]),
                    check_values(&create_sql, &name),
                    "{table}.{name} enum"
                );
            } else if property.get("anyOf").is_none() {
                let expected = if sql_type == "INTEGER" {
                    "integer"
                } else {
                    "string"
                };
                let actual = property["type"]
                    .as_str()
                    .or_else(|| property["type"][0].as_str());
                assert_eq!(actual, Some(expected), "{table}.{name} type");
            }
        }
    }
}
