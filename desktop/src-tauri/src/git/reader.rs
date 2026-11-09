use super::{CommitInfo, GitInfo, RepoInfo};
use crate::error::AppResult;
use git2::{ErrorCode, Repository, StatusOptions};

const COMMIT_LIMIT: usize = 20;

/// Read-only snapshot of the repo at `repo_path`.
pub fn read(repo_path: Option<&str>) -> AppResult<GitInfo> {
    let Some(path) = repo_path.map(str::trim).filter(|p| !p.is_empty()) else {
        return Ok(GitInfo::NoRepoPath);
    };
    // open, not discover: the path must be the repo itself
    let Ok(repo) = Repository::open(path) else {
        return Ok(GitInfo::NotARepo);
    };
    let (branch, head) = match repo.head() {
        Ok(reference) => {
            let branch = reference
                .is_branch()
                .then(|| lossy(reference.shorthand_bytes()));
            (branch, Some(reference.peel_to_commit()?.id().to_string()))
        }
        Err(e) if e.code() == ErrorCode::UnbornBranch => (None, None),
        Err(e) => return Err(e.into()),
    };
    let commits = if head.is_some() {
        recent_commits(&repo)?
    } else {
        Vec::new()
    };
    Ok(GitInfo::Repo(RepoInfo {
        branch,
        head,
        dirty_files: dirty_files(&repo)?,
        commits,
    }))
}

// git allows non-utf8 names, show them with replacement characters
fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn dirty_files(repo: &Repository) -> AppResult<usize> {
    let mut options = StatusOptions::new();
    options.include_untracked(true);
    Ok(repo.statuses(Some(&mut options))?.len())
}

fn recent_commits(repo: &Repository) -> AppResult<Vec<CommitInfo>> {
    let mut walk = repo.revwalk()?;
    walk.push_head()?;
    walk.take(COMMIT_LIMIT)
        .map(|oid| {
            let commit = repo.find_commit(oid?)?;
            Ok(CommitInfo {
                hash: commit.id().to_string(),
                summary: lossy(commit.summary_bytes().unwrap_or_default()),
                author: lossy(commit.author().name_bytes()),
                time: commit.time().seconds() * 1000,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Signature;
    use std::path::Path;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("kenchamios-git-{name}-{}", uuid::Uuid::now_v7()));
        std::fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    fn commit_file(repo: &Repository, name: &str, message: &str) -> git2::Oid {
        let workdir = repo.workdir().expect("workdir");
        std::fs::write(workdir.join(name), message).expect("write file");
        let mut index = repo.index().expect("index");
        index.add_path(Path::new(name)).expect("add");
        index.write().expect("write index");
        let tree = repo
            .find_tree(index.write_tree().expect("tree"))
            .expect("find tree");
        let sig = Signature::now("Tester", "tester@example.com").expect("signature");
        let parents: Vec<git2::Commit> = repo
            .head()
            .ok()
            .and_then(|h| h.peel_to_commit().ok())
            .into_iter()
            .collect();
        let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
            .expect("commit")
    }

    fn repo_info(info: GitInfo) -> RepoInfo {
        match info {
            GitInfo::Repo(repo) => repo,
            other => panic!("expected a repo, got {other:?}"),
        }
    }

    #[test]
    fn missing_or_blank_path_means_no_repo_path() {
        assert_eq!(read(None).expect("read"), GitInfo::NoRepoPath);
        assert_eq!(read(Some("  ")).expect("read"), GitInfo::NoRepoPath);
    }

    #[test]
    fn plain_folder_is_not_a_repo() {
        let dir = temp_dir("plain");
        let info = read(dir.to_str()).expect("read");
        assert_eq!(info, GitInfo::NotARepo);
        assert_eq!(
            read(Some("Z:/does/not/exist")).expect("read"),
            GitInfo::NotARepo
        );
    }

    #[test]
    fn repo_without_commits_has_no_head() {
        let dir = temp_dir("empty");
        Repository::init(&dir).expect("init");
        let repo = repo_info(read(dir.to_str()).expect("read"));
        assert_eq!(repo.head, None);
        assert_eq!(repo.branch, None);
        assert!(repo.commits.is_empty());
    }

    #[test]
    fn reads_branch_head_commits_and_dirty_count() {
        let dir = temp_dir("normal");
        let git = Repository::init(&dir).expect("init");
        commit_file(&git, "a.txt", "first");
        let head = commit_file(&git, "b.txt", "second");
        std::fs::write(dir.join("c.txt"), "new").expect("write");
        std::fs::write(dir.join("a.txt"), "changed").expect("write");

        let repo = repo_info(read(dir.to_str()).expect("read"));

        assert!(repo.branch.is_some());
        assert_eq!(repo.head, Some(head.to_string()));
        assert_eq!(repo.dirty_files, 2);
        let summaries: Vec<&str> = repo.commits.iter().map(|c| c.summary.as_str()).collect();
        assert_eq!(summaries, ["second", "first"]);
        assert_eq!(repo.commits[0].author, "Tester");
    }

    #[test]
    fn commit_list_is_capped_at_twenty() {
        let dir = temp_dir("many");
        let git = Repository::init(&dir).expect("init");
        for i in 0..25 {
            commit_file(&git, "f.txt", &format!("commit {i}"));
        }
        let repo = repo_info(read(dir.to_str()).expect("read"));
        assert_eq!(repo.commits.len(), COMMIT_LIMIT);
        assert_eq!(repo.commits[0].summary, "commit 24");
    }

    #[test]
    fn detached_head_has_hash_but_no_branch() {
        let dir = temp_dir("detached");
        let git = Repository::init(&dir).expect("init");
        let first = commit_file(&git, "a.txt", "first");
        commit_file(&git, "b.txt", "second");
        git.set_head_detached(first).expect("detach");

        let repo = repo_info(read(dir.to_str()).expect("read"));

        assert_eq!(repo.branch, None);
        assert_eq!(repo.head, Some(first.to_string()));
    }
}
