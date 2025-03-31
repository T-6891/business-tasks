mod api;
mod db;
mod models;

use actix_files as fs;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, get};
use dotenv::dotenv;
use r2d2_sqlite::SqliteConnectionManager;
use std::env;
use std::sync::Arc;
use tera::Tera;

use crate::db::{Repository, SqliteRepository};
use crate::models::{User, UserRole};

#[get("/")]
async fn index(
    tmpl: web::Data<Tera>,
    repo: web::Data<Arc<dyn Repository>>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    
    // Получаем списки пользователей для выпадающих списков
    match repo.get_users() {
        Ok(users) => {
            let customers: Vec<&User> = users.iter()
                .filter(|u| u.role == UserRole::Customer)
                .collect();
            
            let executors: Vec<&User> = users.iter()
                .filter(|u| u.role == UserRole::Executor)
                .collect();
            
            ctx.insert("customers", &customers);
            ctx.insert("executors", &executors);
        },
        Err(e) => {
            eprintln!("Ошибка при получении пользователей: {}", e);
            ctx.insert("customers", &Vec::<User>::new());
            ctx.insert("executors", &Vec::<User>::new());
        }
    }
    
    let rendered = match tmpl.render("index.html", &ctx) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Ошибка рендеринга шаблона: {}", e);
            return HttpResponse::InternalServerError().body("Ошибка рендеринга шаблона");
        }
    };
    
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Загружаем переменные среды из .env файла, если он существует
    dotenv().ok();
    
    // Инициализируем логирование
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    // Получаем порт из переменных среды или используем порт по умолчанию
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port = port.parse::<u16>().expect("PORT должен быть числом");
    
    // Получаем путь к базе данных из переменных среды или используем значение по умолчанию
    let db_path = env::var("DATABASE_URL").unwrap_or_else(|_| "business_tasks.db".to_string());
    
    // Настраиваем соединение с базой данных
    let manager = SqliteConnectionManager::file(&db_path);
    let pool = r2d2::Pool::new(manager).expect("Не удалось создать пул соединений");
    
    // Инициализируем базу данных
    {
        let conn = pool.get().expect("Не удалось получить соединение с БД");
        SqliteRepository::init_db(&conn).expect("Не удалось инициализировать БД");
    }
    
    // Создаем репозиторий
    let repo = Arc::new(SqliteRepository::new(Arc::new(pool)));
    
    // Настраиваем шаблонизатор Tera
    let tera = match Tera::new("templates/**/*") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Ошибка компиляции шаблонов: {}", e);
            std::process::exit(1);
        }
    };
    
    println!("Сервер запущен на http://localhost:{}", port);
    
    // Запускаем HTTP-сервер
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::new(repo.clone()))
            .service(index)
            .service(users_page)
            .service(
                web::scope("/api")
                    .configure(api::users::config)
                    .configure(api::tasks::config)
            )
            .service(fs::Files::new("/static", "static").show_files_listing())
            .default_service(web::route().to(|| async {
                HttpResponse::NotFound().body("Страница не найдена")
            }))
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}Ошибка рендеринга шаблона");
        }
    };
    
    HttpResponse::Ok().content_type("text/html").body(rendered)
}

#[get("/users")]
async fn users_page(
    tmpl: web::Data<Tera>,
    repo: web::Data<Arc<dyn Repository>>,
) -> impl Responder {
    let mut ctx = tera::Context::new();
    
    match repo.get_users() {
        Ok(users) => {
            ctx.insert("users", &users);
        },
        Err(e) => {
            eprintln!("Ошибка при получении пользователей: {}", e);
            ctx.insert("users", &Vec::<User>::new());
        }
    }
    
    let rendered = match tmpl.render("users.html", &ctx) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Ошибка рендеринга шаблона: {}", e);
            return HttpResponse::InternalServerError().body("
