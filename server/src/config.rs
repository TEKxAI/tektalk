use anyhow::{Context, Result};
use std::{env, net::SocketAddr};

#[derive(Clone)]
pub struct Config {
    pub bind: SocketAddr,
    pub realtime_tcp_bind: SocketAddr,
    pub realtime_tcp_public: String,
    pub realtime_wss_public: String,
    pub database_url: String,
    pub redis_url: String,
    pub business_solutions_url: String,
    pub jwt_secret: String,
    pub otp_hmac_secret: String,
    pub otp_dev_echo: bool,
    pub snowflake_node_id: u16,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bind: env::var("APP_BIND").unwrap_or_else(|_| "0.0.0.0:8080".into()).parse()?,
            realtime_tcp_bind: env::var("REALTIME_TCP_BIND").unwrap_or_else(|_| "0.0.0.0:8081".into()).parse()?,
            realtime_tcp_public: env::var("REALTIME_TCP_PUBLIC").unwrap_or_else(|_| "tcp://127.0.0.1:8081".into()),
            realtime_wss_public: env::var("REALTIME_WSS_PUBLIC").unwrap_or_else(|_| "ws://127.0.0.1:8080/v1/realtime/connect".into()),
            database_url: env::var("DATABASE_URL").context("DATABASE_URL is required")?,
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".into()),
            business_solutions_url: env::var("BUSINESS_SOLUTIONS_URL").unwrap_or_else(|_| "http://127.0.0.1:50056".into()),
            jwt_secret: required_secret("JWT_SECRET")?,
            otp_hmac_secret: required_secret("OTP_HMAC_SECRET")?,
            otp_dev_echo: env::var("OTP_DEV_ECHO").as_deref() == Ok("true"),
            snowflake_node_id: env::var("SNOWFLAKE_NODE_ID").unwrap_or_else(|_| "0".into()).parse().context("SNOWFLAKE_NODE_ID must be 0..1023")?,
        })
    }
}

fn required_secret(name: &str) -> Result<String> {
    let value = env::var(name).with_context(|| format!("{name} is required"))?;
    anyhow::ensure!(value.len() >= 32, "{name} must be at least 32 bytes");
    Ok(value)
}

