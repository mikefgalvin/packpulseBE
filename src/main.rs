use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

mod handlers;
use crate::handlers::locations::{};
use crate::handlers::organizations::{get_organization_by_id, get_staff_user, create_organization};
use crate::handlers::people::{get_people, get_person};
use crate::handlers::users::{};


pub struct AppState {
    db: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    // Database URL
    let database_url = "postgres://postgres:postgres@localhost:5432/ppdb";

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

    // CORS configuration
    let cors = CorsLayer::new().allow_origin(Any);

    // Building the Axum application
    let app_state = Arc::new(AppState { db: pool });

    // Grouping organization related routes
    let organization_routes = Router::new()
    .route("/:id/staff/:user_id", get(get_staff_user))
    .route("/", post(create_organization));

    // Grouping people related routes
    let people_routes = Router::new()
    .route("/", get(get_people))
    .route("/person", get(get_person));

    let app = Router::new()
        .route("/", get(root))
        .nest("/organizations", organization_routes)
        .nest("/people", people_routes)
        .layer(axum::extract::Extension(app_state.clone()))
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
