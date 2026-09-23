use anyhow::{Context, Result};
use std::{env, net::SocketAddr};

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub database_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub otp_hmac_secret: String,
    pub otp_dev_echo: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bind: env::var("APP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()).parse()?,
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            jwt_secret: required_secret("JWT_SECRET")?,
            otp_hmac_secret: required_secret("OTP_HMAC_SECRET")?,
            otp_dev_echo: env::var("OTP_DEV_ECHO").as_deref() == Ok("true"),
        })
    }
}

fn required_secret(name: &str) -> Result<String> {
    let value = env::var(name).with_context(|| format!("{name} is required"))?;
    anyhow::ensure!(value.len() >= 32, "{name} must be at least 32 bytes");
    Ok(value)
}
