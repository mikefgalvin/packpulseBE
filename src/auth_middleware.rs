use axum::http::{Request, StatusCode, self};
use chrono::Utc;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, errors::ErrorKind};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: uuid::Uuid, // Subject (user id)
    exp: usize,      // Expiry
}

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: uuid::Uuid, // Assuming the `sub` claim is a UUID
}

fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET must be set")
}

fn token_is_valid(token: &str) -> bool {
    let validation = Validation::new(Algorithm::HS256);
    let secret_key = get_jwt_secret();
    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &validation,
    ) {
        Ok(token_data) => {
            // Check if the token is expired
            let timestamp = Utc::now().timestamp() as usize;
            token_data.claims.exp > timestamp
        }
        Err(err) => match *err.kind() {
            ErrorKind::ExpiredSignature => {
                println!("Token has expired.");
                false
            },
            _ => {
                println!("Invalid token: {:?}", err);
                false
            },
        },
    }
}

pub async fn auth<B>(mut request: Request<B>) -> Result<Request<B>, StatusCode> {
        
    let auth_header = request.headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());    

    if let Some(auth_header) = auth_header {
        if token_is_valid(auth_header) { //Custom token validation
            let secret_key = get_jwt_secret(); // Returns secret key from env
            let validation = Validation::new(Algorithm::HS256); // Same Algorithm used to encode
            

            match decode::<Claims>(
                auth_header,
                &DecodingKey::from_secret(secret_key.as_ref()),
                &validation,
            ) {
                Ok(data) => {
                    // Insert AuthenticatedUser into the request extensions
                    let auth_user = AuthenticatedUser { id: data.claims.sub };
                    request.extensions_mut().insert(auth_user);
                    Ok(request)
                },
                Err(_) => Err(StatusCode::UNAUTHORIZED),
            }
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
