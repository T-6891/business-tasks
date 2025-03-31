use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum UserRole {
    Executor,    // Исполнитель
    Customer,    // Заказчик
}

impl User {
    pub fn new(name: String, email: String, role: UserRole) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            email,
            role,
        }
    }
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Executor => write!(f, "Исполнитель"),
            UserRole::Customer => write!(f, "Заказчик"),
        }
    }
}

impl From<&str> for UserRole {
    fn from(s: &str) -> Self {
        match s {
            "executor" => UserRole::Executor,
            "customer" => UserRole::Customer,
            _ => UserRole::Executor, // По умолчанию исполнитель
        }
    }
}
