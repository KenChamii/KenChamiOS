// the change log is filled by triggers, these tests fail when a synced table is left out
use super::schema::SyncSchema;
use super::test_support::fixture;
use super::types::Change;
use crate::db::{Db, projects, tasks};
use serde_json::Value;
use sqlx::AssertSqlSafe;

async fn logged(db: &Db, table: &str, id: &str) -> Option<(i64, i64)> {
    sqlx::query_as("SELECT seq, hlc FROM change_log WHERE table_name = ? AND row_id = ?")
        .bind(table)
        .bind(id)
        .fetch_optional(&db.pool)
        .await
        .expect("read log")
}

async fn insert_raw(db: &Db, schema: &SyncSchema, change: &Change) {
    let table = schema.table(&change.table).expect("synced table");
    let columns: Vec<&str> = table.columns.iter().map(|c| c.name.as_str()).collect();
    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table.name,
        columns.join(", "),
        vec!["?"; columns.len()].join(", ")
    );
    let mut query = sqlx::query(AssertSqlSafe(sql));
    for column in &columns {
        query = match &change.row[*column] {
            Value::String(text) => query.bind(text.as_str()),
            other => query.bind(other.as_i64()),
        };
    }
    query.execute(&db.pool).await.expect("insert row");
}

#[tokio::test]
async fn every_synced_table_logs_inserts_and_updates() {
    let db = Db::in_memory().await;
    let schema = SyncSchema::parse().expect("schema");
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&db.pool)
        .await
        .expect("pragma");
    let changes: Vec<Change> =
        serde_json::from_value(fixture("push-valid")["changes"].clone()).expect("changes");
    assert_eq!(
        changes.len(),
        schema.tables().len(),
        "one fixture row per synced table"
    );

    for change in &changes {
        insert_raw(&db, &schema, change).await;
        let id = change.row["id"].as_str().expect("id");
        let (seq, hlc) = logged(&db, &change.table, id)
            .await
            .unwrap_or_else(|| panic!("{} insert was not logged", change.table));
        assert_eq!(Some(hlc), change.row["updated_at"].as_i64());

        sqlx::query(AssertSqlSafe(format!(
            "UPDATE {} SET updated_at = updated_at + 5 WHERE id = ?",
            change.table
        )))
        .bind(id)
        .execute(&db.pool)
        .await
        .expect("update row");
        let (new_seq, new_hlc) = logged(&db, &change.table, id)
            .await
            .unwrap_or_else(|| panic!("{} update was not logged", change.table));
        assert!(
            new_seq > seq && new_hlc == hlc + 5,
            "{} update",
            change.table
        );
    }
}

#[tokio::test]
async fn the_log_has_one_current_entry_per_row_after_repository_writes() {
    let db = Db::in_memory().await;
    let schema = SyncSchema::parse().expect("schema");
    let project = projects::create_for_test(&db, "Alpha").await;
    let task = tasks::create(&db, tasks::create_input(&project.id, "Write"))
        .await
        .expect("task");
    projects::set_stage(&db, &project.id, crate::domain::project::Stage::Build)
        .await
        .expect("stage");
    projects::delete(&db, &project.id).await.expect("delete");

    for table in schema.tables() {
        let unlogged: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
            "SELECT COUNT(*) FROM {0} t WHERE NOT EXISTS
               (SELECT 1 FROM change_log l WHERE l.table_name = '{0}' AND l.row_id = t.id AND l.hlc = t.updated_at)",
            table.name
        )))
        .fetch_one(&db.pool)
        .await
        .expect("count");
        assert_eq!(
            unlogged, 0,
            "{} has rows without a current log entry",
            table.name
        );
    }
    let (_, hlc) = logged(&db, "tasks", &task.id).await.expect("task logged");
    let deleted: (i64, i64) = sqlx::query_as("SELECT deleted, updated_at FROM tasks WHERE id = ?")
        .bind(&task.id)
        .fetch_one(&db.pool)
        .await
        .expect("task row");
    assert_eq!(deleted, (1, hlc), "the cascade delete is logged");
}

#[tokio::test]
async fn local_only_tables_are_never_logged() {
    let db = Db::in_memory().await;
    crate::db::settings::set_string(&db, "k", "v")
        .await
        .expect("setting");

    let logged_tables: Vec<String> =
        sqlx::query_scalar("SELECT DISTINCT table_name FROM change_log")
            .fetch_all(&db.pool)
            .await
            .expect("tables");
    assert!(logged_tables.is_empty(), "{logged_tables:?}");
}

#[tokio::test]
async fn rows_that_exist_before_the_migration_are_back_filled() {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .in_memory(true)
                .foreign_keys(true),
        )
        .await
        .expect("connect");
    let mut old = sqlx::migrate!("./migrations");
    old.migrations = old
        .migrations
        .iter()
        .filter(|m| m.version < 11)
        .cloned()
        .collect();
    old.run(&pool).await.expect("old schema");
    sqlx::query(
        "INSERT INTO projects (id, created_at, updated_at, device_id, name, slug)
         VALUES ('p1', 1, 77, 'd', 'Old', 'old')",
    )
    .execute(&pool)
    .await
    .expect("seed");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("upgrade");

    let entry: (String, i64) =
        sqlx::query_as("SELECT row_id, hlc FROM change_log WHERE table_name = 'projects'")
            .fetch_one(&pool)
            .await
            .expect("back-filled entry");
    assert_eq!(entry, ("p1".to_string(), 77));
}
