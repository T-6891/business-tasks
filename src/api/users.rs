use actix_web::{web, HttpResponse, Responder, get, post, put, delete};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db::Repository;
use crate::models::{User, UserRole};
use super::ApiError;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    pub name: String,
    pub email: String,
    pub role: String,
}

#[get("/users")]
pub async fn get_users(repo: web::Data<Arc<dyn Repository>>) -> Result<impl Responder, ApiError> {
    let users = repo.get_users()?;
    Ok(HttpResponse::Ok().json(users))
}

#[get("/users/{id}")]
pub async fn get_user(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<String>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    let user = repo.get_user_by_id(&id)?;
    Ok(HttpResponse::Ok().json(user))
}

#[post("/users")]
pub async fn create_user(
    repo: web::Data<Arc<dyn Repository>>,
    req: web::Json<CreateUserRequest>,
) -> Result<impl Responder, ApiError> {
    let user_role = match req.role.as_str() {
        "executor" => UserRole::Executor,
        "customer" => UserRole::Customer,
        _ => return Err(ApiError::BadRequest("Invalid user role".to_string())),
    };

    let user = User::new(req.name.clone(), req.email.clone(), user_role);
    repo.create_user(&user)?;

    Ok(HttpResponse::Created().json(user))
}

#[put("/users/{id}")]
pub async fn update_user(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<String>,
    req: web::Json<UpdateUserRequest>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    
    let mut user = repo.get_user_by_id(&id)?;
    user.name = req.name.clone();
    user.email = req.email.clone();
    
    user.role = match req.role.as_str() {
        "executor" => UserRole::Executor,
        "customer" => UserRole::Customer,
        _ => return Err(ApiError::BadRequest("Invalid user role".to_string())),
    };

    repo.update_user(&user)?;

    Ok(HttpResponse::Ok().json(user))
}

#[delete("/users/{id}")]
pub async fn delete_user(
    repo: web::Data<Arc<dyn Repository>>,
    path: web::Path<String>,
) -> Result<impl Responder, ApiError> {
    let id = path.into_inner();
    repo.delete_user(&id)?;
    Ok(HttpResponse::NoContent().finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(get_users)
       .service(get_user)
       .service(create_user)
       .service(update_user)
       .service(delete_user);
}
