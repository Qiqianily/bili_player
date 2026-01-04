use anyhow::Ok;
use bili_player::logger::init_logger;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_logger("debug").await?;
    tracing::debug!("debug");
    tracing::info!("info");
    tracing::warn!("warn");
    tracing::error!("error");
    Ok(())
}
