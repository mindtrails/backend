use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, RwLock,
    },
    time::Duration,
};

use rand::RngCore;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use thiserror::Error;

pub(in crate::http) mod extractor;
mod store;

pub use store::Store;

pub(in crate::http) const SESSION_COOKIE_NAME: &str = "mindtrails_session";

fn generate_cookie(len: usize) -> String
{
    let mut key = vec![0; len];
    rand::thread_rng().fill_bytes(&mut key);

    base64::encode(key)
}

#[derive(Debug, Serialize, Deserialize)]
pub(in crate::http) struct Session
{
    id: String,
    expires_in: Option<Duration>,
    data: Arc<RwLock<HashMap<String, String>>>,

    #[serde(skip)]
    cookie_value: Option<String>,
    #[serde(skip)]
    data_changed: Arc<AtomicBool>,
}

impl Session
{
    pub(in crate::http) fn new() -> Self
    {
        let cookie = generate_cookie(64);
        // SAFETY: This cannot fail as the cookie is not mutated between the
        // base64 encoding and the base64 decoding, which is the only step at
        // which the below call could fail
        let id = Session::id_from_cookie(&cookie).unwrap();

        Session {
            id,
            expires_in: None,
            data: Arc::new(RwLock::new(HashMap::new())),

            cookie_value: Some(cookie),
            data_changed: Arc::new(AtomicBool::new(false)),
        }
    }

    fn id_from_cookie(cookie: &str) -> Result<String, self::Error>
    {
        let decoded = base64::decode(cookie)?;
        let hash = blake3::hash(&decoded);

        Ok(base64::encode(hash.as_bytes()))
    }

    async fn get<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        // TODO: Figure out how ok it is to unwrap here
        let data = self.data.read().unwrap();
        let value = data.get(key)?;

        serde_json::from_str(value).ok()?
    }

    pub(in crate::http) async fn insert<V>(
        &mut self,
        key: &str,
        value: V,
    ) -> Result<(), self::Error>
    where
        V: Serialize,
    {
        self.insert_raw(key, serde_json::to_string(&value)?).await;

        Ok(())
    }

    async fn insert_raw(&mut self, key: &str, value: String)
    {
        // TODO: Same as line 75
        let mut data = self.data.write().unwrap();
        if data.get(key) != Some(&value) {
            let _previous_value = data.insert(String::from(key), value);
            self.data_changed.store(true, Ordering::Relaxed);
        }
    }

    pub(in crate::http) fn into_cookie_value(mut self) -> Option<String>
    {
        self.cookie_value.take()
    }
}

impl Clone for Session
{
    fn clone(&self) -> Self
    {
        Session {
            id: self.id.clone(),
            expires_in: self.expires_in,
            data: self.data.clone(),

            cookie_value: None,
            data_changed: self.data_changed.clone(),
        }
    }
}

#[derive(Debug, Error)]
pub(in crate::http) enum Error
{
    #[error("{inner}")]
    Base64Decode
    {
        #[from]
        inner: base64::DecodeError,
    },
    #[error("{inner}")]
    SerdeJson
    {
        #[from]
        inner: serde_json::Error,
    },
    #[error("{inner}")]
    Redis
    {
        #[from]
        inner: redis::RedisError,
    },
    #[error("missing request session store extension")]
    MissingStoreExtension,
    #[error("no session found")]
    NoSessionFound,
}
