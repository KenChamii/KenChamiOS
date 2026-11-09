use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ChecklistItem {
    pub id: String,
    pub created_at: i64,
    pub task_id: String,
    pub text: String,
    pub done: bool,
    pub order_key: String,
}
