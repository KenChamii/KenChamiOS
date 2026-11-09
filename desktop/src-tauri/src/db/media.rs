use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use crate::domain::media::{
    EntryInput, ItemDetails, LibraryEntry, MediaSource, MediaStatus, MediaType,
};
use crate::domain::tags;
use crate::error::{AppError, AppResult};
use sqlx::types::Json;

macro_rules! select_entry {
    ($tail:literal) => {
        concat!(
            "SELECT e.id, e.created_at, e.updated_at, e.media_item_id,
        i.type, i.title, i.cover_url, i.cover_cache_path, i.year, i.genres, i.synopsis,
        i.source, i.external_id, e.status, e.progress, e.progress_total, e.rating,
        e.notes, e.tags, e.started_at, e.finished_at, e.platform
     FROM media_entries e JOIN media_items i ON i.id = e.media_item_id ",
            $tail
        )
    };
}

pub async fn list(db: &Db) -> AppResult<Vec<LibraryEntry>> {
    let rows = sqlx::query_as(select_entry!(
        "WHERE e.deleted = 0 AND i.deleted = 0 ORDER BY e.updated_at DESC"
    ))
    .fetch_all(&db.pool)
    .await?;
    Ok(rows)
}

pub async fn get(db: &Db, id: &str) -> AppResult<LibraryEntry> {
    sqlx::query_as(select_entry!(
        "WHERE e.id = ? AND e.deleted = 0 AND i.deleted = 0"
    ))
    .bind(id)
    .fetch_optional(&db.pool)
    .await?
    .ok_or(AppError::NotFound("library entry"))
}

/// Where an item came from; `external_id` is the provider's id, `None` for manual items.
pub struct ItemOrigin<'a> {
    pub media_type: MediaType,
    pub source: MediaSource,
    pub external_id: Option<&'a str>,
}

/// Adds an item and its library entry in one step.
pub async fn add(
    db: &Db,
    origin: ItemOrigin<'_>,
    details: ItemDetails,
    status: MediaStatus,
) -> AppResult<LibraryEntry> {
    let title = details.title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::Invalid("a title is required"));
    }
    if details.progress_total.is_some_and(|t| t < 0) {
        return Err(AppError::Invalid("the total cannot be negative"));
    }
    let item_id = new_id();
    let entry_id = new_id();
    let now = now_ms();
    let mut tx = db.pool.begin().await?;
    if let Some(external_id) = origin.external_id {
        let taken: Option<String> = sqlx::query_scalar(
            "SELECT id FROM media_items WHERE source = ? AND external_id = ? AND deleted = 0",
        )
        .bind(origin.source)
        .bind(external_id)
        .fetch_optional(&mut *tx)
        .await?;
        if taken.is_some() {
            return Err(AppError::Invalid("this title is already in your library"));
        }
    }
    sqlx::query(
        "INSERT INTO media_items
            (id, created_at, updated_at, device_id, type, title, cover_url, year, genres,
             synopsis, source, external_id)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&item_id)
    .bind(now)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(origin.media_type)
    .bind(&title)
    .bind(details.cover_url)
    .bind(details.year)
    .bind(Json(tags::clean(details.genres)))
    .bind(details.synopsis.trim())
    .bind(origin.source)
    .bind(origin.external_id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "INSERT INTO media_entries
            (id, created_at, updated_at, device_id, media_item_id, status, progress_total,
             started_at, finished_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&entry_id)
    .bind(now)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(&item_id)
    .bind(status)
    .bind(details.progress_total)
    .bind((status == MediaStatus::InProgress).then_some(now))
    .bind((status == MediaStatus::Completed).then_some(now))
    .execute(&mut *tx)
    .await?;
    activity::record(
        &mut tx,
        db,
        "media.added",
        "media_entry",
        &entry_id,
        &format!("Added {title}"),
    )
    .await?;
    tx.commit().await?;
    get(db, &entry_id).await
}

/// Remembers the cached cover file of an item; a cache detail, so no activity.
pub async fn set_cover(db: &Db, item_id: &str, file_name: &str) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE media_items SET cover_cache_path = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(file_name)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(item_id)
    .execute(&db.pool)
    .await?;
    require_changed(result, "media item")
}

pub async fn update(db: &Db, id: &str, input: EntryInput) -> AppResult<LibraryEntry> {
    input.validate()?;
    let current = get(db, id).await?;
    if input.progress_total.is_some_and(|t| current.progress > t) {
        return Err(AppError::Invalid("the total is below the current progress"));
    }
    let now = now_ms();
    let moved = input.status != current.status;
    let started_at = input
        .started_at
        .or((moved && input.status == MediaStatus::InProgress).then_some(now));
    let finished_at = input
        .finished_at
        .or((moved && input.status == MediaStatus::Completed).then_some(now));
    let platform = input
        .platform
        .as_deref()
        .map(str::trim)
        .filter(|p| !p.is_empty());
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE media_entries
            SET status = ?, progress_total = ?, rating = ?, notes = ?, tags = ?,
                started_at = ?, finished_at = ?, platform = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(input.status)
    .bind(input.progress_total)
    .bind(input.rating)
    .bind(input.notes.trim())
    .bind(Json(tags::clean(input.tags)))
    .bind(started_at)
    .bind(finished_at)
    .bind(platform)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "library entry")?;
    activity::record(
        &mut tx,
        db,
        "media.updated",
        "media_entry",
        id,
        &format!("Updated {}", current.title),
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

/// Sets the progress counter, logs the change and moves the status along with it.
pub async fn set_progress(db: &Db, id: &str, value: i64) -> AppResult<LibraryEntry> {
    let current = get(db, id).await?;
    if value < 0 {
        return Err(AppError::Invalid("progress cannot be negative"));
    }
    if current.progress_total.is_some_and(|t| value > t) {
        return Err(AppError::Invalid("progress cannot go past the total"));
    }
    if value == current.progress {
        return Ok(current);
    }
    let now = now_ms();
    let (status, started_at, finished_at) = if current.progress_total == Some(value) {
        (
            MediaStatus::Completed,
            current.started_at,
            current.finished_at.or(Some(now)),
        )
    } else if current.status == MediaStatus::Planned && value > current.progress {
        (
            MediaStatus::InProgress,
            current.started_at.or(Some(now)),
            current.finished_at,
        )
    } else {
        (current.status, current.started_at, current.finished_at)
    };
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE media_entries
            SET progress = ?, status = ?, started_at = ?, finished_at = ?,
                updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(value)
    .bind(status)
    .bind(started_at)
    .bind(finished_at)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "library entry")?;
    sqlx::query(
        "INSERT INTO media_progress_log
            (id, created_at, updated_at, device_id, entry_id, delta, value_after, logged_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new_id())
    .bind(now)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .bind(value - current.progress)
    .bind(value)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    activity::record(
        &mut tx,
        db,
        "media.progress",
        "media_entry",
        id,
        &format!(
            "{}: {} {value}",
            current.title,
            progress_unit(current.media_type)
        ),
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

fn progress_unit(media_type: MediaType) -> &'static str {
    match media_type {
        MediaType::Anime => "episode",
        MediaType::Manga => "chapter",
        MediaType::Game => "hours",
    }
}

/// Soft deletes the entry and the item it tracks.
pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let current = get(db, id).await?;
    let mut tx = db.pool.begin().await?;
    for sql in [
        "UPDATE media_entries SET deleted = 1, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
        "UPDATE media_items SET deleted = 1, updated_at = ?, device_id = ?
          WHERE id = (SELECT media_item_id FROM media_entries WHERE id = ?) AND deleted = 0",
    ] {
        let result = sqlx::query(sql)
            .bind(db.stamp())
            .bind(&db.device_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
        require_changed(result, "library entry")?;
    }
    activity::record(
        &mut tx,
        db,
        "media.deleted",
        "media_entry",
        id,
        &format!("Removed {}", current.title),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::activity::kinds_for;

    fn details(title: &str, total: Option<i64>) -> ItemDetails {
        ItemDetails {
            title: title.to_string(),
            cover_url: None,
            year: Some(2020),
            genres: vec!["Action".into(), " ".into(), "Action".into()],
            synopsis: String::new(),
            progress_total: total,
        }
    }

    fn manual() -> ItemOrigin<'static> {
        ItemOrigin {
            media_type: MediaType::Anime,
            source: MediaSource::Manual,
            external_id: None,
        }
    }

    async fn add_manual(db: &Db, title: &str, total: Option<i64>) -> LibraryEntry {
        add(db, manual(), details(title, total), MediaStatus::Planned)
            .await
            .expect("add")
    }

    fn input(status: MediaStatus) -> EntryInput {
        EntryInput {
            status,
            progress_total: None,
            rating: Some(8),
            notes: " good ".into(),
            tags: vec!["fav".into(), "fav".into()],
            started_at: None,
            finished_at: None,
            platform: Some("  ".into()),
        }
    }

    #[tokio::test]
    async fn adding_creates_item_and_entry_with_cleaned_fields() {
        let db = Db::in_memory().await;
        let entry = add_manual(&db, "  Frieren ", Some(28)).await;

        assert_eq!(entry.title, "Frieren");
        assert_eq!(entry.genres.0, ["Action"]);
        assert_eq!(entry.status, MediaStatus::Planned);
        assert_eq!(entry.progress, 0);
        assert_eq!(entry.progress_total, Some(28));
        assert_eq!(list(&db).await.expect("list").len(), 1);
        assert_eq!(kinds_for(&db, &entry.id).await, ["media.added"]);
    }

    #[tokio::test]
    async fn blank_titles_are_rejected() {
        let db = Db::in_memory().await;
        let result = add(&db, manual(), details("  ", None), MediaStatus::Planned).await;
        assert!(matches!(result, Err(AppError::Invalid(_))));
    }

    #[tokio::test]
    async fn the_same_external_title_cannot_be_added_twice() {
        let db = Db::in_memory().await;
        let origin = || ItemOrigin {
            media_type: MediaType::Manga,
            source: MediaSource::Anilist,
            external_id: Some("30013"),
        };
        let first = add(
            &db,
            origin(),
            details("One Piece", None),
            MediaStatus::Planned,
        )
        .await
        .expect("first");

        let second = add(
            &db,
            origin(),
            details("One Piece", None),
            MediaStatus::Planned,
        )
        .await;
        assert!(matches!(second, Err(AppError::Invalid(_))));

        delete(&db, &first.id).await.expect("delete");
        add(
            &db,
            origin(),
            details("One Piece", None),
            MediaStatus::Planned,
        )
        .await
        .expect("re-add after delete");
    }

    #[tokio::test]
    async fn update_cleans_fields_and_fills_dates_on_a_status_change() {
        let db = Db::in_memory().await;
        let entry = add_manual(&db, "Frieren", None).await;

        let updated = update(&db, &entry.id, input(MediaStatus::Completed))
            .await
            .expect("update");

        assert_eq!(updated.status, MediaStatus::Completed);
        assert_eq!(updated.notes, "good");
        assert_eq!(updated.tags.0, ["fav"]);
        assert_eq!(updated.platform, None);
        assert!(updated.finished_at.is_some());
        assert!(updated.started_at.is_none());
        assert_eq!(
            kinds_for(&db, &entry.id).await,
            ["media.added", "media.updated"]
        );
    }

    #[tokio::test]
    async fn update_rejects_bad_ratings_and_totals_below_progress() {
        let db = Db::in_memory().await;
        let entry = add_manual(&db, "Frieren", Some(28)).await;
        set_progress(&db, &entry.id, 10).await.expect("progress");

        let bad_rating = EntryInput {
            rating: Some(11),
            ..input(MediaStatus::InProgress)
        };
        assert!(update(&db, &entry.id, bad_rating).await.is_err());
        let low_total = EntryInput {
            progress_total: Some(5),
            ..input(MediaStatus::InProgress)
        };
        assert!(update(&db, &entry.id, low_total).await.is_err());
    }

    #[tokio::test]
    async fn progress_changes_are_logged_with_activity_and_start_the_entry() {
        let db = Db::in_memory().await;
        let entry = add_manual(&db, "Frieren", Some(3)).await;

        let one = set_progress(&db, &entry.id, 1).await.expect("one");
        assert_eq!(one.status, MediaStatus::InProgress);
        assert!(one.started_at.is_some());
        set_progress(&db, &entry.id, 3).await.expect("finish");
        let back = set_progress(&db, &entry.id, 2).await.expect("back");

        assert_eq!(back.status, MediaStatus::Completed);
        assert!(back.finished_at.is_some());
        let log: Vec<(i64, i64)> = sqlx::query_as(
            "SELECT delta, value_after FROM media_progress_log WHERE entry_id = ?
              ORDER BY logged_at, rowid",
        )
        .bind(&entry.id)
        .fetch_all(&db.pool)
        .await
        .expect("log");
        assert_eq!(log, [(1, 1), (2, 3), (-1, 2)]);
        assert_eq!(
            kinds_for(&db, &entry.id).await,
            [
                "media.added",
                "media.progress",
                "media.progress",
                "media.progress"
            ]
        );
    }

    #[tokio::test]
    async fn progress_rejects_out_of_range_values_and_ignores_no_ops() {
        let db = Db::in_memory().await;
        let entry = add_manual(&db, "Frieren", Some(3)).await;

        assert!(set_progress(&db, &entry.id, -1).await.is_err());
        assert!(set_progress(&db, &entry.id, 4).await.is_err());
        set_progress(&db, &entry.id, 0).await.expect("same value");
        assert_eq!(kinds_for(&db, &entry.id).await, ["media.added"]);
    }

    #[tokio::test]
    async fn delete_hides_the_entry_and_its_item() {
        let db = Db::in_memory().await;
        let entry = add_manual(&db, "Frieren", None).await;

        delete(&db, &entry.id).await.expect("delete");

        assert!(list(&db).await.expect("list").is_empty());
        assert!(matches!(
            delete(&db, &entry.id).await,
            Err(AppError::NotFound(_))
        ));
        assert_eq!(
            kinds_for(&db, &entry.id).await,
            ["media.added", "media.deleted"]
        );
    }

    #[tokio::test]
    async fn cover_file_name_is_stored_on_the_item() {
        let db = Db::in_memory().await;
        let entry = add_manual(&db, "Frieren", None).await;

        set_cover(&db, &entry.media_item_id, "a.png")
            .await
            .expect("cover");

        let again = get(&db, &entry.id).await.expect("get");
        assert_eq!(again.cover_cache_path.as_deref(), Some("a.png"));
    }
}
