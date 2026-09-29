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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issued_token_round_trips_and_is_bound_to_secret() {
        let user = Uuid::new_v4();
        let device = Uuid::new_v4();
        let token = issue("a-secret-that-is-long-enough-for-tests", user, device).unwrap();
        let claims = verify("a-secret-that-is-long-enough-for-tests", &token).unwrap();
        assert_eq!(claims.sub, user);
        assert_eq!(claims.device, device);
        assert!(claims.exp > claims.iat);
        assert!(matches!(verify("another-secret-that-is-long-enough", &token), Err(AppError::Unauthorized)));
    }

    #[test]
    fn malformed_token_is_rejected() {
        assert!(matches!(verify("a-secret-that-is-long-enough-for-tests", "not-a-jwt"), Err(AppError::Unauthorized)));
    }
}
