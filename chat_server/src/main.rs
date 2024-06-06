use std::{env, net::ToSocketAddrs};

use anyhow::{Context, Result};
use chat_server::{get_router, AppState, Config};
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

const CONFIG_PATH: &str = "chat_server\\chat.yaml";

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config_path = env::var("CHAT_CONFIG_PATH").unwrap_or(CONFIG_PATH.to_owned());
    // info!("config_path: {}", config_path);

    let config = Config::try_new(config_path).await?;
    // info!("config: {:?}", config);

    let addr_str = format!("{}:{}", config.listen.host, config.listen.port);

    let addr = addr_str
        .to_socket_addrs()
        .context("parse config host:port to socket addr failed")?
        .next()
        .expect("can't get the socket addr");

    let app_state = AppState::try_new(config).await?;
    // info!("app_state: {:?}", app_state);

    let app = get_router(app_state);

    let listener = TcpListener::bind(addr).await?;
    info!("chat server listen on {addr}");
    axum::serve(listener, app).await?;
    info!("chat server quit");

    Ok(())
}
