use std::ops::Deref;

use crate::{
    jwt::{DecodingKey, EncodingKey},
    Config,
};
use anyhow::{Context, Result};
use sqlx::PgPool;

pub struct AppState {
    pub(crate) config: Config,
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
        Ok(Self {
            config,
            pool,
            dk,
            ek,
        })
    }
}

impl Deref for AppState {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

#[cfg(test)]
mod test {
    use crate::{
        config::{AuthConfig, DbConfig, FileConfig},
        AppState, Config,
    };
    use anyhow::Result;
    use sqlx::{Pool, Postgres};
    use sqlx_db_tester::TestPg;
    use std::env;

    impl AppState {
        pub async fn try_new_test() -> Result<(TestPg, Self)> {
            // read test db server
            let url = match env::var("DATABASE_URL") {
                Ok(url) => url,
                Err(_) => {
                    // 读取.env文件，读取数据库地址
                    dotenv::from_filename("./chat_server/.env").ok();
                    env::var("DATABASE_URL")?
                }
            };

            println!("test db server: {url}");
            // 初始化测试数据库
            let (tdb, _pool) = AppState::init_test_db(url).await?;
            println!("test db name: {}", tdb.dbname);

            let encoding_pem = include_str!("../fixtures/encoding.pem");
            let decoding_pem = include_str!("../fixtures/decoding.pem");

            let config = Config {
                db: DbConfig { url: tdb.url() },
                file: FileConfig {
                    base_dir: "/tmp/chat_server".into(),
                },
                auth: AuthConfig {
                    sk: encoding_pem.to_string(),
                    pk: decoding_pem.to_string(),
                },
                ..Default::default()
            };

            Ok((tdb, Self::try_new(config).await?))
        }

        async fn init_test_db(url: String) -> Result<(TestPg, Pool<Postgres>)> {
            // 创建测试数据库
            let migrations = std::path::Path::new("../migrations");
            let tdb = TestPg::new(url, migrations);

            let pool = tdb.get_pool().await;

            // 插入准备数据
            let sqls = include_str!("../fixtures/test.sql").split(';');
            let mut ts = pool.begin().await.expect("begin transaction failed");
            for sql in sqls {
                if sql.trim().is_empty() {
                    continue;
                }

                // println!("sql: {sql}");

                sqlx::query(sql)
                    .execute(&mut *ts)
                    .await
                    .expect("execute sql failed");
            }
            ts.commit().await.expect("commit transaction failed");

            Ok((tdb, pool))
        }
    }
}
