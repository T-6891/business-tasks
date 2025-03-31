use chrono::{DateTime, Local, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::tag::Tag;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub customer_id: String,
    pub executor_id: String,
    pub created_at: DateTime<Utc>,
    pub due_date: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TaskStatus {
    New,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl Task {
    pub fn new(
        title: String,
        description: String,
        customer_id: String,
        executor_id: String,
        priority: TaskPriority,
        due_date: Option<DateTime<Utc>>,
        tags: Vec<Tag>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            description,
            status: TaskStatus::New,
            priority,
            customer_id,
            executor_id,
            created_at: Utc::now(),
            due_date,
            completed_at: None,
            tags,
        }
    }

    pub fn is_overdue(&self) -> bool {
        if self.status == TaskStatus::Completed || self.status == TaskStatus::Cancelled {
            return false;
        }

        if let Some(due) = self.due_date {
            return Utc::now() > due;
        }

        false
    }

    pub fn overdue_days(&self) -> Option<i64> {
        if !self.is_overdue() {
            return None;
        }

        self.due_date.map(|due| {
            let duration = Utc::now().signed_duration_since(due);
            duration.num_days()
        })
    }
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::New => write!(f, "Новая"),
            TaskStatus::InProgress => write!(f, "В работе"),
            TaskStatus::Completed => write!(f, "Завершена"),
            TaskStatus::Cancelled => write!(f, "Отменена"),
        }
    }
}

impl From<&str> for TaskStatus {
    fn from(s: &str) -> Self {
        match s {
            "new" => TaskStatus::New,
            "in_progress" => TaskStatus::InProgress,
            "completed" => TaskStatus::Completed,
            "cancelled" => TaskStatus::Cancelled,
            _ => TaskStatus::New,
        }
    }
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Низкий"),
            TaskPriority::Medium => write!(f, "Средний"),
            TaskPriority::High => write!(f, "Высокий"),
            TaskPriority::Critical => write!(f, "Критический"),
        }
    }
}

impl From<&str> for TaskPriority {
    fn from(s: &str) -> Self {
        match s {
            "low" => TaskPriority::Low,
            "medium" => TaskPriority::Medium,
            "high" => TaskPriority::High,
            "critical" => TaskPriority::Critical,
            _ => TaskPriority::Medium,
        }
    }
}
