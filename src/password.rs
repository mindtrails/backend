use tokio::task;

use argon2::{
    password_hash::{self, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use thiserror::Error;

pub(crate) async fn hash(password: String) -> Result<String, self::Error>
{
    let password = task::spawn_blocking(move || {
        let salt = SaltString::generate(rand::thread_rng());

        let hashed_password = Argon2::default().hash_password(password.as_bytes(), &salt)?;

        Ok(hashed_password.to_string())
    })
    .await?;

    password
}

pub(crate) async fn verify(password: String, hash: String) -> Result<bool, self::Error>
{
    task::spawn_blocking(move || {
        let hash = PasswordHash::new(&hash).map_err(Error::from)?;

        match Argon2::default().verify_password(password.as_bytes(), &hash) {
            Ok(()) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(err) => Err(err)?,
        }
    })
    .await?
}

#[derive(Debug, Error)]
pub(crate) enum Error
{
    #[error("{inner}")]
    PasswordHash
    {
        #[from]
        inner: password_hash::Error,
    },
    #[error("{inner}")]
    TaskJoin
    {
        #[from]
        inner: task::JoinError,
    },
}
