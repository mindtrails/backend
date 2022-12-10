use std::{env, num, str};

use thiserror::Error;

const FALLBACK_POSTGRES_URL: &str = "postgres://postgres:postgres@localhost/mindtrails";
const FALLBACK_REDIS_URL: &str = "redis://127.0.0.1/";

const FALLBACK_PORT: u16 = 8080;

const FALLBACK_IN_PRODUCTION: bool = false;

#[derive(Debug)]
pub struct Config
{
    postgres_url: String,
    redis_url: String,

    port: u16,
    in_production: bool,
}

impl Config
{
    pub fn init() -> Result<Self, self::Error>
    {
        let postgres_url = match env::var("POSTGRES_URL") {
            Ok(url) => url,
            Err(env::VarError::NotPresent) => String::from(FALLBACK_POSTGRES_URL),
            Err(err) => Err(err)?,
        };
        let redis_url = match env::var("REDIS_URL") {
            Ok(url) => url,
            Err(env::VarError::NotPresent) => String::from(FALLBACK_REDIS_URL),
            Err(err) => Err(err)?,
        };

        let port = match ::std::env::var("PORT") {
            Ok(port) => port.parse()?,
            Err(env::VarError::NotPresent) => FALLBACK_PORT,
            Err(err) => Err(err)?,
        };

        let in_production = match ::std::env::var("PRODUCTION") {
            Ok(production) => production.parse()?,
            Err(env::VarError::NotPresent) => FALLBACK_IN_PRODUCTION,
            Err(err) => Err(err)?,
        };

        Ok(Config {
            postgres_url,
            redis_url,

            port,
            in_production,
        })
    }

    pub fn postgres_url(&self) -> &str
    {
        &self.postgres_url
    }

    pub fn redis_url(&self) -> &str
    {
        &self.redis_url
    }

    pub fn port(&self) -> u16
    {
        self.port
    }

    pub fn in_production(&self) -> bool
    {
        self.in_production
    }
}

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{inner}")]
    EnvVar
    {
        #[from]
        inner: env::VarError,
    },
    #[error("{inner}")]
    ParseInt
    {
        #[from]
        inner: num::ParseIntError,
    },
    #[error("{inner}")]
    ParseBool
    {
        #[from]
        inner: str::ParseBoolError,
    },
}
