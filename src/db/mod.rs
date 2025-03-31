pub mod sqlite;

pub use sqlite::SqliteRepository;
pub use sqlite::DbPool;

use crate::models::{Task, User, Tag};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("entity not found: {0}")]
    NotFound(String),
    
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, RepositoryError>;

pub trait Repository: Send + Sync + 'static {
    // User methods
    fn get_users(&self) -> Result<Vec<User>>;
    fn get_user_by_id(&self, id: &str) -> Result<User>;
    fn create_user(&self, user: &User) -> Result<()>;
    fn update_user(&self, user: &User) -> Result<()>;
    fn delete_user(&self, id: &str) -> Result<()>;
    
    // Task methods
    fn get_tasks(&self) -> Result<Vec<Task>>;
    fn get_task_by_id(&self, id: &str) -> Result<Task>;
    fn create_task(&self, task: &Task) -> Result<()>;
    fn update_task(&self, task: &Task) -> Result<()>;
    fn delete_task(&self, id: &str) -> Result<()>;
    
    // Tag methods
    fn get_tags(&self) -> Result<Vec<Tag>>;
    fn get_tag_by_id(&self, id: &str) -> Result<Tag>;
    fn create_tag(&self, tag: &Tag) -> Result<()>;
    fn get_tags_for_task(&self, task_id: &str) -> Result<Vec<Tag>>;
    fn add_tag_to_task(&self, task_id: &str, tag_id: &str) -> Result<()>;
    fn remove_tag_from_task(&self, task_id: &str, tag_id: &str) -> Result<()>;
}
