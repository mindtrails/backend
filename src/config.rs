use thiserror::Error;

const FALLBACK_PORT: u16 = 3000;

#[derive(Debug)]
pub struct Config
{
    port: u16,
}

impl Config
{
    pub fn init() -> Result<Self>
    {
        let port = match ::std::env::var("PORT") {
            Ok(port) => port.parse()?,
            Err(::std::env::VarError::NotPresent) => FALLBACK_PORT,
            Err(err) => Err(err)?,
        };

        Ok(Config { port })
    }

    pub fn port(&self) -> u16
    {
        self.port
    }
}

type Result<T> = ::core::result::Result<T, Error>;

use std::{env, num};

#[derive(Debug, Error)]
pub enum Error
{
    #[error("{0}")]
    EnvVar(#[from] env::VarError),
    #[error("{0}")]
    ParseInt(#[from] num::ParseIntError),
}
