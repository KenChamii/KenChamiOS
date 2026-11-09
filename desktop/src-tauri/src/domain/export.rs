use super::{Project, ProjectDoc, SavePoint};

/// Plan and save points (newest first) as one Markdown document.
pub fn project_markdown(
    project: &Project,
    plan: Option<&ProjectDoc>,
    save_points: &[SavePoint],
) -> String {
    let mut out = format!("# {}\n\n", project.name);
    if !project.description.is_empty() {
        out.push_str(&format!("{}\n\n", project.description));
    }
    out.push_str("## Plan\n\n");
    match plan.map(|doc| doc.body_md.trim()).filter(|b| !b.is_empty()) {
        Some(body) => out.push_str(&format!("{body}\n\n")),
        None => out.push_str("No plan yet.\n\n"),
    }
    out.push_str("## Save points\n\n");
    if save_points.is_empty() {
        out.push_str("No save points yet.\n");
    }
    for point in save_points {
        out.push_str(&save_point_markdown(point));
    }
    out
}

fn save_point_markdown(point: &SavePoint) -> String {
    let mut heading = format!(
        "{} - {}",
        format_date(point.created_at),
        point.stage.as_str()
    );
    if let Some(branch) = &point.git_branch {
        heading.push_str(&format!(" - {branch}"));
    }
    if let Some(commit) = &point.git_commit {
        heading.push_str(&format!(" @ {}", short_hash(commit)));
    }
    let mut out = format!("### {heading}\n\n{}\n\n", point.note);
    for (title, body) in [("Done", &point.done_md), ("Next", &point.next_md)] {
        if !body.trim().is_empty() {
            out.push_str(&format!("**{title}**\n\n{}\n\n", body.trim()));
        }
    }
    out
}

fn short_hash(hash: &str) -> &str {
    hash.get(..7).unwrap_or(hash)
}

/// UTC date as YYYY-MM-DD from unix ms.
fn format_date(ms: i64) -> String {
    // civil-from-days, shifted so the year starts in March
    let z = ms.div_euclid(86_400_000) + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Stage;
    use sqlx::types::Json;

    fn project() -> Project {
        Project {
            id: "p".into(),
            created_at: 0,
            name: "Demo".into(),
            slug: "demo".into(),
            stage: Stage::Build,
            description: "A demo".into(),
            repo_path: None,
            color: None,
            tags: Json(vec![]),
            archived: false,
            finished_at: None,
        }
    }

    fn save_point(created_at: i64, note: &str, next: &str) -> SavePoint {
        SavePoint {
            id: "s".into(),
            created_at,
            project_id: "p".into(),
            stage: Stage::Build,
            source: "manual".into(),
            note: note.into(),
            done_md: String::new(),
            next_md: next.into(),
            session_summary_md: String::new(),
            git_branch: Some("main".into()),
            git_commit: Some("0123456789abcdef".into()),
        }
    }

    #[test]
    fn formats_known_dates() {
        assert_eq!(format_date(0), "1970-01-01");
        assert_eq!(format_date(1_709_164_800_000), "2024-02-29");
        assert_eq!(format_date(1_794_182_400_000), "2026-11-09");
    }

    #[test]
    fn empty_project_has_placeholders() {
        let md = project_markdown(&project(), None, &[]);
        assert_eq!(
            md,
            "# Demo\n\nA demo\n\n## Plan\n\nNo plan yet.\n\n## Save points\n\nNo save points yet.\n"
        );
    }

    #[test]
    fn save_point_shows_commit_and_skips_empty_sections() {
        let points = [save_point(0, "Wrapped up", "Write tests")];
        let md = project_markdown(&project(), None, &points);
        assert!(md.contains("### 1970-01-01 - build - main @ 0123456\n\nWrapped up\n"));
        assert!(md.contains("**Next**\n\nWrite tests"));
        assert!(!md.contains("**Done**"));
    }
}
