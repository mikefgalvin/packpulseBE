use axum::{
    extract::{FromRequest, FromRequestParts},
    http::StatusCode, async_trait
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Claims {
    sub: String, // Subject (user id)
    exp: usize,  // Expiry
}

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub id: String,
}

#[async_trait]
impl<B> FromRequest<B> for AuthenticatedUser
where
    B: Send, // required bound
{
    type Rejection = StatusCode;

    async fn from_request(req: &mut FromRequestParts<B>) -> Result<Self, Self::Rejection> {
        if let Some(headers) = req.headers() {
            if let Some(auth_header) = headers.get(http::header::AUTHORIZATION) {
                let token = auth_header.to_str().unwrap_or("");
                let validation = Validation::new(Algorithm::HS256);

                match decode::<Claims>(
                    token,
                    &DecodingKey::from_secret("your_secret_key".as_ref()),
                    &validation,
                ) {
                    Ok(data) => {
                        // Successfully authenticated
                        return Ok(AuthenticatedUser { id: data.claims.sub });
                    },
                    Err(_) => return Err(StatusCode::UNAUTHORIZED),
                }
            }
        }
        Err(StatusCode::UNAUTHORIZED)
    }
}