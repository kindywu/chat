use anyhow::Result;
use sqlx::{postgres::PgListener, Pool, Postgres};
use sqlx_db_tester::TestPg;
use std::{env, time::Duration};
use tokio::{
    io::{stdin, AsyncBufReadExt, BufReader},
    time::sleep,
};
use tracing::{error, info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    // 读取当前目录
    println!("{:?}", env::current_dir()?);
    // 读取.env文件，读取数据库地址
    dotenv::from_filename("./notify_server/examples/.env").ok();

    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    // 读取当前目录
    info!("{:?}", env::current_dir()?);
    // 读取.env文件，读取数据库地址
    dotenv::from_filename("./chat_server/examples/.env").ok();
    let url = env::var("DATABASE_URL")?;
    info!("{url}");

    // 初始化测试数据库
    let (tdb, pool) = init_test_db(url.clone()).await?;
    info!("test db name: {}", tdb.dbname);

    // 初始化监听器，拿到新建数据库的URL
    let url = tdb.url();

    start_listener(tdb.url()).await?;

    sleep(Duration::from_secs(3)).await;

    let result = sqlx::query("delete from messages").execute(&pool).await?;

    info!(
        "delete all messages row_affected: {}",
        result.rows_affected()
    );

    let chats: Vec<(i64, Option<String>)> = sqlx::query_as("select id, name from chats")
        .fetch_all(&pool)
        .await?;

    for chat in chats {
        let result = sqlx::query("delete from chats where id = $1")
            .bind(chat.0)
            .execute(&pool)
            .await?;
        info!(
            "delete chat (id={},name={:?}) row_affected: {}",
            chat.0,
            chat.1,
            result.rows_affected()
        );
    }

    // let result = sqlx::query("delete from chats").execute(&pool).await?;
    // info!("delete chats row_affected: {}", result.rows_affected());

    // wait for input
    let mut reader = BufReader::new(stdin());
    let mut line = String::new();
    reader.read_line(&mut line).await?;

    info!("tdb url: {url}");
    Ok(())
}

async fn init_test_db(url: String) -> Result<(TestPg, Pool<Postgres>)> {
    // 创建测试数据库
    let tdb = TestPg::new(url, std::path::Path::new("migrations"));
    let pool = tdb.get_pool().await;

    // 插入准备数据
    let sqls = include_str!("./test.sql").split(';');
    let mut ts = pool.begin().await.expect("begin transaction failed");
    for sql in sqls {
        if sql.trim().is_empty() {
            continue;
        }

        // info!("sql: {sql}");

        sqlx::query(sql)
            .execute(&mut *ts)
            .await
            .expect("execute sql failed");
    }
    ts.commit().await.expect("commit transaction failed");

    Ok((tdb, pool))
}

async fn start_listener(url: String) -> Result<()> {
    let mut listener = PgListener::connect(&url).await?;
    listener.listen("chat_updated").await?;
    listener.listen("chat_message_created").await?;

    tokio::spawn(async move {
        info!("start `PgListener` on {url}");
        loop {
            match listener.recv().await {
                Ok(notification) => {
                    warn!("Received notification: {:?}", notification);
                }
                Err(e) => {
                    error!("Error receiving notification: {}", e);
                    break;
                }
            }
        }
        info!("stop `PgListener` on {url}");
        Ok::<_, anyhow::Error>(())
    });
    Ok(())
}
