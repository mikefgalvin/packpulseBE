use axum::{http::{StatusCode,header::{SET_COOKIE, self}, HeaderValue}, response::{IntoResponse, Response}, Json, extract::{State, self}, Extension};
use axum_macros::debug_handler;
use chrono::{Utc, Duration};
use serde::{Serialize, Deserialize};
use serde_json::{Value as JsonValue, json};
use sqlx::{PgPool, Row, FromRow};
use uuid::Uuid;
use std::sync::Arc;
use bcrypt::{hash, DEFAULT_COST};
use jsonwebtoken::{encode, decode, EncodingKey, Validation, Algorithm, DecodingKey, Header};
use cookie::CookieBuilder;
use cookie::time::Duration as CookieDuration;

use crate::auth_middleware::AuthenticatedUser;


// STRUCTS

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: uuid::Uuid, // Subject (user id)
    exp: usize,      // Expiry
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    password_hash: String,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct UserClient {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(sqlx::FromRow)]
struct DbUser {
    id: Uuid,
    first_name: String,
    last_name: String,
    email: String,
    password_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    password: String,
}

#[derive(Serialize)]
pub struct UserWithToken {
    pub user: UserClient,
    pub token: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserWithDetails {
    pub user: User,
    pub organizations: JsonValue,
    pub locations: JsonValue,
}

// HELPERS

fn generate_jwt(user_id: uuid::Uuid) -> String {
    let expiration = (Utc::now() + Duration::days(30)).timestamp() as usize;
    let claims = Claims { sub: user_id, exp: expiration };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret("your_secret".as_ref())).unwrap()
}

fn validate_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret("your_secret".as_ref()),
        &Validation::new(Algorithm::HS256),
    ).map(|data| data.claims)
}

async fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

// ENDPOINTS
pub async fn register_user(
    State(pool): State<Arc<PgPool>>,
    extract::Json(payload): extract::Json<NewUser>,
) -> impl IntoResponse {

    let hashed_password = match hash_password(&payload.password).await {
        Ok(hash) => hash,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to hash password".to_string()
            ).into_response();
        }
    };

    match sqlx::query_as::<_, UserClient>(
        "INSERT INTO users (first_name, last_name, email, password_hash) VALUES ($1, $2, $3, $4) RETURNING id, first_name, last_name, email")
        .bind(&payload.first_name)
        .bind(&payload.last_name)
        .bind(&payload.email)
        .bind(&hashed_password)
        .fetch_one(&*pool)
        .await {
            Ok(user) => {
                // Create JWT token
                let token = generate_jwt(user.id);
                // Create the cookie
                let cookie = CookieBuilder::new("token", token)
                    .http_only(true) // HTTP-only for security
                    .secure(true) // Secure, sent over HTTPS only
                    .path("/") // Available on all paths
                    .max_age(CookieDuration::days(60))
                    .build();

                // Construct the response
                let response_body = json!({
                    "success": true,
                    "user": {
                        "id": user.id,
                        "first_name": user.first_name,
                        "last_name": user.last_name,
                        "email": user.email
                    }
                });

                let cookie = cookie.to_string();

                (
                    StatusCode::OK,
                    [(header::SET_COOKIE, cookie)],
                    Json(response_body),
                ).into_response()

            },
            Err(e) => {
                eprintln!("Failed to execute query: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to register user".to_string()).into_response()
            },
    }
}

pub async fn login_user(
    Extension(pool): Extension<Arc<PgPool>>,
    extract::Json(payload): extract::Json<LoginRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    let db_user = sqlx::query_as::<_, DbUser>("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_one(&*pool)
        .await;

    match db_user {
        Ok(user) => {
            if bcrypt::verify(&payload.password, &user.password_hash).unwrap_or(false) {
                let token = generate_jwt(user.id);
                Ok((StatusCode::OK, token))
            } else {
                Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))
            }
        },
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())),
    }
}

pub async fn get_user(
    Extension(pool): Extension<Arc<PgPool>>,
    Extension(auth_user): Extension<AuthenticatedUser>,
) -> impl IntoResponse {
    // Use the user_id from the AuthenticatedUser extension
    let user_id = auth_user.id;

    let result = sqlx::query(
        "SELECT 
                    json_build_object(
                        'id', u.id, 
                        'first_name', u.first_name, 
                        'last_name', u.last_name, 
                        'email', u.email,
                        'organizations', (SELECT json_agg(json_build_object('id', o.id, 'name', o.name))
                                          FROM organizations o
                                          JOIN org_staff os ON o.id = os.organization_id
                                          WHERE os.user_id = u.id::uuid),
                        'locations', (SELECT json_agg(json_build_object('id', l.id, 'name', l.name, 'type', l.type))
                                      FROM locations l
                                      JOIN org_locations ol ON l.id = ol.location_id
                                      JOIN organizations o ON ol.organization_id = o.id
                                      JOIN org_staff os ON o.id = os.organization_id 
                                      WHERE os.user_id = u.id::uuid)
                    ) as user_data
                FROM 
                    users u
                    WHERE u.id = $1::uuid")
        .bind(user_id)
        .fetch_one(&*pool)
        .await;

        match result {
            Ok(row) => {
                let user_data: serde_json::Value = row.try_get("user_data").unwrap_or_else(|_| json!({}));
                (StatusCode::OK, Json(user_data))
            },
            Err(e) => {
                eprintln!("Failed to execute query: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": "Internal server error"})))
            }
        }
    }
