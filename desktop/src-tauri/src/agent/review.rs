use super::secrets::{self, SecretHit};
use crate::error::{AppError, AppResult};
use git2::{
    BranchType, Delta, Diff, DiffFormat, DiffOptions, IndexAddOption, MergeAnalysis, Oid,
    Repository, StatusOptions, WorktreePruneOptions, build::CheckoutBuilder,
};
use serde::Serialize;
use std::fs;
use std::path::Path;

// a runaway agent could write a huge file; the review shows the start of the patch
const MAX_PATCH_BYTES: usize = 400_000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileChange {
    pub path: String,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunDiff {
    pub files: Vec<FileChange>,
    pub patch: String,
    pub truncated: bool,
    pub secret: Option<SecretHit>,
}

/// Everything the agent changed in the worktree since `base_commit`, untracked files included.
pub fn diff(worktree: &Path, base_commit: &str) -> AppResult<RunDiff> {
    let repo = Repository::open(worktree)?;
    let base = repo.find_commit(Oid::from_str(base_commit)?)?.tree()?;
    let mut options = DiffOptions::new();
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true);
    let diff = repo.diff_tree_to_workdir_with_index(Some(&base), Some(&mut options))?;

    let files = diff
        .deltas()
        .map(|delta| FileChange {
            path: delta
                .new_file()
                .path()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default(),
            status: status_name(delta.status()),
        })
        .collect();
    let (patch, truncated) = patch_text(&diff)?;
    Ok(RunDiff {
        files,
        patch,
        truncated,
        secret: find_secret(&diff)?,
    })
}

fn status_name(status: Delta) -> &'static str {
    match status {
        Delta::Added | Delta::Untracked => "added",
        Delta::Deleted => "deleted",
        Delta::Renamed => "renamed",
        _ => "modified",
    }
}

fn patch_text(diff: &Diff) -> AppResult<(String, bool)> {
    let mut bytes = Vec::new();
    let mut truncated = false;
    diff.print(DiffFormat::Patch, |_, _, line| {
        if bytes.len() >= MAX_PATCH_BYTES {
            truncated = true;
            return true;
        }
        if matches!(line.origin(), '+' | '-' | ' ') {
            bytes.push(line.origin() as u8);
        }
        bytes.extend_from_slice(line.content());
        true
    })?;
    Ok((String::from_utf8_lossy(&bytes).into_owned(), truncated))
}

fn find_secret(diff: &Diff) -> AppResult<Option<SecretHit>> {
    let mut hit = None;
    diff.print(DiffFormat::Patch, |delta, _, line| {
        if hit.is_none() && line.origin() == '+' {
            let text = String::from_utf8_lossy(line.content());
            if let Some(rule) = secrets::check_line(&text) {
                let path = delta.new_file().path().unwrap_or(Path::new(""));
                hit = Some(SecretHit {
                    rule,
                    path: path.to_string_lossy().into_owned(),
                });
            }
        }
        true
    })?;
    Ok(hit)
}

/// Commits everything in the worktree as the user from Git config; refused when a secret is in
/// the diff, when nothing changed or when there is no Git identity.
pub fn approve(worktree: &Path, base_commit: &str, message: &str) -> AppResult<String> {
    let message = message.trim();
    if message.is_empty() {
        return Err(AppError::Invalid("the commit message is required"));
    }
    if let Some(hit) = diff(worktree, base_commit)?.secret {
        return Err(AppError::Agent(format!(
            "the changes contain a {} in {}; remove it before approving",
            hit.rule, hit.path
        )));
    }
    let repo = Repository::open(worktree)?;
    let who = repo
        .signature()
        .map_err(|_| AppError::Agent("set user.name and user.email in Git config first".into()))?;
    let mut index = repo.index()?;
    index.add_all(["*"], IndexAddOption::DEFAULT, None)?;
    index.update_all(["*"], None)?;
    index.write()?;
    let tree = repo.find_tree(index.write_tree()?)?;
    let parent = repo.head()?.peel_to_commit()?;
    if tree.id() == parent.tree_id() {
        return Err(AppError::Agent("the agent made no changes".into()));
    }
    let id = repo.commit(Some("HEAD"), &who, &who, message, &tree, &[&parent])?;
    Ok(id.to_string())
}

/// Merges the run's branch into the branch checked out in the user's repo. Fast-forward when
/// possible, otherwise a merge commit. A conflict is found before anything is written.
pub fn merge(repo_path: &str, branch: &str) -> AppResult<String> {
    let repo = Repository::open(repo_path.trim())
        .map_err(|_| AppError::Agent("the project's repo path is not a Git repo".into()))?;
    let head = repo.head()?;
    if !head.is_branch() {
        return Err(AppError::Agent(
            "the repo is on a detached HEAD; check out a branch first".into(),
        ));
    }
    let head_ref = String::from_utf8_lossy(head.name_bytes()).into_owned();
    let mut status = StatusOptions::new();
    status.include_untracked(false);
    if !repo.statuses(Some(&mut status))?.is_empty() {
        return Err(AppError::Agent(
            "the repo has uncommitted changes; commit or stash them first".into(),
        ));
    }
    let ours = head.peel_to_commit()?;
    let theirs = repo
        .find_branch(branch, BranchType::Local)?
        .get()
        .peel_to_commit()?;
    let analysis =
        repo.merge_analysis_for_ref(&head, &[&repo.find_annotated_commit(theirs.id())?])?;

    if analysis.0.contains(MergeAnalysis::ANALYSIS_UP_TO_DATE) {
        return Ok(ours.id().to_string());
    }
    let mut safe = CheckoutBuilder::new();
    safe.safe();
    if analysis.0.contains(MergeAnalysis::ANALYSIS_FASTFORWARD) {
        repo.checkout_tree(theirs.as_object(), Some(&mut safe))?;
        repo.find_reference(&head_ref)?
            .set_target(theirs.id(), &format!("merge {branch}: fast-forward"))?;
        return Ok(theirs.id().to_string());
    }

    let mut merged = repo.merge_commits(&ours, &theirs, None)?;
    if merged.has_conflicts() {
        let paths: Vec<String> = merged
            .conflicts()?
            .filter_map(Result::ok)
            .filter_map(|c| c.our.or(c.their).or(c.ancestor))
            .map(|entry| String::from_utf8_lossy(&entry.path).into_owned())
            .collect();
        return Err(AppError::Agent(format!(
            "merge conflict in {}; nothing was changed",
            paths.join(", ")
        )));
    }
    let who = repo
        .signature()
        .map_err(|_| AppError::Agent("set user.name and user.email in Git config first".into()))?;
    let tree = repo.find_tree(merged.write_tree_to(&repo)?)?;
    repo.checkout_tree(tree.as_object(), Some(&mut safe))?;
    let message = format!("Merge branch '{branch}'");
    let id = repo.commit(Some("HEAD"), &who, &who, &message, &tree, &[&ours, &theirs])?;
    Ok(id.to_string())
}

/// Removes the run's checkout; the branch stays unless `delete_branch`.
pub fn remove_worktree(
    repo_path: &str,
    run_id: &str,
    worktree: &Path,
    branch: Option<&str>,
    delete_branch: bool,
) -> AppResult<()> {
    if let Ok(repo) = Repository::open(repo_path.trim()) {
        if let Ok(registered) = repo.find_worktree(run_id) {
            let mut options = WorktreePruneOptions::new();
            options.valid(true).locked(true).working_tree(true);
            registered.prune(Some(&mut options))?;
        }
        if let (true, Some(name)) = (delete_branch, branch)
            && let Ok(mut found) = repo.find_branch(name, BranchType::Local)
        {
            found.delete()?;
        }
    }
    if worktree.exists() {
        fs::remove_dir_all(worktree)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::worktree;
    use git2::Signature;
    use std::path::PathBuf;

    struct Fixture {
        repo_dir: PathBuf,
        worktrees: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Self {
            let root = std::env::temp_dir()
                .join(format!("kenchamios-review-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&root);
            let repo_dir = root.join("repo");
            fs::create_dir_all(&repo_dir).expect("dir");
            let repo = Repository::init(&repo_dir).expect("init");
            let mut config = repo.config().expect("config");
            config.set_str("user.name", "Test").expect("name");
            config
                .set_str("user.email", "test@example.com")
                .expect("email");
            fs::write(repo_dir.join("a.txt"), "one\n").expect("write");
            commit_all(&repo, "first");
            Self {
                repo_dir,
                worktrees: root.join("worktrees"),
            }
        }

        fn path(&self) -> &str {
            self.repo_dir.to_str().expect("utf-8")
        }

        fn run(&self, id: &str, title: &str) -> worktree::Worktree {
            worktree::create(self.path(), &self.worktrees, id, title).expect("worktree")
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            if let Some(root) = self.repo_dir.parent() {
                let _ = fs::remove_dir_all(root);
            }
        }
    }

    fn commit_all(repo: &Repository, message: &str) -> Oid {
        let mut index = repo.index().expect("index");
        index
            .add_all(["*"], IndexAddOption::DEFAULT, None)
            .expect("add");
        index.write().expect("write index");
        let tree = repo
            .find_tree(index.write_tree().expect("tree"))
            .expect("find");
        let who = Signature::now("Test", "test@example.com").expect("sig");
        let parents: Vec<_> = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .into_iter()
            .collect();
        let refs: Vec<_> = parents.iter().collect();
        repo.commit(Some("HEAD"), &who, &who, message, &tree, &refs)
            .expect("commit")
    }

    #[test]
    fn the_diff_lists_edits_new_and_deleted_files() {
        let fx = Fixture::new("diff");
        let wt = fx.run("run-1", "Edit things");
        fs::write(wt.path.join("a.txt"), "one\ntwo\n").expect("edit");
        fs::write(wt.path.join("b.txt"), "new\n").expect("new");

        let changes = diff(&wt.path, &wt.base_commit).expect("diff");

        let names: Vec<_> = changes
            .files
            .iter()
            .map(|f| (f.path.as_str(), f.status))
            .collect();
        assert_eq!(names, [("a.txt", "modified"), ("b.txt", "added")]);
        assert!(changes.patch.contains("+two"));
        assert!(changes.patch.contains("+new"));
        assert!(changes.secret.is_none());
    }

    #[test]
    fn a_secret_in_an_added_line_blocks_approve() {
        let fx = Fixture::new("secret");
        let wt = fx.run("run-1", "Leak");
        fs::write(
            wt.path.join("env.txt"),
            [
                "key = sk-ant-",
                "abcdefghijklmnopqrstuvwx
",
            ]
            .concat(),
        )
        .expect("write");

        let hit = diff(&wt.path, &wt.base_commit)
            .expect("diff")
            .secret
            .expect("hit");
        assert_eq!(hit.rule, "Anthropic API key");
        assert_eq!(hit.path, "env.txt");
        let err = approve(&wt.path, &wt.base_commit, "feat: leak").expect_err("blocked");
        assert!(err.to_string().contains("Anthropic API key"));
    }

    #[test]
    fn approve_commits_with_the_users_identity_and_no_trailer() {
        let fx = Fixture::new("approve");
        let wt = fx.run("run-1", "Add b");
        fs::write(wt.path.join("b.txt"), "new\n").expect("write");

        let sha = approve(&wt.path, &wt.base_commit, "feat: add b").expect("approve");

        let repo = Repository::open(&wt.path).expect("open");
        let commit = repo
            .find_commit(Oid::from_str(&sha).expect("oid"))
            .expect("commit");
        assert_eq!(commit.message().ok(), Some("feat: add b"));
        assert_eq!(commit.author().name().ok(), Some("Test"));
        assert!(
            approve(&wt.path, &wt.base_commit, "feat: again").is_err(),
            "nothing left to commit"
        );
    }

    #[test]
    fn approve_needs_changes_and_a_message() {
        let fx = Fixture::new("empty");
        let wt = fx.run("run-1", "Nothing");
        assert!(approve(&wt.path, &wt.base_commit, "").is_err());
        let err = approve(&wt.path, &wt.base_commit, "feat: x").expect_err("no changes");
        assert!(err.to_string().contains("no changes"));
    }

    #[test]
    fn merge_fast_forwards_and_updates_the_checkout() {
        let fx = Fixture::new("ff");
        let wt = fx.run("run-1", "Add b");
        fs::write(wt.path.join("b.txt"), "new\n").expect("write");
        let sha = approve(&wt.path, &wt.base_commit, "feat: add b").expect("approve");

        let merged = merge(fx.path(), &wt.branch).expect("merge");

        assert_eq!(merged, sha);
        assert_eq!(
            fs::read_to_string(fx.repo_dir.join("b.txt")).expect("read"),
            "new\n"
        );
    }

    #[test]
    fn merge_makes_a_merge_commit_when_the_branches_diverged() {
        let fx = Fixture::new("diverged");
        let wt = fx.run("run-1", "Add b");
        fs::write(wt.path.join("b.txt"), "new\n").expect("write");
        approve(&wt.path, &wt.base_commit, "feat: add b").expect("approve");
        let repo = Repository::open(fx.path()).expect("open");
        fs::write(fx.repo_dir.join("c.txt"), "mine\n").expect("write");
        commit_all(&repo, "other work");

        let merged = merge(fx.path(), &wt.branch).expect("merge");

        let commit = repo
            .find_commit(Oid::from_str(&merged).expect("oid"))
            .expect("commit");
        assert_eq!(commit.parent_count(), 2);
        assert!(fx.repo_dir.join("b.txt").exists() && fx.repo_dir.join("c.txt").exists());
    }

    #[test]
    fn a_conflict_changes_nothing() {
        let fx = Fixture::new("conflict");
        let wt = fx.run("run-1", "Edit a");
        fs::write(wt.path.join("a.txt"), "agent\n").expect("write");
        approve(&wt.path, &wt.base_commit, "feat: edit a").expect("approve");
        let repo = Repository::open(fx.path()).expect("open");
        fs::write(fx.repo_dir.join("a.txt"), "mine\n").expect("write");
        let before = commit_all(&repo, "my edit");

        let err = merge(fx.path(), &wt.branch).expect_err("conflict");

        assert!(err.to_string().contains("a.txt"));
        assert_eq!(repo.head().expect("head").target(), Some(before));
        assert_eq!(
            fs::read_to_string(fx.repo_dir.join("a.txt")).expect("read"),
            "mine\n"
        );
    }

    #[test]
    fn a_dirty_repo_refuses_the_merge() {
        let fx = Fixture::new("dirty");
        let wt = fx.run("run-1", "Add b");
        fs::write(wt.path.join("b.txt"), "new\n").expect("write");
        approve(&wt.path, &wt.base_commit, "feat: add b").expect("approve");
        fs::write(fx.repo_dir.join("a.txt"), "changed\n").expect("dirty");

        let err = merge(fx.path(), &wt.branch).expect_err("dirty");
        assert!(err.to_string().contains("uncommitted"));
    }

    #[test]
    fn removing_a_worktree_can_also_delete_its_branch() {
        let fx = Fixture::new("remove");
        let wt = fx.run("run-1", "Add b");

        remove_worktree(fx.path(), "run-1", &wt.path, Some(&wt.branch), true).expect("remove");

        assert!(!wt.path.exists());
        let repo = Repository::open(fx.path()).expect("open");
        assert!(repo.find_branch(&wt.branch, BranchType::Local).is_err());
    }
}
