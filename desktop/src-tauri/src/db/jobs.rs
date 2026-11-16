use super::activity;
use super::handle::{Db, new_id, now_ms, require_changed};
use super::settings;
use crate::domain::job::{
    JobApplication, JobInput, JobStage, JobView, PREP_HEADING, PREP_TEMPLATE,
};
use crate::error::{AppError, AppResult};
use crate::vault::dates::{day_string, days_of, monday_of, parse_day};
use serde::Serialize;
use std::collections::BTreeMap;

const NOTIFIED_PREFIX: &str = "jobs.next_notified.";
const WEEKS_SHOWN: i64 = 12;
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

pub async fn get(db: &Db, id: &str) -> AppResult<JobApplication> {
    sqlx::query_as("SELECT * FROM job_applications WHERE id = ? AND deleted = 0")
        .bind(id)
        .fetch_optional(&db.pool)
        .await?
        .ok_or(AppError::NotFound("application"))
}

/// Live applications, newest change first, with the stale and due flags for `now`.
pub async fn list(db: &Db, now: i64) -> AppResult<Vec<JobView>> {
    let rows: Vec<JobApplication> = sqlx::query_as(
        "SELECT * FROM job_applications WHERE deleted = 0 ORDER BY updated_at DESC, id",
    )
    .fetch_all(&db.pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|job| JobView {
            stale: job.is_stale(now),
            due: job.is_due(now),
            job,
        })
        .collect())
}

pub async fn create(db: &Db, input: JobInput) -> AppResult<JobApplication> {
    let input = input.clean()?;
    let id = new_id();
    let now = now_ms();
    let stage = if input.applied_on.is_some() {
        JobStage::Applied
    } else {
        JobStage::Saved
    };
    let mut tx = db.pool.begin().await?;
    sqlx::query(
        "INSERT INTO job_applications
            (id, created_at, updated_at, device_id, company, role, url, location, salary, source,
             stage, applied_on, contact, resume_version, next_step, next_step_at, notes_md,
             stage_changed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(now)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(&input.company)
    .bind(&input.role)
    .bind(&input.url)
    .bind(&input.location)
    .bind(&input.salary)
    .bind(input.source)
    .bind(stage)
    .bind(&input.applied_on)
    .bind(&input.contact)
    .bind(&input.resume_version)
    .bind(&input.next_step)
    .bind(input.next_step_at)
    .bind(&input.notes_md)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    log_stage(&mut tx, &id, stage, now).await?;
    activity::record(
        &mut tx,
        db,
        "job.created",
        "job",
        &id,
        &format!("Saved {} at {}", input.role, input.company),
    )
    .await?;
    tx.commit().await?;
    get(db, &id).await
}

async fn log_stage(
    conn: &mut sqlx::SqliteConnection,
    job_id: &str,
    stage: JobStage,
    at: i64,
) -> AppResult<()> {
    sqlx::query("INSERT INTO job_stage_log (job_id, stage, entered_at) VALUES (?, ?, ?)")
        .bind(job_id)
        .bind(stage)
        .bind(at)
        .execute(conn)
        .await?;
    Ok(())
}

/// Replaces the fields; the stage is changed with `move_stage`.
pub async fn update(db: &Db, id: &str, input: JobInput) -> AppResult<JobApplication> {
    let input = input.clean()?;
    let result = sqlx::query(
        "UPDATE job_applications
            SET company = ?, role = ?, url = ?, location = ?, salary = ?, source = ?,
                applied_on = ?, contact = ?, resume_version = ?, next_step = ?, next_step_at = ?,
                notes_md = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(&input.company)
    .bind(&input.role)
    .bind(&input.url)
    .bind(&input.location)
    .bind(&input.salary)
    .bind(input.source)
    .bind(&input.applied_on)
    .bind(&input.contact)
    .bind(&input.resume_version)
    .bind(&input.next_step)
    .bind(input.next_step_at)
    .bind(&input.notes_md)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&db.pool)
    .await?;
    require_changed(result, "application")?;
    get(db, id).await
}

/// Moves the application to a stage. Reaching Applied sets the applied day when it is empty, and
/// reaching Interview adds the prep checklist to the notes once.
pub async fn move_stage(
    db: &Db,
    id: &str,
    stage: JobStage,
    today: &str,
) -> AppResult<JobApplication> {
    let job = get(db, id).await?;
    if job.stage == stage {
        return Ok(job);
    }
    let now = now_ms();
    let applied_on = match (stage, &job.applied_on) {
        (JobStage::Applied, None) => Some(today.to_string()),
        _ => job.applied_on.clone(),
    };
    let notes = if stage == JobStage::Interview && !job.notes_md.contains(PREP_HEADING) {
        let base = job.notes_md.trim_end();
        let separator = if base.is_empty() { "" } else { "\n\n" };
        format!("{base}{separator}{PREP_TEMPLATE}")
    } else {
        job.notes_md.clone()
    };
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE job_applications
            SET stage = ?, stage_changed_at = ?, applied_on = ?, notes_md = ?,
                updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(stage)
    .bind(now)
    .bind(applied_on)
    .bind(notes)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "application")?;
    log_stage(&mut tx, id, stage, now).await?;
    activity::record(
        &mut tx,
        db,
        "job.moved",
        "job",
        id,
        &format!("{} at {} is now {}", job.role, job.company, stage.as_str()),
    )
    .await?;
    tx.commit().await?;
    get(db, id).await
}

/// Replaces only the notes, for edits that come from the vault.
pub async fn set_notes(db: &Db, id: &str, notes: &str) -> AppResult<()> {
    let result = sqlx::query(
        "UPDATE job_applications SET notes_md = ?, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(notes)
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&db.pool)
    .await?;
    require_changed(result, "application")
}

/// Soft delete; it can be restored from the trash.
pub async fn delete(db: &Db, id: &str) -> AppResult<()> {
    let mut tx = db.pool.begin().await?;
    let result = sqlx::query(
        "UPDATE job_applications SET deleted = 1, updated_at = ?, device_id = ?
          WHERE id = ? AND deleted = 0",
    )
    .bind(db.stamp())
    .bind(&db.device_id)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    require_changed(result, "application")?;
    activity::record(
        &mut tx,
        db,
        "job.deleted",
        "job",
        id,
        "Deleted an application",
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// What the dashboard and the notifier need: the application and the step that is due.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, sqlx::FromRow)]
pub struct DueStep {
    pub id: String,
    pub company: String,
    pub role: String,
    pub next_step: String,
    pub next_step_at: i64,
}

/// Active applications whose next step time has come and was not announced for that time yet.
/// They are marked as announced, so a step is announced again only when its time changes.
pub async fn take_due_steps(db: &Db, now: i64) -> AppResult<Vec<DueStep>> {
    let due: Vec<DueStep> = sqlx::query_as(
        "SELECT id, company, role, next_step, next_step_at
           FROM job_applications j
          WHERE deleted = 0 AND stage IN ('saved', 'applied', 'screening', 'interview', 'offer')
            AND next_step_at IS NOT NULL AND next_step_at <= ?1
            AND NOT EXISTS (SELECT 1 FROM settings s
                             WHERE s.key = ?2 || j.id AND s.deleted = 0
                               AND s.value_json = json_quote(CAST(j.next_step_at AS TEXT)))
          ORDER BY next_step_at, id",
    )
    .bind(now)
    .bind(NOTIFIED_PREFIX)
    .fetch_all(&db.pool)
    .await?;
    for step in &due {
        let key = format!("{NOTIFIED_PREFIX}{}", step.id);
        settings::set_string(db, &key, &step.next_step_at.to_string()).await?;
    }
    Ok(due)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StageTime {
    pub stage: JobStage,
    pub avg_days: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JobStats {
    /// applications per week for the last twelve weeks, oldest first: (Monday, count)
    pub per_week: Vec<(String, i64)>,
    pub applied: i64,
    pub responded: i64,
    /// 0 to 1, or `None` before the first application
    pub response_rate: Option<f64>,
    pub time_per_stage: Vec<StageTime>,
}

pub async fn stats(db: &Db, now: i64) -> AppResult<JobStats> {
    let days: Vec<Option<String>> = sqlx::query_scalar(
        "SELECT applied_on FROM job_applications WHERE deleted = 0 AND applied_on IS NOT NULL",
    )
    .fetch_all(&db.pool)
    .await?;
    let this_monday = monday_of(days_of(now));
    let mut weeks: BTreeMap<i64, i64> = (0..WEEKS_SHOWN)
        .map(|back| (this_monday - back * 7, 0))
        .collect();
    for day in days.into_iter().flatten() {
        if let Some(parsed) = parse_day(&day)
            && let Some(count) = weeks.get_mut(&monday_of(parsed))
        {
            *count += 1;
        }
    }

    let log: Vec<(String, JobStage, i64)> = sqlx::query_as(
        "SELECT l.job_id, l.stage, l.entered_at
           FROM job_stage_log l JOIN job_applications j ON j.id = l.job_id
          WHERE j.deleted = 0 ORDER BY l.job_id, l.entered_at",
    )
    .fetch_all(&db.pool)
    .await?;
    let mut by_job: BTreeMap<&str, Vec<(JobStage, i64)>> = BTreeMap::new();
    for (job_id, stage, at) in &log {
        by_job.entry(job_id).or_default().push((*stage, *at));
    }
    let (mut applied, mut responded) = (0, 0);
    let mut spent: BTreeMap<&'static str, (JobStage, i64, i64)> = BTreeMap::new();
    for steps in by_job.values() {
        if steps.iter().any(|(stage, _)| *stage != JobStage::Saved) {
            applied += 1;
            if steps.iter().any(|(stage, _)| stage.is_response()) {
                responded += 1;
            }
        }
        for (index, (stage, entered)) in steps.iter().enumerate() {
            // the last stage of a job still in play runs until now
            let left = match steps.get(index + 1) {
                Some((_, next)) => *next,
                None if stage.is_active() => now,
                None => continue,
            };
            let entry = spent.entry(stage.as_str()).or_insert((*stage, 0, 0));
            entry.1 += (left - entered).max(0);
            entry.2 += 1;
        }
    }
    let mut time_per_stage: Vec<StageTime> = spent
        .into_values()
        .map(|(stage, total, count)| StageTime {
            stage,
            avg_days: total as f64 / count as f64 / DAY_MS as f64,
        })
        .collect();
    time_per_stage.sort_by_key(|item| stage_order(item.stage));
    Ok(JobStats {
        per_week: weeks
            .into_iter()
            .map(|(monday, n)| (day_string(monday), n))
            .collect(),
        applied,
        responded,
        response_rate: (applied > 0).then(|| responded as f64 / applied as f64),
        time_per_stage,
    })
}

fn stage_order(stage: JobStage) -> u8 {
    match stage {
        JobStage::Saved => 0,
        JobStage::Applied => 1,
        JobStage::Screening => 2,
        JobStage::Interview => 3,
        JobStage::Offer => 4,
        JobStage::Accepted => 5,
        JobStage::Rejected => 6,
        JobStage::Withdrawn => 7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::job::JobSource;

    fn input(company: &str) -> JobInput {
        JobInput {
            company: company.into(),
            role: "Engineer".into(),
            url: String::new(),
            location: String::new(),
            salary: String::new(),
            source: JobSource::Linkedin,
            applied_on: None,
            contact: String::new(),
            resume_version: String::new(),
            next_step: String::new(),
            next_step_at: None,
            notes_md: String::new(),
        }
    }

    #[tokio::test]
    async fn a_new_application_starts_saved_or_applied() {
        let db = Db::in_memory().await;
        let saved = create(&db, input("Acme")).await.expect("saved");
        assert_eq!(saved.stage, JobStage::Saved);
        let applied = create(
            &db,
            JobInput {
                applied_on: Some("2026-11-10".into()),
                ..input("Beta")
            },
        )
        .await
        .expect("applied");
        assert_eq!(applied.stage, JobStage::Applied);
    }

    #[tokio::test]
    async fn moving_to_applied_sets_the_day_and_interview_adds_the_prep_once() {
        let db = Db::in_memory().await;
        let job = create(&db, input("Acme")).await.expect("job");

        let applied = move_stage(&db, &job.id, JobStage::Applied, "2026-11-16")
            .await
            .expect("applied");
        assert_eq!(applied.applied_on.as_deref(), Some("2026-11-16"));

        let first = move_stage(&db, &job.id, JobStage::Interview, "2026-11-20")
            .await
            .expect("interview");
        assert!(first.notes_md.contains(PREP_HEADING));
        move_stage(&db, &job.id, JobStage::Offer, "2026-11-21")
            .await
            .expect("offer");
        let again = move_stage(&db, &job.id, JobStage::Interview, "2026-11-22")
            .await
            .expect("back");
        assert_eq!(again.notes_md.matches(PREP_HEADING).count(), 1);
    }

    #[tokio::test]
    async fn update_replaces_fields_and_delete_hides_the_row() {
        let db = Db::in_memory().await;
        let job = create(&db, input("Acme")).await.expect("job");
        let changed = update(
            &db,
            &job.id,
            JobInput {
                role: "Lead".into(),
                ..input("Acme")
            },
        )
        .await
        .expect("update");
        assert_eq!(changed.role, "Lead");

        delete(&db, &job.id).await.expect("delete");

        assert!(list(&db, now_ms()).await.expect("list").is_empty());
        assert!(get(&db, &job.id).await.is_err());
    }

    #[tokio::test]
    async fn due_steps_are_announced_once_per_time() {
        let db = Db::in_memory().await;
        let job = create(
            &db,
            JobInput {
                next_step: "Send thank you".into(),
                next_step_at: Some(1_000),
                ..input("Acme")
            },
        )
        .await
        .expect("job");

        let first = take_due_steps(&db, 2_000).await.expect("first");
        let second = take_due_steps(&db, 2_000).await.expect("second");
        assert_eq!(first.len(), 1);
        assert!(second.is_empty());

        update(
            &db,
            &job.id,
            JobInput {
                next_step_at: Some(1_500),
                ..input("Acme")
            },
        )
        .await
        .expect("update");
        assert_eq!(take_due_steps(&db, 2_000).await.expect("third").len(), 1);
    }

    #[tokio::test]
    async fn the_board_flags_stale_and_due_applications() {
        let db = Db::in_memory().await;
        create(
            &db,
            JobInput {
                next_step_at: Some(10),
                ..input("Acme")
            },
        )
        .await
        .expect("job");

        let now = now_ms();
        let today = list(&db, now).await.expect("list");
        assert!(today[0].due && !today[0].stale);
        let later = list(&db, now + 8 * DAY_MS).await.expect("list");
        assert!(later[0].stale);
    }

    #[tokio::test]
    async fn stats_count_weeks_responses_and_time_in_stage() {
        let db = Db::in_memory().await;
        let day = parse_day("2026-11-16").expect("day");
        let now = day * DAY_MS + DAY_MS / 2;
        let a = create(
            &db,
            JobInput {
                applied_on: Some("2026-11-16".into()),
                ..input("A")
            },
        )
        .await
        .expect("a");
        create(
            &db,
            JobInput {
                applied_on: Some("2026-11-10".into()),
                ..input("B")
            },
        )
        .await
        .expect("b");
        create(&db, input("C")).await.expect("c");
        move_stage(&db, &a.id, JobStage::Screening, "2026-11-17")
            .await
            .expect("move");

        let stats = stats(&db, now).await.expect("stats");

        assert_eq!(stats.per_week.len(), 12);
        assert_eq!(
            stats.per_week.last().expect("last"),
            &("2026-11-16".to_string(), 1)
        );
        assert_eq!(stats.per_week[10], ("2026-11-09".to_string(), 1));
        assert_eq!(stats.applied, 2);
        assert_eq!(stats.responded, 1);
        assert_eq!(stats.response_rate, Some(0.5));
        assert!(
            stats
                .time_per_stage
                .iter()
                .any(|t| t.stage == JobStage::Saved)
        );
    }
}
