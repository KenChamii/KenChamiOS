// two databases with their own clocks and device ids, edited and synced in every awkward order
use super::apply::apply_changes;
use super::export::read_page;
use super::purge::purge_tombstones;
use super::schema::SyncSchema;
use super::test_support::fixture;
use super::types::Change;
use crate::db::{Db, open_in_memory, paired_devices, projects, tasks};
use crate::domain::project::{ProjectInput, Stage};
use crate::sync::hlc::Hlc;
use serde_json::Value;

struct Peer {
    db: Db,
    // how far this peer has read the other one's log
    read_until: i64,
}

async fn peer(device_id: &str) -> Peer {
    let pool = open_in_memory().await.expect("open db");
    Peer {
        db: Db::new(pool, Hlc::new(), device_id.to_string()),
        read_until: 0,
    }
}

async fn pull_into(source: &Db, target: &mut Peer, schema: &SyncSchema) -> (usize, usize) {
    let (mut applied, mut ignored) = (0, 0);
    loop {
        let page = read_page(source, schema, target.read_until, 3, &target.db.device_id)
            .await
            .expect("read page");
        let outcome = apply_changes(&target.db, schema, &page.changes, None)
            .await
            .expect("apply");
        applied += outcome.applied;
        ignored += outcome.ignored;
        target.read_until = page.next;
        if !page.has_more {
            return (applied, ignored);
        }
    }
}

struct Pair {
    a: Peer,
    b: Peer,
    schema: SyncSchema,
}

impl Pair {
    async fn new() -> Self {
        Self {
            a: peer("dev-a").await,
            b: peer("dev-b").await,
            schema: SyncSchema::parse().expect("schema"),
        }
    }

    // a full round: b takes a's changes, then a takes b's
    async fn sync(&mut self) {
        pull_into(&self.a.db, &mut self.b, &self.schema).await;
        pull_into(&self.b.db, &mut self.a, &self.schema).await;
    }

    async fn snapshot(&self, db: &Db) -> Vec<Value> {
        let page = read_page(db, &self.schema, 0, 100_000, "nobody")
            .await
            .expect("snapshot");
        let mut rows: Vec<Value> = page
            .changes
            .into_iter()
            .map(|c| serde_json::json!({ "table": c.table, "row": c.row }))
            .collect();
        rows.sort_by_key(|r| r.to_string());
        rows
    }

    async fn assert_converged(&self) {
        assert_eq!(
            self.snapshot(&self.a.db).await,
            self.snapshot(&self.b.db).await
        );
    }
}

fn project_input(name: &str) -> ProjectInput {
    ProjectInput {
        name: name.to_string(),
        description: String::new(),
        repo_path: None,
        color: None,
        tags: vec![],
        archived: false,
    }
}

async fn project_name(db: &Db, id: &str) -> String {
    sqlx::query_scalar("SELECT name FROM projects WHERE id = ?")
        .bind(id)
        .fetch_one(&db.pool)
        .await
        .expect("project row")
}

async fn project_deleted(db: &Db, id: &str) -> i64 {
    sqlx::query_scalar("SELECT deleted FROM projects WHERE id = ?")
        .bind(id)
        .fetch_one(&db.pool)
        .await
        .expect("project row")
}

#[tokio::test]
async fn edits_in_both_directions_reach_the_other_side() {
    let mut pair = Pair::new().await;
    let project = projects::create_for_test(&pair.a.db, "Alpha").await;
    pair.sync().await;
    let task = tasks::create(
        &pair.b.db,
        tasks::create_input(&project.id, "From the phone"),
    )
    .await
    .expect("task");
    pair.sync().await;

    assert_eq!(project_name(&pair.b.db, &project.id).await, "Alpha");
    let title: String = sqlx::query_scalar("SELECT title FROM tasks WHERE id = ?")
        .bind(&task.id)
        .fetch_one(&pair.a.db.pool)
        .await
        .expect("task on a");
    assert_eq!(title, "From the phone");
    pair.assert_converged().await;
}

#[tokio::test]
async fn offline_edits_on_both_sides_merge() {
    let mut pair = Pair::new().await;
    let one = projects::create_for_test(&pair.a.db, "One").await;
    let two = projects::create_for_test(&pair.a.db, "Two").await;
    pair.sync().await;

    projects::update(&pair.a.db, &one.id, project_input("One (desktop)"))
        .await
        .expect("edit on a");
    projects::update(&pair.b.db, &two.id, project_input("Two (phone)"))
        .await
        .expect("edit on b");
    let three = projects::create_for_test(&pair.b.db, "Three").await;
    pair.sync().await;

    for db in [&pair.a.db, &pair.b.db] {
        assert_eq!(project_name(db, &one.id).await, "One (desktop)");
        assert_eq!(project_name(db, &two.id).await, "Two (phone)");
        assert_eq!(project_name(db, &three.id).await, "Three");
    }
    pair.assert_converged().await;
}

#[tokio::test]
async fn concurrent_edits_of_one_row_end_with_the_later_one_everywhere() {
    let mut pair = Pair::new().await;
    let project = projects::create_for_test(&pair.a.db, "Start").await;
    pair.sync().await;

    projects::update(&pair.a.db, &project.id, project_input("Earlier"))
        .await
        .expect("edit a");
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    projects::update(&pair.b.db, &project.id, project_input("Later"))
        .await
        .expect("edit b");
    pair.sync().await;
    pair.sync().await;

    assert_eq!(project_name(&pair.a.db, &project.id).await, "Later");
    assert_eq!(project_name(&pair.b.db, &project.id).await, "Later");
    pair.assert_converged().await;
}

#[tokio::test]
async fn a_later_delete_beats_an_edit_and_a_later_edit_beats_a_delete() {
    let mut pair = Pair::new().await;
    let first = projects::create_for_test(&pair.a.db, "First").await;
    let second = projects::create_for_test(&pair.a.db, "Second").await;
    pair.sync().await;

    projects::update(&pair.a.db, &first.id, project_input("Edited"))
        .await
        .expect("edit");
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    projects::delete(&pair.b.db, &first.id)
        .await
        .expect("delete");

    projects::delete(&pair.a.db, &second.id)
        .await
        .expect("delete");
    tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    projects::update(&pair.b.db, &second.id, project_input("Edited later"))
        .await
        .expect("edit");
    pair.sync().await;
    pair.sync().await;

    for db in [&pair.a.db, &pair.b.db] {
        assert_eq!(project_deleted(db, &first.id).await, 1);
        assert_eq!(project_deleted(db, &second.id).await, 0);
    }
    pair.assert_converged().await;
}

#[tokio::test]
async fn syncing_again_changes_nothing() {
    let mut pair = Pair::new().await;
    projects::create_for_test(&pair.a.db, "Alpha").await;
    pair.sync().await;
    let before = pair.snapshot(&pair.b.db).await;

    let page = read_page(&pair.a.db, &pair.schema, 0, 500, "dev-b")
        .await
        .expect("page");
    let replay = apply_changes(&pair.b.db, &pair.schema, &page.changes, None)
        .await
        .expect("replay");

    assert_eq!(replay.applied, 0);
    assert_eq!(pair.snapshot(&pair.b.db).await, before);
    pair.sync().await;
    assert_eq!(pair.snapshot(&pair.b.db).await, before);
}

#[tokio::test]
async fn pages_that_arrive_in_the_wrong_order_still_converge() {
    let pair = Pair::new().await;
    let project = projects::create_for_test(&pair.a.db, "Parent").await;
    tasks::create(&pair.a.db, tasks::create_input(&project.id, "Child"))
        .await
        .expect("task");

    let mut pages = Vec::new();
    let mut since = 0;
    loop {
        let page = read_page(&pair.a.db, &pair.schema, since, 2, "dev-b")
            .await
            .expect("page");
        since = page.next;
        let more = page.has_more;
        pages.push(page.changes);
        if !more {
            break;
        }
    }
    assert!(pages.len() > 1);
    for changes in pages.iter().rev() {
        apply_changes(&pair.b.db, &pair.schema, changes, None)
            .await
            .expect("apply");
    }

    pair.assert_converged().await;
}

#[tokio::test]
async fn a_batch_with_one_bad_change_changes_nothing() {
    let pair = Pair::new().await;
    let mut changes: Vec<Change> =
        serde_json::from_value(fixture("push-valid")["changes"].clone()).expect("changes");
    changes[3].row.insert(
        "updated_at".to_string(),
        Value::from(9_000_000_000_000_000_000_i64),
    );

    let result = apply_changes(&pair.b.db, &pair.schema, &changes, None).await;

    assert!(result.is_err());
    assert!(pair.snapshot(&pair.b.db).await.is_empty());
}

#[tokio::test]
async fn the_clock_moves_past_what_was_received() {
    let mut pair = Pair::new().await;
    let project = projects::create_for_test(&pair.a.db, "Alpha").await;
    pair.sync().await;
    let received: i64 = sqlx::query_scalar("SELECT updated_at FROM projects WHERE id = ?")
        .bind(&project.id)
        .fetch_one(&pair.b.db.pool)
        .await
        .expect("row");

    assert!(pair.b.db.stamp() > received);
}

#[tokio::test]
async fn a_peer_never_gets_its_own_rows_back() {
    let mut pair = Pair::new().await;
    projects::create_for_test(&pair.b.db, "From b").await;
    pull_into(&pair.b.db, &mut pair.a, &pair.schema).await;

    let echo = read_page(&pair.a.db, &pair.schema, 0, 500, "dev-b")
        .await
        .expect("page");

    assert!(echo.changes.is_empty());
    assert!(echo.next > 0, "the cursor still moves past skipped rows");
}

async fn shared_cases(name: &str) -> Vec<Value> {
    fixture(name).as_array().expect("cases").clone()
}

fn change_of(table: &str, row: &Value) -> Change {
    Change {
        table: table.to_string(),
        row: row.as_object().expect("row").clone(),
    }
}

#[tokio::test]
async fn shared_conflict_cases_resolve_as_specified() {
    let schema = SyncSchema::parse().expect("schema");
    for case in shared_cases("conflicts").await {
        let db = peer("dev-local").await.db;
        let table = case["table"].as_str().expect("table");
        apply_changes(&db, &schema, &[change_of(table, &case["stored"])], None)
            .await
            .expect("store");

        let outcome = apply_changes(&db, &schema, &[change_of(table, &case["incoming"])], None)
            .await
            .expect("incoming");

        let name = &case["name"];
        assert_eq!(
            outcome.applied == 1,
            case["applied"].as_bool().expect("applied"),
            "{name}"
        );
        let (title, deleted): (String, i64) =
            sqlx::query_as("SELECT title, deleted FROM tasks WHERE id = 't1'")
                .fetch_one(&db.pool)
                .await
                .expect("row");
        assert_eq!(title, case["title"].as_str().expect("title"), "{name}");
        if let Some(expected) = case["deleted"].as_i64() {
            assert_eq!(deleted, expected, "{name}");
        }
    }
}

#[tokio::test]
async fn shared_unique_key_cases_keep_one_live_row() {
    let schema = SyncSchema::parse().expect("schema");
    for case in shared_cases("unique-keys").await {
        let db = peer("dev-local").await.db;
        let table = case["table"].as_str().expect("table");
        apply_changes(&db, &schema, &[change_of(table, &case["stored"])], None)
            .await
            .expect("store");
        apply_changes(&db, &schema, &[change_of(table, &case["incoming"])], None)
            .await
            .expect("incoming");

        let table = schema.table(table).expect("synced table");
        let live: Vec<String> = sqlx::query_scalar(sqlx::AssertSqlSafe(format!(
            "SELECT id FROM {} WHERE deleted = 0 ORDER BY id",
            table.name
        )))
        .fetch_all(&db.pool)
        .await
        .expect("live rows");
        let expected: Vec<String> = case["live"]
            .as_array()
            .expect("live")
            .iter()
            .map(|v| v.as_str().expect("id").to_string())
            .collect();
        assert_eq!(live, expected, "{}", case["name"]);
    }
}

#[tokio::test]
async fn the_unique_key_result_is_the_same_on_both_sides() {
    let mut pair = Pair::new().await;
    let project = projects::create_for_test(&pair.a.db, "Alpha").await;
    pair.sync().await;
    for (db, body) in [(&pair.a.db, "from a"), (&pair.b.db, "from b")] {
        crate::db::docs::upsert(db, &project.id, crate::domain::DocKind::Idea, "Idea", body)
            .await
            .expect("doc");
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    pair.sync().await;
    pair.sync().await;

    pair.assert_converged().await;
    let live: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM project_docs WHERE deleted = 0")
        .fetch_one(&pair.a.db.pool)
        .await
        .expect("count");
    assert_eq!(live, 1);
}

async fn aged_tombstone(db: &Db, age_days: i64) -> String {
    let project = projects::create_for_test(db, "Gone").await;
    projects::delete(db, &project.id).await.expect("delete");
    let old = (crate::db::now_ms() - age_days * 24 * 60 * 60 * 1000) << 16;
    sqlx::query("UPDATE projects SET updated_at = ? WHERE id = ?")
        .bind(old)
        .bind(&project.id)
        .execute(&db.pool)
        .await
        .expect("age it");
    project.id
}

async fn project_count(db: &Db, id: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM projects WHERE id = ?")
        .bind(id)
        .fetch_one(&db.pool)
        .await
        .expect("count")
}

#[tokio::test]
async fn old_tombstones_go_only_after_every_device_has_them() {
    let db = peer("dev-a").await.db;
    let id = aged_tombstone(&db, 31).await;
    let now = crate::db::now_ms();

    assert_eq!(
        purge_tombstones(&db, now).await.expect("purge"),
        0,
        "no paired device"
    );
    paired_devices::add(&db, "phone", "Phone", "hash")
        .await
        .expect("pair");
    assert_eq!(
        purge_tombstones(&db, now).await.expect("purge"),
        0,
        "device behind"
    );
    assert_eq!(project_count(&db, &id).await, 1);

    let latest = super::export::latest_seq(&db).await.expect("latest");
    paired_devices::record_sync(&db, "phone", Some(latest))
        .await
        .expect("ack");
    assert!(purge_tombstones(&db, now).await.expect("purge") >= 1);
    assert_eq!(project_count(&db, &id).await, 0);
    let left: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM change_log WHERE row_id = ?")
        .bind(&id)
        .fetch_one(&db.pool)
        .await
        .expect("log");
    assert_eq!(left, 0);
}

#[tokio::test]
async fn young_tombstones_and_live_rows_stay() {
    let db = peer("dev-a").await.db;
    let young = aged_tombstone(&db, 29).await;
    let live = projects::create_for_test(&db, "Alive").await;
    paired_devices::add(&db, "phone", "Phone", "hash")
        .await
        .expect("pair");
    let latest = super::export::latest_seq(&db).await.expect("latest");
    paired_devices::record_sync(&db, "phone", Some(latest))
        .await
        .expect("ack");

    purge_tombstones(&db, crate::db::now_ms())
        .await
        .expect("purge");

    assert_eq!(project_count(&db, &young).await, 1);
    assert_eq!(project_count(&db, &live.id).await, 1);
    assert_eq!(
        projects::get(&db, &live.id)
            .await
            .expect("live")
            .project
            .stage,
        Stage::Idea
    );
}

#[tokio::test]
async fn a_tombstone_that_is_still_referenced_waits() {
    let db = peer("dev-a").await.db;
    let parent = aged_tombstone(&db, 31).await;
    sqlx::query(
        "INSERT INTO tasks (id, created_at, updated_at, device_id, project_id, title, order_key)
         VALUES ('t-child', 1, 1, 'd', ?, 'Child', 'a0')",
    )
    .bind(&parent)
    .execute(&db.pool)
    .await
    .expect("child");
    paired_devices::add(&db, "phone", "Phone", "hash")
        .await
        .expect("pair");
    let latest = super::export::latest_seq(&db).await.expect("latest");
    paired_devices::record_sync(&db, "phone", Some(latest))
        .await
        .expect("ack");

    purge_tombstones(&db, crate::db::now_ms())
        .await
        .expect("purge");

    assert_eq!(project_count(&db, &parent).await, 1);
}
