use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

pub async fn get_people() -> impl IntoResponse {
    let people = vec![
        Person {
            name: String::from("Person A"),
            age: 36,
            favourite_food: Some(String::from("Pizza")),
        },
        Person {
            name: String::from("Person B"),
            age: 5,
            favourite_food: Some(String::from("Broccoli")),
        },
        Person {
            name: String::from("Person Zoolander"),
            age: 100,
            favourite_food: None,
        },
    ];

    (StatusCode::OK, Json(people))
}

pub async fn get_person() -> impl IntoResponse {
    let person = Person {
        name: String::from("PPDB PERSON ZZZZZ 2222"),
        age: 36,
        favourite_food: Some(String::from("Pizza")),
    };

    (StatusCode::OK, Json(person))
}

#[derive(Serialize)]
pub struct Person {
    pub name: String,
    pub age: u32,
    pub favourite_food: Option<String>
}