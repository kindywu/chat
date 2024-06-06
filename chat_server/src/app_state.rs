use crate::{
    jwt::{DecodingKey, EncodingKey},
    Config,
};
use anyhow::{Context, Result};
use sqlx::PgPool;

pub struct AppState {
    pub(crate) pool: PgPool,
    #[allow(dead_code)]
    pub(crate) dk: DecodingKey,
    pub(crate) ek: EncodingKey,
}

impl AppState {
    pub async fn try_new(config: Config) -> Result<Self> {
        let pool = PgPool::connect(&config.db.url)
            .await
            .context("connect to db failed")?;
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let ek = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        Ok(Self { pool, dk, ek })
    }
}
