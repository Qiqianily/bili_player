use std::str::FromStr;
use tracing_subscriber::{Layer as _, layer::SubscriberExt, util::SubscriberInitExt};

use crate::utils::LocalTimer;

/// 初始化日志相关
///
/// # 参数
/// - logger_level: 日志等级
pub async fn init_logger(logger_level: &str) -> anyhow::Result<()> {
    // set logger level form params if there had error use default info level
    let level = tracing::level_filters::LevelFilter::from_str(logger_level)
        .unwrap_or(tracing::level_filters::LevelFilter::INFO);

    // 自定义日志输出到控制台的格式
    let stdout_layer = tracing_subscriber::fmt::Layer::default()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_target(false)
        .with_timer(LocalTimer)
        .with_writer(std::io::stdout)
        .with_filter(level);

    tracing_subscriber::registry()
        .with(stdout_layer) // 输出到终端
        .init();
    Ok(())
}
