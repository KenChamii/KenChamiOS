use crate::domain::slugify;
use crate::error::{AppError, AppResult};
use git2::{BranchType, Repository, WorktreeAddOptions};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_SLUG: usize = 40;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Worktree {
    pub branch: String,
    pub path: PathBuf,
    /// `None` when the repo had a detached HEAD
    pub base_branch: Option<String>,
    pub base_commit: String,
}

/// A new branch `agent/<task-slug>` from the current HEAD, checked out in `dir/<run_id>`.
/// The user's own checkout is never touched.
pub fn create(repo_path: &str, dir: &Path, run_id: &str, task_title: &str) -> AppResult<Worktree> {
    let repo = Repository::open(repo_path.trim())
        .map_err(|_| AppError::Agent("the project's repo path is not a Git repo".into()))?;
    let head = repo
        .head()
        .map_err(|_| AppError::Agent("the repo has no commits yet".into()))?;
    let base_branch = head
        .is_branch()
        .then(|| String::from_utf8_lossy(head.shorthand_bytes()).into_owned());
    let commit = head.peel_to_commit()?;

    let branch_name = free_branch_name(&repo, task_title)?;
    let branch = repo.branch(&branch_name, &commit, false)?;
    fs::create_dir_all(dir)?;
    let path = dir.join(run_id);
    let mut options = WorktreeAddOptions::new();
    options.reference(Some(branch.get()));
    repo.worktree(run_id, &path, Some(&options))?;

    Ok(Worktree {
        branch: branch_name,
        path,
        base_branch,
        base_commit: commit.id().to_string(),
    })
}

fn free_branch_name(repo: &Repository, task_title: &str) -> AppResult<String> {
    let mut slug = if task_title.chars().any(|c| c.is_ascii_alphanumeric()) {
        slugify(task_title)
    } else {
        "task".to_string()
    };
    slug.truncate(MAX_SLUG);
    let slug = slug.trim_end_matches('-');
    let mut name = format!("agent/{slug}");
    let mut n = 2;
    while repo.find_branch(&name, BranchType::Local).is_ok() {
        name = format!("agent/{slug}-{n}");
        n += 1;
    }
    Ok(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Signature;

    struct TempRepo(PathBuf);

    impl TempRepo {
        fn new(name: &str, with_commit: bool) -> Self {
            let dir = std::env::temp_dir()
                .join(format!("kenchamios-worktree-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("create temp repo");
            let repo = Repository::init(&dir).expect("init");
            if with_commit {
                fs::write(dir.join("a.txt"), "one").expect("write");
                let mut index = repo.index().expect("index");
                index.add_path(Path::new("a.txt")).expect("add");
                let tree = repo
                    .find_tree(index.write_tree().expect("tree"))
                    .expect("find");
                let who = Signature::now("Test", "test@example.com").expect("signature");
                repo.commit(Some("HEAD"), &who, &who, "first", &tree, &[])
                    .expect("commit");
            }
            Self(dir)
        }

        fn path(&self) -> &str {
            self.0.to_str().expect("utf-8 temp path")
        }
    }

    impl Drop for TempRepo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn worktrees_dir(repo: &TempRepo) -> PathBuf {
        repo.0.join("..").join(format!(
            "kenchamios-worktrees-{}",
            repo.0.file_name().expect("name").to_string_lossy()
        ))
    }

    #[test]
    fn makes_a_branch_and_a_checkout_outside_the_repo() {
        let repo = TempRepo::new("make", true);
        let dir = worktrees_dir(&repo);
        let made = create(repo.path(), &dir, "run-1", "Add the Table!").expect("create");

        assert_eq!(made.branch, "agent/add-the-table");
        assert_eq!(made.path, dir.join("run-1"));
        assert_eq!(
            fs::read_to_string(made.path.join("a.txt")).expect("read"),
            "one"
        );
        assert!(made.base_branch.is_some());
        assert_eq!(made.base_commit.len(), 40);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_taken_branch_name_gets_a_number() {
        let repo = TempRepo::new("taken", true);
        let dir = worktrees_dir(&repo);
        create(repo.path(), &dir, "run-1", "Same task").expect("first");
        let second = create(repo.path(), &dir, "run-2", "Same task").expect("second");
        assert_eq!(second.branch, "agent/same-task-2");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_title_without_letters_still_gets_a_name() {
        let repo = TempRepo::new("blank", true);
        let dir = worktrees_dir(&repo);
        let made = create(repo.path(), &dir, "run-1", "!!!").expect("create");
        assert_eq!(made.branch, "agent/task");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_repo_without_commits_or_a_plain_folder_is_refused() {
        let empty = TempRepo::new("empty", false);
        let dir = worktrees_dir(&empty);
        let err = create(empty.path(), &dir, "run-1", "t").expect_err("no commits");
        assert!(err.to_string().contains("no commits"));

        let plain = std::env::temp_dir().join(format!("kenchamios-plain-{}", std::process::id()));
        fs::create_dir_all(&plain).expect("plain");
        let err = create(plain.to_str().expect("utf-8"), &dir, "run-1", "t").expect_err("not git");
        assert!(err.to_string().contains("not a Git repo"));
        let _ = fs::remove_dir_all(&plain);
    }
}
