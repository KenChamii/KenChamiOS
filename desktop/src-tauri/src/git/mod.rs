mod reader;

pub use reader::read;

use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CommitInfo {
    pub hash: String,
    pub summary: String,
    pub author: String,
    pub time: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RepoInfo {
    /// None while HEAD is detached or the repo has no commits yet
    pub branch: Option<String>,
    /// None while the repo has no commits yet
    pub head: Option<String>,
    pub dirty_files: usize,
    pub commits: Vec<CommitInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum GitInfo {
    NoRepoPath,
    NotARepo,
    Repo(RepoInfo),
}

impl GitInfo {
    pub fn branch(&self) -> Option<&str> {
        match self {
            Self::Repo(repo) => repo.branch.as_deref(),
            _ => None,
        }
    }

    pub fn head(&self) -> Option<&str> {
        match self {
            Self::Repo(repo) => repo.head.as_deref(),
            _ => None,
        }
    }
}
