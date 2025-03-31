use actix_web::{web, HttpResponse, Responder, get, post, put, delete};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::Repository;
use crate::models::{Task, TaskPriority, TaskStatus, Tag};
use super::ApiError;

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
    pub priority: String,
    pub customer_id: String,
    pub executor_id: String,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>, // Список имен тегов
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub customer_id: String,
    pub executor_id: String,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>, // Список имен тегов
}

#[derive(Deserialize)]
pub struct CreateTagRequest {
    pub name: String,
}

#[get("/tasks")]
pub async fn get_tasks(repo: web::Data<Arc<dyn Repository>>) -> Result<impl Responder, ApiError> {
    let tasks = repo.get_tasks()?;
    Ok(HttpResponse::Ok().json(tasks))
}

#[get("/tasks/{id}")]
pub async fn get_task(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<String>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    let task = repo.get_task_by_id(&id)?;
    Ok(HttpResponse::Ok().json(task))
}

#[post("/tasks")]
pub async fn create_task(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<CreateTaskRequest>,
) -> Result<impl Responder, ApiError> {
    // Проверяем, существуют ли заказчик и исполнитель
    let customer = repo.get_user_by_id(&req.customer_id)?;
    let executor = repo.get_user_by_id(&req.executor_id)?;
    
    // Обрабатываем приоритет
    let priority = match req.priority.as_str() {
        "low" => TaskPriority::Low,
        "medium" => TaskPriority::Medium,
        "high" => TaskPriority::High,
        "critical" => TaskPriority::Critical,
        _ => return Err(ApiError::BadRequest("Invalid priority".to_string())),
    };
    
    // Получаем или создаем теги
    let mut tags = Vec::new();
    for tag_name in &req.tags {
        // Ищем тег по имени среди существующих
        let existing_tags = repo.get_tags()?;
        let tag = existing_tags.iter()
            .find(|t| t.name == *tag_name)
            .cloned();
        
        if let Some(tag) = tag {
            tags.push(tag);
        } else {
            // Создаем новый тег
            let new_tag = Tag::new(tag_name.clone());
            repo.create_tag(&new_tag)?;
            tags.push(new_tag);
        }
    }
    
    // Создаем задачу
    let task = Task::new(
        req.title.clone(),
        req.description.clone(),
        req.customer_id.clone(),
        req.executor_id.clone(),
        priority,
        req.due_date,
        tags,
    );
    
    repo.create_task(&task)?;
    
    Ok(HttpResponse::Created().json(task))
}

#[put("/tasks/{id}")]
pub async fn update_task(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<String>,
    req: web::Json<UpdateTaskRequest>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    
    // Получаем существующую задачу
    let mut task = repo.get_task_by_id(&id)?;
    
    // Проверяем, существуют ли заказчик и исполнитель
    let customer = repo.get_user_by_id(&req.customer_id)?;
    let executor = repo.get_user_by_id(&req.executor_id)?;
    
    // Обновляем поля задачи
    task.title = req.title.clone();
    task.description = req.description.clone();
    
    task.status = match req.status.as_str() {
        "new" => TaskStatus::New,
        "in_progress" => TaskStatus::InProgress,
        "completed" => {
            // Если задача завершена, устанавливаем время завершения
            if task.status != TaskStatus::Completed {
                task.completed_at = Some(Utc::now());
            }
            TaskStatus::Completed
        },
        "cancelled" => TaskStatus::Cancelled,
        _ => return Err(ApiError::BadRequest("Invalid status".to_string())),
    };
    
    task.priority = match req.priority.as_str() {
        "low" => TaskPriority::Low,
        "medium" => TaskPriority::Medium,
        "high" => TaskPriority::High,
        "critical" => TaskPriority::Critical,
        _ => return Err(ApiError::BadRequest("Invalid priority".to_string())),
    };
    
    task.customer_id = req.customer_id.clone();
    task.executor_id = req.executor_id.clone();
    task.due_date = req.due_date;
    
    // Обновляем теги
    task.tags.clear();
    for tag_name in &req.tags {
        // Ищем тег по имени среди существующих
        let existing_tags = repo.get_tags()?;
        let tag = existing_tags.iter()
            .find(|t| t.name == *tag_name)
            .cloned();
        
        if let Some(tag) = tag {
            task.tags.push(tag);
        } else {
            // Создаем новый тег
            let new_tag = Tag::new(tag_name.clone());
            repo.create_tag(&new_tag)?;
            task.tags.push(new_tag);
        }
    }
    
    repo.update_task(&task)?;
    
    Ok(HttpResponse::Ok().json(task))
}

#[delete("/tasks/{id}")]
pub async fn delete_task(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<String>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    repo.delete_task(&id)?;
    Ok(HttpResponse::NoContent().finish())
}

#[get("/tags")]
pub async fn get_tags(repo: web::Data<Arc<dyn Repository>>) -> Result<impl Responder, ApiError> {
    let tags = repo.get_tags()?;
    Ok(HttpResponse::Ok().json(tags))
}

#[post("/tags")]
pub async fn create_tag(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<CreateTagRequest>,
) -> Result<impl Responder, ApiError> {
    let tag = Tag::new(req.name.clone());
    repo.create_tag(&tag)?;
    Ok(HttpResponse::Created().json(tag))
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_tasks)
       .service(get_task)
       .service(create_task)
       .service(update_task)
       .service(delete_task)
       .service(get_tags)
       .service(create_tag);
}
