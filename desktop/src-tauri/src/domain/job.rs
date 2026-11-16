use super::civil_date;
use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

pub const STALE_DAYS: i64 = 7;
const DAY_MS: i64 = 24 * 60 * 60 * 1000;
const MAX_TEXT: usize = 200;
const MAX_NOTES: usize = 100_000;

pub const PREP_HEADING: &str = "## Interview prep";
pub const PREP_TEMPLATE: &str = "## Interview prep\n\n\
- What the company does, and what they need from this role\n\
- Two projects of mine that fit\n\
- Questions to ask them\n\
- Salary range and what I would accept\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum JobStage {
    Saved,
    Applied,
    Screening,
    Interview,
    Offer,
    Accepted,
    Rejected,
    Withdrawn,
}

impl JobStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Saved => "saved",
            Self::Applied => "applied",
            Self::Screening => "screening",
            Self::Interview => "interview",
            Self::Offer => "offer",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }

    /// Still in play: these are the ones that get follow-ups and can go stale.
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Saved | Self::Applied | Self::Screening | Self::Interview | Self::Offer
        )
    }

    /// The employer answered: a call, an interview, an offer or a no.
    pub fn is_response(self) -> bool {
        matches!(
            self,
            Self::Screening | Self::Interview | Self::Offer | Self::Accepted | Self::Rejected
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum JobSource {
    Linkedin,
    CompanySite,
    JobBoard,
    Referral,
    Recruiter,
    Other,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct JobApplication {
    pub id: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub company: String,
    pub role: String,
    pub url: String,
    pub location: String,
    pub salary: String,
    pub source: JobSource,
    pub stage: JobStage,
    pub applied_on: Option<String>,
    pub contact: String,
    pub resume_version: String,
    pub next_step: String,
    pub next_step_at: Option<i64>,
    pub notes_md: String,
    pub stage_changed_at: i64,
}

impl JobApplication {
    /// When the application last changed in any way, in unix ms (updated_at is an hlc).
    pub fn changed_at(&self) -> i64 {
        self.updated_at >> 16
    }

    /// Active and untouched for a week.
    pub fn is_stale(&self, now: i64) -> bool {
        self.stage.is_active() && now - self.changed_at() >= STALE_DAYS * DAY_MS
    }

    /// The next step has a time that has come.
    pub fn is_due(&self, now: i64) -> bool {
        self.stage.is_active() && self.next_step_at.is_some_and(|at| at <= now)
    }
}

/// An application with the two flags the board shows.
#[derive(Debug, Clone, Serialize)]
pub struct JobView {
    #[serde(flatten)]
    pub job: JobApplication,
    pub stale: bool,
    pub due: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JobInput {
    pub company: String,
    pub role: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub salary: String,
    pub source: JobSource,
    pub applied_on: Option<String>,
    #[serde(default)]
    pub contact: String,
    #[serde(default)]
    pub resume_version: String,
    #[serde(default)]
    pub next_step: String,
    pub next_step_at: Option<i64>,
    #[serde(default)]
    pub notes_md: String,
}

fn short(text: &str, what: &'static str) -> AppResult<String> {
    let text = text.trim();
    if text.chars().count() > MAX_TEXT {
        return Err(AppError::Invalid(what));
    }
    Ok(text.to_string())
}

impl JobInput {
    /// Trimmed and checked. The address must be http(s), because the page opens it.
    pub fn clean(self) -> AppResult<Self> {
        let company = short(&self.company, "the company name is too long")?;
        let role = short(&self.role, "the role is too long")?;
        if company.is_empty() || role.is_empty() {
            return Err(AppError::Invalid("company and role are required"));
        }
        let url = short(&self.url, "the address is too long")?;
        let lower = url.to_lowercase();
        if !url.is_empty() && !lower.starts_with("https://") && !lower.starts_with("http://") {
            return Err(AppError::Invalid(
                "the posting address must start with http",
            ));
        }
        let applied_on = match self.applied_on.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(day) => Some(civil_date::clean(day)?),
        };
        if self.notes_md.len() > MAX_NOTES {
            return Err(AppError::Invalid("the notes are too long"));
        }
        Ok(Self {
            company,
            role,
            url,
            location: short(&self.location, "the location is too long")?,
            salary: short(&self.salary, "the salary is too long")?,
            applied_on,
            contact: short(&self.contact, "the contact is too long")?,
            resume_version: short(&self.resume_version, "the resume version is too long")?,
            next_step: short(&self.next_step, "the next step is too long")?,
            ..self
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> JobInput {
        JobInput {
            company: " Acme ".into(),
            role: "Engineer".into(),
            url: String::new(),
            location: String::new(),
            salary: String::new(),
            source: JobSource::Other,
            applied_on: None,
            contact: String::new(),
            resume_version: String::new(),
            next_step: String::new(),
            next_step_at: None,
            notes_md: String::new(),
        }
    }

    fn job(stage: JobStage, changed_ms: i64) -> JobApplication {
        JobApplication {
            id: "j".into(),
            created_at: 0,
            updated_at: changed_ms << 16,
            company: "A".into(),
            role: "R".into(),
            url: String::new(),
            location: String::new(),
            salary: String::new(),
            source: JobSource::Other,
            stage,
            applied_on: None,
            contact: String::new(),
            resume_version: String::new(),
            next_step: String::new(),
            next_step_at: None,
            notes_md: String::new(),
            stage_changed_at: 0,
        }
    }

    #[test]
    fn clean_trims_and_requires_company_and_role() {
        assert_eq!(input().clean().expect("clean").company, "Acme");
        assert!(
            JobInput {
                role: " ".into(),
                ..input()
            }
            .clean()
            .is_err()
        );
        assert!(
            JobInput {
                company: String::new(),
                ..input()
            }
            .clean()
            .is_err()
        );
    }

    #[test]
    fn only_http_addresses_are_accepted() {
        let with = |url: &str| {
            JobInput {
                url: url.into(),
                ..input()
            }
            .clean()
        };
        assert!(with("https://acme.com/jobs/1").is_ok());
        assert!(with("javascript:alert(1)").is_err());
        assert!(with("file:///etc/passwd").is_err());
    }

    #[test]
    fn the_applied_day_must_be_a_real_day() {
        let with = |day: &str| {
            JobInput {
                applied_on: Some(day.into()),
                ..input()
            }
            .clean()
        };
        assert_eq!(
            with("2026-11-16").expect("day").applied_on.as_deref(),
            Some("2026-11-16")
        );
        assert!(with("2026-13-40").is_err());
        assert_eq!(with("  ").expect("blank").applied_on, None);
    }

    #[test]
    fn active_applications_go_stale_after_a_week() {
        let now = 100 * DAY_MS;
        assert!(job(JobStage::Applied, now - 8 * DAY_MS).is_stale(now));
        assert!(!job(JobStage::Applied, now - 6 * DAY_MS).is_stale(now));
        assert!(!job(JobStage::Rejected, now - 30 * DAY_MS).is_stale(now));
    }

    #[test]
    fn a_next_step_is_due_once_its_time_has_come() {
        let mut item = job(JobStage::Interview, 0);
        item.next_step_at = Some(50);
        assert!(item.is_due(60));
        assert!(!item.is_due(40));
        item.stage = JobStage::Offer;
        item.next_step_at = None;
        assert!(!item.is_due(60));
    }

    #[test]
    fn responses_are_calls_interviews_offers_and_rejections() {
        assert!(JobStage::Rejected.is_response());
        assert!(JobStage::Screening.is_response());
        assert!(!JobStage::Applied.is_response());
        assert!(!JobStage::Withdrawn.is_response());
    }
}
