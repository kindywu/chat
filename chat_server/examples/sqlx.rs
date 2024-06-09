use anyhow::Result;
use sqlx_db_tester::TestPg;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    // 读取当前目录
    println!("{:?}", env::current_dir()?);

    // 读取.env文件
    dotenv::from_filename("./chat_server/examples/.env").ok();
    let url = env::var("DATABASE_URL")?;
    println!("{url}");

    let tdb = TestPg::new(url, std::path::Path::new("migrations"));
    let pool = tdb.get_pool().await;

    // run prepared sql to insert test dat
    let sqls = include_str!("./test.sql").split(';');
    let mut ts = pool.begin().await.expect("begin transaction failed");
    for sql in sqls {
        if sql.trim().is_empty() {
            continue;
        }

        println!("sql: {sql}");

        sqlx::query(sql)
            .execute(&mut *ts)
            .await
            .expect("execute sql failed");
    }
    ts.commit().await.expect("commit transaction failed");

    Ok(())
}
