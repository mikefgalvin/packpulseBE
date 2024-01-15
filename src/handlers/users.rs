use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

pub async fn get_user() -> impl IntoResponse {
    let person = User {
        id: String::from("0001"),
        first_name: String::from("Mike"),
        last_name: String::from("Galvin"),
    };

    (StatusCode::OK, Json(person))
}

#[derive(Serialize)]
pub struct User {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
}