use time::{format_description, OffsetDateTime, UtcOffset};

/// 获取当前时间，格式为 "2026-02-10 09:38:48 +0800"
pub fn get_current_time() -> String {
    // 使用 UTC 时间，并手动添加时区偏移
    let utc_now = OffsetDateTime::now_utc();
    
    // 假设本地时区为 +0800（北京时间）
    let offset = UtcOffset::from_hms(8, 0, 0).unwrap_or(UtcOffset::UTC);
    
    // 将 UTC 时间转换为本地时间
    let local_now = utc_now.to_offset(offset);
    format_time(&local_now)
}

/// 格式化时间为 "2026-02-10 09:38:48 +0800" 格式
pub fn format_time(time: &OffsetDateTime) -> String {
    let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour][offset_minute]").unwrap();
    time.format(&format).unwrap()
}

/// 解析 "2026-02-10 09:38:48 +0800" 格式的时间字符串
#[allow(dead_code)]
pub fn parse_time(time_str: &str) -> Result<OffsetDateTime, time::error::Parse> {
    let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] [offset_hour][offset_minute]").unwrap();
    OffsetDateTime::parse(time_str, &format)
}
