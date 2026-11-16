use super::dates::{day_start_ms, parse_day};
use super::sync::{Direction, Kind, run};
use crate::db::projects::create_for_test;
use crate::db::tasks::{self, create_input};
use crate::db::{Db, daily_notes, docs, planner};
use crate::domain::planner::TaskPlan;
use crate::domain::{DocKind, TaskStatus};
use std::fs;
use std::path::PathBuf;

const TODAY: &str = "2026-11-16";

struct Env {
    db: Db,
    vault: PathBuf,
    data: PathBuf,
    now: i64,
}

impl Env {
    async fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("kenchamios-vault-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let (vault, data) = (root.join("vault"), root.join("data"));
        fs::create_dir_all(&vault).expect("vault");
        fs::create_dir_all(&data).expect("data");
        let day = parse_day(TODAY).expect("day");
        Self {
            db: Db::in_memory().await,
            vault,
            data,
            now: day_start_ms(day) + 12 * 3_600_000,
        }
    }

    fn root(&self) -> &str {
        self.vault.to_str().expect("utf-8")
    }

    async fn sync(&self) -> super::sync::Report {
        run(&self.db, &self.data, self.root(), self.now, true)
            .await
            .expect("sync")
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.vault.join(relative)).expect("read")
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.vault.join(relative);
        fs::create_dir_all(path.parent().expect("parent")).expect("dirs");
        fs::write(path, text).expect("write");
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        if let Some(root) = self.vault.parent() {
            let _ = fs::remove_dir_all(root);
        }
    }
}

const DAILY: &str = "KenChamiOS/Daily/2026-11-16.md";

#[tokio::test]
async fn export_writes_the_files_and_a_second_run_writes_nothing() {
    let env = Env::new("export").await;
    daily_notes::save(&env.db, TODAY, "my day")
        .await
        .expect("note");

    let first = env.sync().await;
    assert!(
        first
            .changes
            .iter()
            .any(|c| c.path == DAILY && c.kind == Kind::Added)
    );
    assert!(env.read(DAILY).contains("my day"));
    assert!(env.vault.join("KenChamiOS/Dashboard.md").exists());
    assert!(env.vault.join("KenChamiOS/Inbox.md").exists());

    let second = env.sync().await;
    assert!(second.changes.is_empty(), "{:?}", second.changes);
}

#[tokio::test]
async fn a_preview_writes_nothing() {
    let env = Env::new("preview").await;
    daily_notes::save(&env.db, TODAY, "x").await.expect("note");

    let report = run(&env.db, &env.data, env.root(), env.now, false)
        .await
        .expect("preview");

    assert!(!report.changes.is_empty());
    assert!(!env.vault.join("KenChamiOS").exists());
}

#[tokio::test]
async fn an_edit_in_the_vault_reaches_the_app() {
    let env = Env::new("vault-edit").await;
    daily_notes::save(&env.db, TODAY, "before")
        .await
        .expect("note");
    env.sync().await;

    env.write(
        DAILY,
        &env.read(DAILY).replace("before", "written in obsidian"),
    );
    let report = env.sync().await;

    let note = daily_notes::get(&env.db, TODAY)
        .await
        .expect("get")
        .expect("note");
    assert_eq!(note.body, "written in obsidian");
    assert!(
        report
            .changes
            .iter()
            .any(|c| c.direction == Direction::ToApp)
    );
    assert!(env.sync().await.changes.is_empty());
}

#[tokio::test]
async fn an_edit_in_the_app_reaches_the_vault_and_keeps_text_around_it() {
    let env = Env::new("app-edit").await;
    daily_notes::save(&env.db, TODAY, "before")
        .await
        .expect("note");
    env.sync().await;
    env.write(
        DAILY,
        &env.read(DAILY)
            .replace("# 2026-11-16", "# 2026-11-16\n\nmy own words"),
    );
    env.sync().await;

    daily_notes::save(&env.db, TODAY, "after")
        .await
        .expect("note");
    env.sync().await;

    let text = env.read(DAILY);
    assert!(text.contains("after") && !text.contains("before"));
    assert!(text.contains("my own words"));
}

#[tokio::test]
async fn when_both_sides_change_the_vault_wins_and_the_app_text_is_kept_aside() {
    let env = Env::new("conflict").await;
    daily_notes::save(&env.db, TODAY, "original text")
        .await
        .expect("note");
    env.sync().await;

    daily_notes::save(&env.db, TODAY, "app side")
        .await
        .expect("note");
    env.write(
        DAILY,
        &env.read(DAILY).replace("original text", "vault side"),
    );
    let report = env.sync().await;

    assert!(report.changes.iter().any(|c| c.kind == Kind::Conflict));
    let note = daily_notes::get(&env.db, TODAY)
        .await
        .expect("get")
        .expect("note");
    assert_eq!(note.body, "vault side");
    let kept: Vec<_> = fs::read_dir(env.data.join("vault-conflicts"))
        .expect("dir")
        .flatten()
        .collect();
    assert_eq!(kept.len(), 1);
    assert!(
        fs::read_to_string(kept[0].path())
            .expect("read")
            .contains("app side")
    );
}

#[tokio::test]
async fn a_note_written_by_hand_is_adopted() {
    let env = Env::new("adopt").await;
    env.write("KenChamiOS/Daily/2026-11-10.md", "just typed this\n");

    env.sync().await;

    let note = daily_notes::get(&env.db, "2026-11-10")
        .await
        .expect("get")
        .expect("note");
    assert_eq!(note.body, "just typed this");
    let text = env.read("KenChamiOS/Daily/2026-11-10.md");
    assert!(text.starts_with("---\nkco_id: daily:2026-11-10"));
    assert!(env.sync().await.changes.is_empty());
}

#[tokio::test]
async fn project_docs_go_both_ways_and_a_renamed_file_is_still_found() {
    let env = Env::new("project").await;
    let project = create_for_test(&env.db, "Demo App").await;
    docs::upsert(&env.db, &project.id, DocKind::Plan, "Plan", "step one")
        .await
        .expect("doc");
    env.sync().await;
    let original = "KenChamiOS/Projects/demo-app.md";
    assert!(env.read(original).contains("step one"));

    let moved = "KenChamiOS/Projects/renamed.md";
    env.write(moved, &env.read(original).replace("step one", "step two"));
    fs::remove_file(env.vault.join(original)).expect("remove");
    env.sync().await;

    let plan = docs::get(&env.db, &project.id, DocKind::Plan)
        .await
        .expect("get")
        .expect("doc");
    assert_eq!(plan.body_md, "step two");
    assert!(
        !env.vault.join(original).exists(),
        "the renamed file is the one kept"
    );
}

#[tokio::test]
async fn ticking_a_task_in_the_vault_completes_it_and_back() {
    let env = Env::new("ticks").await;
    let project = create_for_test(&env.db, "P").await;
    let task = tasks::create(&env.db, create_input(&project.id, "Write tests"))
        .await
        .expect("task");
    let plan = TaskPlan {
        planned_on: Some(TODAY.to_string()),
        scheduled_at: None,
        duration_min: None,
    };
    planner::set_plan(&env.db, &task.id, plan)
        .await
        .expect("plan");
    env.sync().await;
    assert!(env.read(DAILY).contains("- [ ] Write tests"));

    env.write(
        DAILY,
        &env.read(DAILY)
            .replace("- [ ] Write tests", "- [x] Write tests"),
    );
    env.sync().await;
    assert_eq!(
        tasks::get(&env.db, &task.id).await.expect("task").status,
        TaskStatus::Done
    );
    assert!(env.read(DAILY).contains("- [x] Write tests"));

    env.write(
        DAILY,
        &env.read(DAILY)
            .replace("- [x] Write tests", "- [ ] Write tests"),
    );
    env.sync().await;
    assert_eq!(
        tasks::get(&env.db, &task.id).await.expect("task").status,
        TaskStatus::Todo
    );
}

#[tokio::test]
async fn inbox_lines_become_tasks_once() {
    let env = Env::new("inbox").await;
    env.sync().await;
    let inbox = "KenChamiOS/Inbox.md";
    env.write(
        inbox,
        &format!(
            "{}- buy milk\n* call mum\n- [x] old thing\n",
            env.read(inbox)
        ),
    );

    let first = env.sync().await;
    let second = env.sync().await;

    assert_eq!(first.inbox_imported, 2);
    assert_eq!(second.inbox_imported, 0);
    let items = tasks::list(&env.db, crate::domain::project::INBOX_ID)
        .await
        .expect("inbox");
    let mut titles: Vec<_> = items.iter().map(|t| t.title.as_str()).collect();
    titles.sort_unstable();
    assert_eq!(titles, ["buy milk", "call mum"]);
    assert!(
        env.read(inbox)
            .contains("- [x] buy milk <!-- kco:imported -->")
    );
}

#[tokio::test]
async fn excluded_projects_stay_out_of_the_vault() {
    let env = Env::new("excluded").await;
    create_for_test(&env.db, "Secret App").await;
    create_for_test(&env.db, "Open App").await;
    crate::db::vault::set_excluded(&env.db, &["secret app".to_string()])
        .await
        .expect("set");

    env.sync().await;

    assert!(env.vault.join("KenChamiOS/Projects/open-app.md").exists());
    assert!(!env.vault.join("KenChamiOS/Projects/secret-app.md").exists());
    assert!(!env.read("KenChamiOS/Dashboard.md").contains("Secret App"));
}

#[tokio::test]
async fn a_missing_vault_is_refused() {
    let env = Env::new("missing").await;
    let gone = env.vault.join("nope");
    let result = run(
        &env.db,
        &env.data,
        gone.to_str().expect("utf-8"),
        env.now,
        true,
    )
    .await;
    assert!(result.is_err());
}
