use argon2::{password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::{AppError, AppResult};

pub fn hash_password(value: &str) -> AppResult<String> {
    validate_password(value)?;
    hash_secret(value)
}

pub fn hash_secret(value: &str) -> AppResult<String> {
    Argon2::default().hash_password(value.as_bytes(), &SaltString::generate(&mut OsRng))
        .map(|h| h.to_string()).map_err(|e| AppError::Internal(anyhow::anyhow!(e)))
}

pub fn verify_password(hash: &str, value: &str) -> bool {
    PasswordHash::new(hash).ok().is_some_and(|h| Argon2::default().verify_password(value.as_bytes(), &h).is_ok())
}

pub fn normalize_answer(value: &str) -> String { value.trim().to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ") }

pub fn hmac_hex(secret: &[u8], value: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret).expect("HMAC accepts any key length");
    mac.update(value.as_bytes());
    hex(mac.finalize().into_bytes().as_slice())
}

pub fn validate_password(value: &str) -> AppResult<()> {
    if value.len() < 10 || !value.chars().any(char::is_uppercase) || !value.chars().any(char::is_numeric) {
        return Err(AppError::Invalid("password must have 10+ chars, an uppercase letter and a number"));
    }
    Ok(())
}

fn hex(bytes: &[u8]) -> String { bytes.iter().map(|b| format!("{b:02x}")).collect() }
