use super::handle::Db;
use crate::domain::dashboard::{
    ActivityRow, DueTask, FocusRow, GenreCount, MediaTypeStat, RecapRange, RecapView,
    TimelineGroup, TimelineItem, bucket_totals, project_totals,
};
use crate::domain::media::MediaType;
use crate::error::AppResult;
use sqlx::SqlitePool;

const MAX_DUE_TASKS: i64 = 50;
const MAX_TIMELINE: i64 = 200;
const TOP_GENRES: i64 = 5;

/// Open tasks due before `until` (the end of today in the UI's time zone), overdue ones first.
pub async fn due_tasks(db: &Db, until: i64) -> AppResult<Vec<DueTask>> {
    let rows = sqlx::query_as(
        "SELECT t.id, t.title, t.project_id, p.name AS project_name, t.due_at, t.priority, t.status,
                t.recurrence
           FROM tasks t JOIN projects p ON p.id = t.project_id
          WHERE t.deleted = 0 AND t.status != 'done' AND t.due_at IS NOT NULL AND t.due_at < ?
            AND p.deleted = 0 AND p.archived = 0
          ORDER BY t.due_at, t.id
          LIMIT ?",
    )
    .bind(until)
    .bind(MAX_DUE_TASKS)
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

/// The newest activity first, optionally of one group only.
pub async fn timeline(
    db: &Db,
    group: Option<TimelineGroup>,
    limit: i64,
) -> AppResult<Vec<TimelineItem>> {
    let prefixes = group.map(|group| serde_json::to_string(group.prefixes()));
    let rows: Vec<ActivityRow> = sqlx::query_as(
        "SELECT id, kind, entity_type, entity_id, summary, at FROM activity
          WHERE deleted = 0
            AND (?1 IS NULL OR substr(kind, 1, instr(kind, '.') - 1) IN
                 (SELECT value FROM json_each(?1)))
          ORDER BY at DESC, rowid DESC
          LIMIT ?2",
    )
    .bind(prefixes.transpose()?)
    .bind(limit.clamp(1, MAX_TIMELINE))
    .fetch_all(&db.pool)
    .await?;
    Ok(rows.into_iter().map(TimelineItem::from).collect())
}

/// Focus time, media progress and project movement in `[from, to)`.
pub async fn recap(db: &Db, range: &RecapRange) -> AppResult<RecapView> {
    range.validate()?;
    let focus = focus_rows(&db.pool, range).await?;
    Ok(RecapView {
        focus_seconds: focus.iter().map(|row| row.duration_s).sum(),
        focus_by_bucket: bucket_totals(&range.buckets, &focus),
        focus_by_project: project_totals(&focus),
        media: media_stats(&db.pool, range).await?,
        top_genres: top_genres(&db.pool, range).await?,
        stage_changes: stage_changes(&db.pool, range).await?,
        projects_finished: projects_finished(&db.pool, range).await?,
    })
}

// breaks are not worked time
async fn focus_rows(pool: &SqlitePool, range: &RecapRange) -> AppResult<Vec<FocusRow>> {
    let rows = sqlx::query_as(
        "SELECT t.project_id, p.name AS project_name, p.color AS project_color,
                t.started_at, t.duration_s
           FROM time_entries t JOIN projects p ON p.id = t.project_id
          WHERE t.deleted = 0 AND t.kind != 'break' AND p.deleted = 0
            AND t.started_at >= ? AND t.started_at < ?",
    )
    .bind(range.from)
    .bind(range.to)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn media_stats(pool: &SqlitePool, range: &RecapRange) -> AppResult<Vec<MediaTypeStat>> {
    let units: Vec<(MediaType, i64)> = sqlx::query_as(
        "SELECT i.type, SUM(l.delta)
           FROM media_progress_log l
           JOIN media_entries e ON e.id = l.entry_id
           JOIN media_items i ON i.id = e.media_item_id
          WHERE l.deleted = 0 AND e.deleted = 0 AND l.logged_at >= ? AND l.logged_at < ?
          GROUP BY i.type",
    )
    .bind(range.from)
    .bind(range.to)
    .fetch_all(pool)
    .await?;
    let completed: Vec<(MediaType, i64)> = sqlx::query_as(
        "SELECT i.type, COUNT(*)
           FROM media_entries e JOIN media_items i ON i.id = e.media_item_id
          WHERE e.deleted = 0 AND e.status = 'completed' AND e.finished_at >= ? AND e.finished_at < ?
          GROUP BY i.type",
    )
    .bind(range.from)
    .bind(range.to)
    .fetch_all(pool)
    .await?;

    let mut stats: Vec<MediaTypeStat> = Vec::new();
    for (media_type, delta) in units {
        stat_of(&mut stats, media_type).units = delta.max(0);
    }
    for (media_type, count) in completed {
        stat_of(&mut stats, media_type).completed = count;
    }
    stats.sort_by_key(|stat| stat.media_type.as_str());
    Ok(stats)
}

fn stat_of(stats: &mut Vec<MediaTypeStat>, media_type: MediaType) -> &mut MediaTypeStat {
    let index = stats
        .iter()
        .position(|stat| stat.media_type == media_type)
        .unwrap_or_else(|| {
            stats.push(MediaTypeStat {
                media_type,
                units: 0,
                completed: 0,
            });
            stats.len() - 1
        });
    &mut stats[index]
}

async fn top_genres(pool: &SqlitePool, range: &RecapRange) -> AppResult<Vec<GenreCount>> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        "SELECT g.value, COUNT(*) AS n
           FROM media_entries e
           JOIN media_items i ON i.id = e.media_item_id, json_each(i.genres) g
          WHERE e.deleted = 0 AND e.status = 'completed'
            AND e.finished_at >= ? AND e.finished_at < ?
          GROUP BY g.value ORDER BY n DESC, g.value LIMIT ?",
    )
    .bind(range.from)
    .bind(range.to)
    .bind(TOP_GENRES)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(genre, count)| GenreCount { genre, count })
        .collect())
}

async fn stage_changes(pool: &SqlitePool, range: &RecapRange) -> AppResult<i64> {
    let count = sqlx::query_scalar(
        "SELECT COUNT(*) FROM activity
          WHERE kind = 'project.stage_changed' AND deleted = 0 AND at >= ? AND at < ?",
    )
    .bind(range.from)
    .bind(range.to)
    .fetch_one(pool)
    .await?;
    Ok(count)
}

async fn projects_finished(pool: &SqlitePool, range: &RecapRange) -> AppResult<Vec<String>> {
    let names = sqlx::query_scalar(
        "SELECT name FROM projects
          WHERE deleted = 0 AND finished_at >= ? AND finished_at < ?
          ORDER BY finished_at",
    )
    .bind(range.from)
    .bind(range.to)
    .fetch_all(pool)
    .await?;
    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::media::{self, ItemOrigin};
    use crate::db::projects::{self, create_for_test};
    use crate::db::{tasks, time_entries};
    use crate::domain::media::{ItemDetails, MediaSource, MediaStatus};
    use crate::domain::project::Stage;
    use crate::domain::task::{Priority, TaskCreate, TaskStatus};
    use crate::domain::time_entry::TimeEntryInput;

    async fn task(db: &Db, project_id: &str, title: &str, due_at: Option<i64>) -> String {
        tasks::create(
            db,
            TaskCreate {
                project_id: project_id.into(),
                parent_id: None,
                title: title.into(),
                body_md: String::new(),
                acceptance_md: String::new(),
                related_files: vec![],
                priority: Priority::Medium,
                due_at,
                tags: vec![],
                recurrence: None,
            },
        )
        .await
        .expect("task")
        .id
    }

    async fn focus(db: &Db, project_id: &str, started_at: i64, duration_s: i64) {
        let input = TimeEntryInput {
            task_id: None,
            started_at,
            duration_s,
            note: String::new(),
        };
        time_entries::create_manual(db, project_id, input)
            .await
            .expect("entry");
    }

    async fn add_title(
        db: &Db,
        title: &str,
        media_type: MediaType,
        total: Option<i64>,
        genres: &[&str],
    ) -> String {
        let details = ItemDetails {
            title: title.into(),
            cover_url: None,
            year: None,
            genres: genres.iter().map(|g| g.to_string()).collect(),
            synopsis: String::new(),
            progress_total: total,
        };
        let origin = ItemOrigin {
            media_type,
            source: MediaSource::Manual,
            external_id: None,
        };
        media::add(db, origin, details, MediaStatus::Planned)
            .await
            .expect("add")
            .id
    }

    fn range(from: i64, to: i64, buckets: Vec<i64>) -> RecapRange {
        RecapRange { from, to, buckets }
    }

    #[tokio::test]
    async fn due_tasks_include_overdue_and_stop_before_the_boundary() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        task(&db, &project.id, "overdue", Some(500)).await;
        task(&db, &project.id, "last moment", Some(999)).await;
        task(&db, &project.id, "tomorrow", Some(1000)).await;
        task(&db, &project.id, "no date", None).await;
        let done = task(&db, &project.id, "done", Some(100)).await;
        tasks::move_to(&db, &done, TaskStatus::Done, None, None)
            .await
            .expect("done");

        let due = due_tasks(&db, 1000).await.expect("due");

        let titles: Vec<_> = due.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, ["overdue", "last moment"]);
        assert_eq!(due[0].project_name, "Demo");
    }

    #[tokio::test]
    async fn due_tasks_skip_archived_projects() {
        let db = Db::in_memory().await;
        let kept = create_for_test(&db, "Kept").await;
        let archived = create_for_test(&db, "Archived").await;
        sqlx::query("UPDATE projects SET archived = 1 WHERE id = ?")
            .bind(&archived.id)
            .execute(&db.pool)
            .await
            .expect("archive");
        task(&db, &kept.id, "shown", Some(1)).await;
        task(&db, &archived.id, "hidden", Some(1)).await;

        let due = due_tasks(&db, 1000).await.expect("due");

        assert_eq!(due.len(), 1);
        assert_eq!(due[0].title, "shown");
    }

    #[tokio::test]
    async fn the_timeline_is_newest_first_and_filters_by_group() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Demo").await;
        focus(&db, &project.id, 1_000, 600).await;
        add_title(&db, "Frieren", MediaType::Anime, None, &[]).await;

        let all = timeline(&db, None, 50).await.expect("all");
        assert_eq!(all[0].kind, "media.added");
        assert!(all.iter().any(|item| item.kind == "project.created"));

        let focus_only = timeline(&db, Some(TimelineGroup::Focus), 50)
            .await
            .expect("focus");
        assert_eq!(focus_only.len(), 1);
        assert_eq!(focus_only[0].group, Some(TimelineGroup::Focus));
        let media_only = timeline(&db, Some(TimelineGroup::Media), 50)
            .await
            .expect("media");
        assert_eq!(media_only.len(), 1);
        let projects_only = timeline(&db, Some(TimelineGroup::Projects), 50)
            .await
            .expect("projects");
        assert!(!projects_only.is_empty());
        assert!(
            projects_only
                .iter()
                .all(|item| item.group == Some(TimelineGroup::Projects))
        );

        assert_eq!(timeline(&db, None, 1).await.expect("limited").len(), 1);
    }

    #[tokio::test]
    async fn focus_is_counted_inside_the_range_and_split_by_project_and_bucket() {
        let db = Db::in_memory().await;
        let first = create_for_test(&db, "First").await;
        let second = create_for_test(&db, "Second").await;
        focus(&db, &first.id, 99, 5).await;
        focus(&db, &first.id, 100, 60).await;
        focus(&db, &second.id, 250, 120).await;
        focus(&db, &first.id, 299, 30).await;
        focus(&db, &first.id, 300, 7).await;
        sqlx::query(
            "INSERT INTO time_entries
                (id, created_at, updated_at, device_id, project_id, kind, started_at, ended_at,
                 duration_s)
             VALUES ('b', 0, 0, 'd', ?, 'break', 150, 160, 10)",
        )
        .bind(&first.id)
        .execute(&db.pool)
        .await
        .expect("break");

        let view = recap(&db, &range(100, 300, vec![100, 200, 300]))
            .await
            .expect("recap");

        assert_eq!(view.focus_seconds, 210);
        assert_eq!(view.focus_by_bucket, [60, 150]);
        assert_eq!(view.focus_by_project.len(), 2);
        assert_eq!(view.focus_by_project[0].name, "Second");
        assert_eq!(view.focus_by_project[0].seconds, 120);
        assert_eq!(view.focus_by_project[1].seconds, 90);
    }

    #[tokio::test]
    async fn an_empty_period_gives_zeroes() {
        let db = Db::in_memory().await;

        let view = recap(&db, &range(0, 10, vec![0, 5, 10]))
            .await
            .expect("recap");

        assert_eq!(view.focus_seconds, 0);
        assert_eq!(view.focus_by_bucket, [0, 0]);
        assert!(view.media.is_empty());
        assert!(view.top_genres.is_empty());
        assert_eq!(view.stage_changes, 0);
    }

    #[tokio::test]
    async fn bad_ranges_are_refused() {
        let db = Db::in_memory().await;

        assert!(recap(&db, &range(10, 10, vec![])).await.is_err());
        assert!(recap(&db, &range(0, 10, vec![5, 1])).await.is_err());
    }

    #[tokio::test]
    async fn media_units_completed_titles_and_genres_follow_the_range() {
        let db = Db::in_memory().await;
        let show = add_title(
            &db,
            "Show",
            MediaType::Anime,
            Some(2),
            &["Drama", "Fantasy"],
        )
        .await;
        let manga = add_title(&db, "Manga", MediaType::Manga, None, &["Drama"]).await;
        media::set_progress(&db, &show, 1).await.expect("progress");
        media::set_progress(&db, &show, 2).await.expect("progress");
        media::set_progress(&db, &manga, 40)
            .await
            .expect("progress");
        sqlx::query("UPDATE media_progress_log SET logged_at = 150")
            .execute(&db.pool)
            .await
            .expect("time travel");
        sqlx::query("UPDATE media_progress_log SET logged_at = 400 WHERE value_after = 40")
            .execute(&db.pool)
            .await
            .expect("time travel");
        sqlx::query("UPDATE media_entries SET finished_at = 150 WHERE id = ?")
            .bind(&show)
            .execute(&db.pool)
            .await
            .expect("finish");

        let view = recap(&db, &range(100, 200, vec![100, 200]))
            .await
            .expect("recap");

        assert_eq!(view.media.len(), 1);
        assert_eq!(view.media[0].media_type, MediaType::Anime);
        assert_eq!((view.media[0].units, view.media[0].completed), (2, 1));
        let genres: Vec<_> = view.top_genres.iter().map(|g| g.genre.as_str()).collect();
        assert_eq!(genres, ["Drama", "Fantasy"]);

        let later = recap(&db, &range(200, 500, vec![200, 500]))
            .await
            .expect("later");
        assert_eq!(later.media[0].media_type, MediaType::Manga);
        assert_eq!((later.media[0].units, later.media[0].completed), (40, 0));
    }

    #[tokio::test]
    async fn stage_moves_and_finished_projects_are_counted() {
        let db = Db::in_memory().await;
        let project = create_for_test(&db, "Shipped").await;
        projects::set_stage(&db, &project.id, Stage::Build)
            .await
            .expect("stage");
        projects::set_stage(&db, &project.id, Stage::Done)
            .await
            .expect("stage");

        let view = recap(&db, &range(0, i64::MAX / 2, vec![]))
            .await
            .expect("recap");

        assert_eq!(view.stage_changes, 2);
        assert_eq!(view.projects_finished, ["Shipped"]);
    }
}
