use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Connection, Row};
use serde_json;
use std::sync::Arc;

use crate::models::{Task, TaskPriority, TaskStatus, Tag, User, UserRole};
use super::{Repository, RepositoryError, Result};

pub type DbPool = Pool<SqliteConnectionManager>;

pub struct SqliteRepository {
    pool: Arc<DbPool>,
}

impl SqliteRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    pub fn init_db(conn: &Connection) -> rusqlite::Result<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                role TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                status TEXT NOT NULL,
                priority TEXT NOT NULL,
                customer_id TEXT NOT NULL,
                executor_id TEXT NOT NULL,
                created_at TEXT NOT NULL,
                due_date TEXT,
                completed_at TEXT,
                FOREIGN KEY (customer_id) REFERENCES users (id),
                FOREIGN KEY (executor_id) REFERENCES users (id)
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS task_tags (
                task_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                PRIMARY KEY (task_id, tag_id),
                FOREIGN KEY (task_id) REFERENCES tasks (id),
                FOREIGN KEY (tag_id) REFERENCES tags (id)
            )",
            [],
        )?;

        Ok(())
    }
}

impl Repository for SqliteRepository {
    // User methods
    fn get_users(&self) -> Result<Vec<User>> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, name, email, role FROM users")?;
        let rows = stmt.query_map([], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                role: UserRole::from(row.get::<_, String>(3)?.as_str()),
            })
        })?;

        let mut users = Vec::new();
        for row in rows {
            users.push(row?);
        }

        Ok(users)
    }

    fn get_user_by_id(&self, id: &str) -> Result<User> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, name, email, role FROM users WHERE id = ?")?;
        let user = stmt.query_row(params![id], |row| {
            Ok(User {
                id: row.get(0)?,
                name: row.get(1)?,
                email: row.get(2)?,
                role: UserRole::from(row.get::<_, String>(3)?.as_str()),
            })
        }).map_err(|_| RepositoryError::NotFound(format!("User with id {} not found", id)))?;

        Ok(user)
    }

    fn create_user(&self, user: &User) -> Result<()> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        conn.execute(
            "INSERT INTO users (id, name, email, role) VALUES (?, ?, ?, ?)",
            params![
                user.id,
                user.name,
                user.email,
                format!("{:?}", user.role).to_lowercase(),
            ],
        )?;

        Ok(())
    }

    fn update_user(&self, user: &User) -> Result<()> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let rows_affected = conn.execute(
            "UPDATE users SET name = ?, email = ?, role = ? WHERE id = ?",
            params![
                user.name,
                user.email,
                format!("{:?}", user.role).to_lowercase(),
                user.id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {} not found", user.id)));
        }

        Ok(())
    }

    fn delete_user(&self, id: &str) -> Result<()> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let rows_affected = conn.execute("DELETE FROM users WHERE id = ?", params![id])?;

        if rows_affected == 0 {
            return Err(RepositoryError::NotFound(format!("User with id {} not found", id)));
        }

        Ok(())
    }
    
    // Task methods
    fn get_tasks(&self) -> Result<Vec<Task>> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let mut stmt = conn.prepare(
            "SELECT id, title, description, status, priority, customer_id, executor_id, 
                    created_at, due_date, completed_at 
             FROM tasks"
        )?;
        
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let task = Task {
                id: id.clone(),
                title: row.get(1)?,
                description: row.get(2)?,
                status: TaskStatus::from(row.get::<_, String>(3)?.as_str()),
                priority: TaskPriority::from(row.get::<_, String>(4)?.as_str()),
                customer_id: row.get(5)?,
                executor_id: row.get(6)?,
                created_at: parse_datetime(row.get::<_, String>(7)?)?,
                due_date: row.get::<_, Option<String>>(8)?.map(parse_datetime).transpose()?,
                completed_at: row.get::<_, Option<String>>(9)?.map(parse_datetime).transpose()?,
                tags: Vec::new(), // Заполним позже
            };
            Ok(task)
        })?;

        let mut tasks = Vec::new();
        for row in rows {
            let mut task = row?;
            // Получаем теги для задачи
            task.tags = self.get_tags_for_task(&task.id)?;
            tasks.push(task);
        }

        Ok(tasks)
    }

    fn get_task_by_id(&self, id: &str) -> Result<Task> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let mut stmt = conn.prepare(
            "SELECT id, title, description, status, priority, customer_id, executor_id, 
                    created_at, due_date, completed_at 
             FROM tasks WHERE id = ?"
        )?;
        
        let task = stmt.query_row(params![id], |row| {
            let task_id: String = row.get(0)?;
            let task = Task {
                id: task_id.clone(),
                title: row.get(1)?,
                description: row.get(2)?,
                status: TaskStatus::from(row.get::<_, String>(3)?.as_str()),
                priority: TaskPriority::from(row.get::<_, String>(4)?.as_str()),
                customer_id: row.get(5)?,
                executor_id: row.get(6)?,
                created_at: parse_datetime(row.get::<_, String>(7)?)?,
                due_date: row.get::<_, Option<String>>(8)?.map(parse_datetime).transpose()?,
                completed_at: row.get::<_, Option<String>>(9)?.map(parse_datetime).transpose()?,
                tags: Vec::new(), // Заполним позже
            };
            Ok(task)
        }).map_err(|_| RepositoryError::NotFound(format!("Task with id {} not found", id)))?;

        // Получаем теги для задачи
        let tags = self.get_tags_for_task(id)?;
        
        Ok(Task { tags, ..task })
    }

    fn create_task(&self, task: &Task) -> Result<()> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        conn.execute(
            "INSERT INTO tasks (id, title, description, status, priority, customer_id, executor_id, 
                              created_at, due_date, completed_at) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                task.id,
                task.title,
                task.description,
                format!("{:?}", task.status).to_lowercase(),
                format!("{:?}", task.priority).to_lowercase(),
                task.customer_id,
                task.executor_id,
                task.created_at.to_rfc3339(),
                task.due_date.map(|d| d.to_rfc3339()),
                task.completed_at.map(|d| d.to_rfc3339()),
            ],
        )?;

        // Добавляем теги
        for tag in &task.tags {
            self.add_tag_to_task(&task.id, &tag.id)?;
        }

        Ok(())
    }

    fn update_task(&self, task: &Task) -> Result<()> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let rows_affected = conn.execute(
            "UPDATE tasks 
             SET title = ?, description = ?, status = ?, priority = ?, 
                 customer_id = ?, executor_id = ?, due_date = ?, completed_at = ? 
             WHERE id = ?",
            params![
                task.title,
                task.description,
                format!("{:?}", task.status).to_lowercase(),
                format!("{:?}", task.priority).to_lowercase(),
                task.customer_id,
                task.executor_id,
                task.due_date.map(|d| d.to_rfc3339()),
                task.completed_at.map(|d| d.to_rfc3339()),
                task.id,
            ],
        )?;

        if rows_affected == 0 {
            return Err(RepositoryError::NotFound(format!("Task with id {} not found", task.id)));
        }

        // Обновляем теги: сначала удаляем все, потом добавляем заново
        conn.execute("DELETE FROM task_tags WHERE task_id = ?", params![task.id])?;
        
        for tag in &task.tags {
            self.add_tag_to_task(&task.id, &tag.id)?;
        }

        Ok(())
    }

    fn delete_task(&self, id: &str) -> Result<()> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        // Сначала удаляем связи с тегами
        conn.execute("DELETE FROM task_tags WHERE task_id = ?", params![id])?;
        
        // Затем удаляем саму задачу
        let rows_affected = conn.execute("DELETE FROM tasks WHERE id = ?", params![id])?;

        if rows_affected == 0 {
            return Err(RepositoryError::NotFound(format!("Task with id {} not found", id)));
        }

        Ok(())
    }
    
    // Tag methods
    fn get_tags(&self) -> Result<Vec<Tag>> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, name FROM tags")?;
        let rows = stmt.query_map([], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }

        Ok(tags)
    }

    fn get_tag_by_id(&self, id: &str) -> Result<Tag> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let mut stmt = conn.prepare("SELECT id, name FROM tags WHERE id = ?")?;
        let tag = stmt.query_row(params![id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        }).map_err(|_| RepositoryError::NotFound(format!("Tag with id {} not found", id)))?;

        Ok(tag)
    }

    fn create_tag(&self, tag: &Tag) -> Result<()> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        conn.execute(
            "INSERT INTO tags (id, name) VALUES (?, ?)",
            params![tag.id, tag.name],
        )?;

        Ok(())
    }

    fn get_tags_for_task(&self, task_id: &str) -> Result<Vec<Tag>> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let mut stmt = conn.prepare(
            "SELECT t.id, t.name 
             FROM tags t
             JOIN task_tags tt ON t.id = tt.tag_id
             WHERE tt.task_id = ?"
        )?;
        
        let rows = stmt.query_map(params![task_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;

        let mut tags = Vec::new();
        for row in rows {
            tags.push(row?);
        }

        Ok(tags)
    }

    fn add_tag_to_task(&self, task_id: &str, tag_id: &str) -> Result<()> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        // Проверяем, существует ли задача
        if conn.query_row(
            "SELECT 1 FROM tasks WHERE id = ?",
            params![task_id],
            |_| Ok(()),
        ).is_err() {
            return Err(RepositoryError::NotFound(format!("Task with id {} not found", task_id)));
        }

        // Проверяем, существует ли тег
        if conn.query_row(
            "SELECT 1 FROM tags WHERE id = ?",
            params![tag_id],
            |_| Ok(()),
        ).is_err() {
            return Err(RepositoryError::NotFound(format!("Tag with id {} not found", tag_id)));
        }

        // Добавляем связь (игнорируем, если уже существует)
        conn.execute(
            "INSERT OR IGNORE INTO task_tags (task_id, tag_id) VALUES (?, ?)",
            params![task_id, tag_id],
        )?;

        Ok(())
    }

    fn remove_tag_from_task(&self, task_id: &str, tag_id: &str) -> Result<()> {
        let conn = self.pool.get().map_err(|e| RepositoryError::Internal(e.to_string()))?;
        
        let rows_affected = conn.execute(
            "DELETE FROM task_tags WHERE task_id = ? AND tag_id = ?",
            params![task_id, tag_id],
        )?;

        if rows_affected == 0 {
            return Err(RepositoryError::NotFound(
                format!("Relationship between task {} and tag {} not found", task_id, tag_id)
            ));
        }

        Ok(())
    }
}

// Вспомогательная функция для парсинга DateTime из строки
fn parse_datetime(s: String) -> rusqlite::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))
}
