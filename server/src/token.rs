use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::{AppError, AppResult};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims { pub sub: Uuid, pub device: Uuid, pub exp: usize, pub iat: usize }

pub fn issue(secret: &str, user_id: Uuid, device_id: Uuid) -> AppResult<String> {
    let now = Utc::now();
    let claims = Claims { sub: user_id, device: device_id, iat: now.timestamp() as usize, exp: (now + Duration::minutes(15)).timestamp() as usize };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).map_err(|e| AppError::Internal(e.into()))
}

pub fn verify(secret: &str, value: &str) -> AppResult<Claims> {
    decode::<Claims>(value, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())
        .map(|d| d.claims).map_err(|_| AppError::Unauthorized)
}

