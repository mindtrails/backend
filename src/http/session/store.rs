use redis::AsyncCommands;

use crate::http::session;

#[derive(Debug, Clone)]
pub struct Store
{
    client: redis::Client,
}

impl Store
{
    pub fn new(client: redis::Client) -> Self
    {
        Store { client }
    }

    async fn connection(&self) -> Result<redis::aio::Connection, session::Error>
    {
        self.client
            .get_tokio_connection()
            .await
            .map_err(session::Error::from)
    }

    pub(in crate::http) async fn load_session(
        &self,
        cookie: &str,
    ) -> Result<session::Session, session::Error>
    {
        let id = session::Session::id_from_cookie(cookie)?;
        let mut connection = self.connection().await?;

        let record = connection
            .get::<_, Option<String>>(id)
            .await
            .map(|record| record.ok_or_else(|| session::Error::NoSessionFound))
            .map_err(session::Error::from)
            .flatten()?;

        let session = serde_json::from_str(&record)?;

        Ok(session)
    }

    pub(in crate::http) async fn store_session(
        &self,
        session: session::Session,
    ) -> Result<Option<String>, session::Error>
    {
        let record = serde_json::to_string(&session)?;
        let mut connection = self.connection().await?;

        match session.expires_in {
            Some(expiry) => {
                connection
                    .set_ex(session.id.clone(), record, expiry.as_secs() as usize)
                    .await?
            }
            None => connection.set(session.id.clone(), record).await?,
        };

        Ok(session.into_cookie_value())
    }

    pub(in crate::http) async fn destroy_session(
        &self,
        session: session::Session,
    ) -> Result<(), session::Error>
    {
        let mut connection = self.connection().await?;

        connection.del(session.id).await?;

        Ok(())
    }
}
