use super::activity;
use super::handle::{Db, new_id};
use super::media::{ItemOrigin, NewEntry, insert};
use crate::domain::media_transfer::{
    ImportPreview, ImportRow, ImportSummary, ParsedFile, PreviewRow,
};
use crate::error::AppResult;
use sqlx::{FromRow, SqliteConnection};
use std::collections::HashSet;

#[derive(FromRow)]
struct KnownRow {
    #[sqlx(rename = "type")]
    media_type: String,
    title: String,
    year: Option<i64>,
    source: String,
    external_id: Option<String>,
}

/// What the library already holds, to spot rows that are already in it.
struct Known {
    titles: HashSet<(String, String, Option<i64>)>,
    provider_ids: HashSet<(String, String)>,
}

impl Known {
    async fn load(conn: &mut SqliteConnection) -> AppResult<Self> {
        let rows: Vec<KnownRow> = sqlx::query_as(
            "SELECT type, title, year, source, external_id FROM media_items WHERE deleted = 0",
        )
        .fetch_all(conn)
        .await?;
        let mut known = Self {
            titles: HashSet::new(),
            provider_ids: HashSet::new(),
        };
        for row in rows {
            known
                .titles
                .insert((row.media_type, row.title.to_lowercase(), row.year));
            if let Some(external_id) = row.external_id {
                known.provider_ids.insert((row.source, external_id));
            }
        }
        Ok(known)
    }

    fn contains(&self, row: &ImportRow) -> bool {
        self.titles.contains(&title_key(row))
            || id_key(row).is_some_and(|key| self.provider_ids.contains(&key))
    }

    fn remember(&mut self, row: &ImportRow) {
        self.titles.insert(title_key(row));
        if let Some(key) = id_key(row) {
            self.provider_ids.insert(key);
        }
    }
}

fn title_key(row: &ImportRow) -> (String, String, Option<i64>) {
    (
        row.media_type.as_str().to_string(),
        row.details.title.to_lowercase(),
        row.details.year,
    )
}

fn id_key(row: &ImportRow) -> Option<(String, String)> {
    let external_id = row.external_id.clone()?;
    Some((row.source.as_str().to_string(), external_id))
}

/// Marks which rows are already in the library (or repeat an earlier row); writes nothing.
pub async fn preview(db: &Db, parsed: ParsedFile) -> AppResult<ImportPreview> {
    let mut conn = db.pool.acquire().await?;
    let mut known = Known::load(&mut conn).await?;
    let mut rows = Vec::with_capacity(parsed.rows.len());
    for row in parsed.rows {
        let duplicate = known.contains(&row);
        known.remember(&row);
        rows.push(PreviewRow { row, duplicate });
    }
    let duplicate_count = rows.iter().filter(|row| row.duplicate).count();
    Ok(ImportPreview {
        new_count: rows.len() - duplicate_count,
        duplicate_count,
        rows,
        rejected: parsed.rejected,
    })
}

/// Adds every new row in one transaction, so a failure leaves the library as it was.
pub async fn import(db: &Db, parsed: ParsedFile) -> AppResult<ImportSummary> {
    let mut tx = db.pool.begin().await?;
    let mut known = Known::load(&mut tx).await?;
    let mut added = 0;
    let mut duplicates = 0;
    for row in &parsed.rows {
        if known.contains(row) {
            duplicates += 1;
            continue;
        }
        known.remember(row);
        let origin = ItemOrigin {
            media_type: row.media_type,
            source: row.source,
            external_id: row.external_id.as_deref(),
        };
        let entry = NewEntry {
            status: row.status,
            progress: row.progress,
            rating: row.rating,
            notes: &row.notes,
            tags: row.tags.clone(),
            started_at: row.started_at,
            finished_at: row.finished_at,
            platform: row.platform.as_deref(),
        };
        insert(&mut tx, db, &origin, &row.details, &entry).await?;
        added += 1;
    }
    if added > 0 {
        activity::record(
            &mut tx,
            db,
            "media.imported",
            "media_import",
            &new_id(),
            &format!("Imported {added} titles"),
        )
        .await?;
    }
    tx.commit().await?;
    Ok(ImportSummary {
        added,
        duplicates,
        rejected: parsed.rejected.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::media;
    use crate::domain::media::{ItemDetails, MediaSource, MediaStatus, MediaType};
    use crate::domain::media_transfer::RawRow;

    fn parsed(rows: &[(&str, &str, &str)]) -> ParsedFile {
        let mut parsed = ParsedFile::default();
        for (index, (media_type, title, status)) in rows.iter().enumerate() {
            parsed.push(RawRow {
                line: index + 2,
                media_type: media_type.to_string(),
                title: title.to_string(),
                status: status.to_string(),
                progress: "3".into(),
                total: "12".into(),
                rating: "8".into(),
                tags: "cozy".into(),
                ..RawRow::default()
            });
        }
        parsed
    }

    async fn count(db: &Db) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM media_entries WHERE deleted = 0")
            .fetch_one(&db.pool)
            .await
            .expect("count")
    }

    #[tokio::test]
    async fn import_adds_rows_with_all_their_fields() {
        let db = Db::in_memory().await;

        let summary = import(&db, parsed(&[("anime", "Frieren", "in_progress")]))
            .await
            .expect("import");

        assert_eq!(
            (summary.added, summary.duplicates, summary.rejected),
            (1, 0, 0)
        );
        let entry = media::list(&db).await.expect("list").remove(0);
        assert_eq!(entry.title, "Frieren");
        assert_eq!(entry.progress, 3);
        assert_eq!(entry.progress_total, Some(12));
        assert_eq!(entry.rating, Some(8));
        assert_eq!(entry.tags.0, ["cozy"]);
        assert_eq!(entry.status, MediaStatus::InProgress);
    }

    #[tokio::test]
    async fn titles_already_in_the_library_or_repeated_in_the_file_are_skipped() {
        let db = Db::in_memory().await;
        let details = ItemDetails {
            title: "frieren".into(),
            cover_url: None,
            year: None,
            genres: vec![],
            synopsis: String::new(),
            progress_total: None,
        };
        let origin = ItemOrigin {
            media_type: MediaType::Anime,
            source: MediaSource::Manual,
            external_id: None,
        };
        media::add(&db, origin, details, MediaStatus::Planned)
            .await
            .expect("seed");
        let file = || {
            parsed(&[
                ("anime", "Frieren", "planned"),
                ("manga", "Frieren", "planned"),
                ("manga", "FRIEREN", "planned"),
                ("anime", "Berserk", "planned"),
                ("anime", "", "planned"),
            ])
        };

        let shown = preview(&db, file()).await.expect("preview");
        assert_eq!((shown.new_count, shown.duplicate_count), (2, 2));
        assert_eq!(shown.rejected.len(), 1);
        assert_eq!(count(&db).await, 1, "preview must not write");

        let summary = import(&db, file()).await.expect("import");
        assert_eq!(
            (summary.added, summary.duplicates, summary.rejected),
            (2, 2, 1)
        );
        assert_eq!(count(&db).await, 3);

        let again = import(&db, file()).await.expect("again");
        assert_eq!(again.added, 0);
        assert_eq!(count(&db).await, 3);
    }

    #[tokio::test]
    async fn provider_ids_count_as_duplicates() {
        let db = Db::in_memory().await;
        let row = |title: &str| RawRow {
            line: 2,
            media_type: "game".into(),
            title: title.into(),
            status: "planned".into(),
            source: "rawg".into(),
            external_id: "3328".into(),
            ..RawRow::default()
        };
        let mut first = ParsedFile::default();
        first.push(row("Hollow Knight"));
        import(&db, first).await.expect("import");

        let mut second = ParsedFile::default();
        second.push(row("Hollow Knight (renamed)"));
        let summary = import(&db, second).await.expect("import");

        assert_eq!((summary.added, summary.duplicates), (0, 1));
    }

    #[tokio::test]
    async fn an_import_is_one_timeline_entry_and_a_failure_changes_nothing() {
        let db = Db::in_memory().await;
        import(
            &db,
            parsed(&[("anime", "A", "planned"), ("anime", "B", "planned")]),
        )
        .await
        .expect("import");
        let activity: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM activity WHERE kind = 'media.imported'")
                .fetch_one(&db.pool)
                .await
                .expect("count");
        assert_eq!(activity, 1);

        // without the activity table the import fails after its inserts, so it must roll back
        sqlx::query("DROP TABLE activity")
            .execute(&db.pool)
            .await
            .expect("drop");
        let result = import(&db, parsed(&[("anime", "C", "planned")])).await;
        assert!(result.is_err());
        assert_eq!(count(&db).await, 2);
    }
}
