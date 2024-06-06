use anyhow::{Context, Result};
use chat_server::{get_router, AppState, Config};
use dotenv::dotenv;
use std::env;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    // 读取.env文件
    dotenv().ok();
    // 注册日志
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    // 读取配置文件路径
    let config_path =
        env::var("CHAT_CONFIG_PATH").context("can't read the env $CHAT_CONFIG_PATH$")?;
    // info!("config_path: {}", config_path);

    // 读取配置
    let config = Config::try_new(&config_path).await?;
    // info!("config: {:?}", config);

    // 构造监听路径
    let addr = format!("{}:{}", config.listen.host, config.listen.port);
    // info!("addr: {:?}", addr);

    // 构造应用状态
    let app_state = AppState::try_new(config).await?;
    // info!("app_state: {:?}", app_state);

    // 构造应用路由
    let app = get_router(app_state);

    // 监听端口
    let listener = TcpListener::bind(&addr).await?;
    info!("chat server listen on {addr}");

    // 绑定axum处理
    axum::serve(listener, app).await?;
    info!("chat server quit");

    Ok(())
}
