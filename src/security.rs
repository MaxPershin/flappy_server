use axum::{body::Body, extract::Request, middleware::Next, response::Response};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::{error::JwtError, state::AppState};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    pub sub: String,
    pub exp: usize,
    pub role: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
}

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
    pub _leeway: u64,
    pub validation: Validation,
}

impl JwtConfig {
    pub fn new(secret: &str) -> Self {
        let mut validation = Validation::default();
        validation.leeway = 60;
        validation.validate_exp = true;
        validation.validate_nbf = true;

        Self {
            secret: secret.to_string(),
            _leeway: 60,
            validation,
        }
    }
}

pub async fn jwt_middleware(
    req: Request<Body>,
    next: Next,
    state: AppState,
) -> Result<Response, JwtError> {
    let token = req
        .headers()
        .get("Authorization")
        .ok_or(JwtError::MissingAuthHeader)?
        .to_str()
        .map_err(|_| JwtError::InvalidTokenFormat)?
        .strip_prefix("Bearer ")
        .ok_or(JwtError::InvalidTokenFormat)?
        .trim();

    let _claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.jwt_config.secret.as_ref()),
        &state.jwt_config.validation,
    )
    .map_err(|e| JwtError::DecodeError(e))?
    .claims;

    Ok(next.run(req).await)
}

pub fn generate_jwt(user_id: &str, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(1))
        .expect("Invalid timestamp! Server is shutdown!")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
        role: "default".into(),
    };

    encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub async fn validate_user(_username: &String, _password: &String) -> Result<User, String> {
    Ok(User {
        id: "default".into(),
    })
}
