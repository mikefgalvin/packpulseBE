use axum::http::{header, self};
use axum::{routing::get, Router, Extension,  http::HeaderValue};
use std::net::SocketAddr;
extern crate dotenv;

use dotenv::dotenv;
use std::env;
use std::str::FromStr;
use tower_http::cors::{CorsLayer, AllowOrigin, AllowMethods, AllowHeaders};
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use http::Method;

mod handlers;
use crate::handlers::locations::{};
use crate::handlers::organizations::{get_organization, get_org_user, get_org_shifts};
use crate::handlers::people::{get_people, get_person};
use crate::handlers::users::{get_user, register_user, login_user};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Setting up the database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Running database migrations
    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        eprintln!("Failed to run database migrations: {:?}", e);
        std::process::exit(1);
    }
    
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers(vec![
            http::header::AUTHORIZATION, 
            http::header::ACCEPT, 
            http::header::CONTENT_TYPE
        ])
        .allow_origin(AllowOrigin::exact(
            HeaderValue::from_str("http://localhost:3000").unwrap()
        ))
        .allow_credentials(true);
    let pool = Arc::new(pool);

    // Routers
    let user_routes = Router::new()
    .route("/:id", get(get_user));

    let organization_routes = Router::new()
    .route("/:id", get(get_organization))
    .route("/:id/me/:user_id", get(get_org_user))
    .route("/:id/shifts", get(get_org_shifts));

    let people_routes = Router::new()
    .route("/", get(get_people))
    .route("/person", get(get_person));

    let app = Router::new()
        .route("/", get(root))
        .route("/register", axum::routing::post(register_user))
        .route("/login", axum::routing::post(login_user))
        .nest("/users", user_routes)
        .nest("/organizations", organization_routes)
        .nest("/people", people_routes)
        .layer(Extension(pool.clone()))
        .layer(cors);

    // Server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    println!("Listening on {}", addr);

    // Starting the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server failed to start");
}

async fn root() -> &'static str {
    "Hello, World!"
}
