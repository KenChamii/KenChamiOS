use crate::error::AppResult;
use crate::workflow::safe_path::resolve_inside;
use std::fs;

const VAULT_GUIDE: &str = "# Vault guide\n\n\
Notes in `KenChamiOS/` are kept in step with the KenChamiOS app.\n\n\
- Write freely outside the `<!-- kco:start ... -->` and `<!-- kco:end ... -->` markers.\n\
- Text between the markers is shared with the app and changes both ways (daily notes, project idea and plan, task ticks).\n\
- `Inbox.md`: add one list item per line, the app turns each into an inbox task.\n\
- `Dashboard.md` and `Reviews/` are written by the app; edits there are overwritten.\n\
- Never change `kco_id` in the frontmatter.\n";

const MORNING: &str = "Read `KenChamiOS/Dashboard.md` and today's file in `KenChamiOS/Daily/`.\n\
Summarise what is planned, point out anything overdue, and suggest the first task to start.\n";

const REVIEW: &str = "Read the latest file in `KenChamiOS/Reviews/` and this week's daily notes.\n\
Write what went well, what slipped, and three things for next week.\n\
Put the answer in the note region of today's daily file.\n";

const CAPTURE: &str = "Take the text I give you and add it to `KenChamiOS/Inbox.md` as one list item per idea.\n\
Do not change any line that is already there.\n";

const FILES: [(&str, &str); 4] = [
    ("CLAUDE.md", VAULT_GUIDE),
    (".claude/commands/morning.md", MORNING),
    (".claude/commands/review.md", REVIEW),
    (".claude/commands/capture.md", CAPTURE),
];

/// Writes the guide files the vault lacks; a file the user already has is never touched.
pub fn write_missing(root: &str) -> AppResult<usize> {
    let mut written = 0;
    for (relative, text) in FILES {
        let path = resolve_inside(root, relative)?;
        if path.exists() {
            continue;
        }
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, text)?;
        written += 1;
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_the_guides_once_and_never_overwrites() {
        let dir = std::env::temp_dir().join(format!("kenchamios-guides-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("dir");
        let root = dir.to_str().expect("utf-8");

        assert_eq!(write_missing(root).expect("first"), 4);
        fs::write(dir.join("CLAUDE.md"), "mine").expect("edit");
        assert_eq!(write_missing(root).expect("second"), 0);

        assert_eq!(
            fs::read_to_string(dir.join("CLAUDE.md")).expect("read"),
            "mine"
        );
        assert!(dir.join(".claude/commands/morning.md").exists());
        let _ = fs::remove_dir_all(&dir);
    }
}
