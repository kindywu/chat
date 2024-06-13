use anyhow::Result;
use sqlx::{Pool, Postgres};
use sqlx_db_tester::TestPg;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // 读取当前目录
    println!("{:?}", env::current_dir()?);
    // 读取.env文件，读取数据库地址
    dotenv::from_filename("./chat_server/examples/.env").ok();
    let url = env::var("DATABASE_URL")?;
    println!("{url}");

    // 初始化测试数据库
    let (tdb, pool) = init_test_db(url).await?;
    println!("test db name: {}", tdb.dbname);
    // 开始测试
    let result: Vec<(i64, String)> = sqlx::query_as("select id,fullname from users")
        .fetch_all(&pool)
        .await?;

    for user in result {
        println!("{user:?}");
    }

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

        // println!("sql: {sql}");

        sqlx::query(sql)
            .execute(&mut *ts)
            .await
            .expect("execute sql failed");
    }
    ts.commit().await.expect("commit transaction failed");

    Ok((tdb, pool))
}
