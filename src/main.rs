use axum::{
    routing::{get, post}, Router, 
    http::{HeaderValue, self},
    middleware::map_request
};
use std::net::SocketAddr;
extern crate dotenv;

use dotenv::dotenv;
use std::env;
use tower_http::cors::{CorsLayer, AllowOrigin};
use std::sync::Arc;
use sqlx::postgres::PgPoolOptions;
use http::Method;

mod handlers;
use crate::handlers::organizations::{get_organization, get_org_user, get_org_shifts, get_user_org_shifts};
use crate::handlers::people::{get_people, get_person};
use crate::handlers::users::{get_user, register_user, login_user};

mod auth_middleware;
use crate::auth_middleware::auth;


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
        .route("/me", get(get_user))
        .with_state(pool.clone())
        .route_layer(map_request(auth));

    let organization_routes = Router::new()
        .route("/:id", get(get_organization))
        .route("/:id/me/:user_id", get(get_org_user))
        .route("/:id/shifts", get(get_org_shifts))
        .route("/:id/my-shifts", get(get_user_org_shifts))
        .with_state(pool.clone())
        .route_layer(map_request(auth));

    let people_routes = Router::new()
        .route("/", get(get_people))
        .route("/person", get(get_person))
        .with_state(pool.clone())
        .route_layer(map_request(auth));

    let auth_routes = Router::new()
        .route("/register", post(register_user))
        .route("/login", post(login_user))
        .with_state(pool.clone());

    let app = Router::new()
        .nest("/auth", auth_routes)
        .nest("/users", user_routes)
        .nest("/organizations", organization_routes)
        .nest("/people", people_routes)
        .route_layer(cors);

    // Server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    // Starting the server
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("Server failed to start");
}
