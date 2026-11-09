use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::Json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum TaskStatus {
    Todo,
    Doing,
    Review,
    Done,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "todo",
            Self::Doing => "doing",
            Self::Review => "review",
            Self::Done => "done",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Priority {
    Low,
    Medium,
    High,
}

impl Priority {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Task {
    pub id: String,
    pub created_at: i64,
    pub project_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub body_md: String,
    pub acceptance_md: String,
    pub related_files: Json<Vec<String>>,
    pub status: TaskStatus,
    pub priority: Priority,
    pub due_at: Option<i64>,
    pub order_key: String,
    pub tags: Json<Vec<String>>,
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskCreate {
    pub project_id: String,
    pub parent_id: Option<String>,
    pub title: String,
    #[serde(default)]
    pub body_md: String,
    #[serde(default)]
    pub acceptance_md: String,
    #[serde(default)]
    pub related_files: Vec<String>,
    pub priority: Priority,
    pub due_at: Option<i64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskUpdate {
    pub title: String,
    pub body_md: String,
    pub acceptance_md: String,
    pub related_files: Vec<String>,
    pub priority: Priority,
    pub due_at: Option<i64>,
    pub tags: Vec<String>,
    #[serde(default)]
    pub recurrence: Option<String>,
}

/// A moved task, and the next occurrence when completing it repeated it.
#[derive(Debug, Clone, Serialize)]
pub struct TaskMoved {
    pub task: Task,
    pub next: Option<Task>,
}
