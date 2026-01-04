use sqlx::types::chrono;
use sqlx::types::chrono::{NaiveDateTime, Utc};

// Custom FormatTime implementation that formats timestamps
// in the format of "2022-01-01T00:00:00.000"
// custom time format
pub struct LocalTimer;

// east8 timezone
pub const fn east8() -> Option<chrono::FixedOffset> {
    chrono::FixedOffset::east_opt(8 * 3600)
}

// implement the FormatTime trait for LocalTimer
impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Utc::now().with_timezone(&east8().unwrap());
        write!(w, "{}", now.format("%FT%T%.3f"))
    }
}
/// 返回当前东八区时间的 NaiveDateTime（不带时区信息）
pub fn get_local_naive_datetime() -> NaiveDateTime {
    let east8 = east8().unwrap(); // UTC+8 时区
    let now_with_tz = Utc::now().with_timezone(&east8); // 带时区的时间 DateTime<FixedOffset>
    now_with_tz.naive_local() // 转为本地时间的 NaiveDateTime
}
