use anyhow::{anyhow, Error};
use chrono::{FixedOffset, Local, Offset};
use tz::{LocalTimeType, TimeZone};

/// Detects the system's current IANA timezone name (e.g. "Asia/Manila").
pub fn system_timezone() -> Result<String, Error> {
    iana_time_zone::get_timezone().map_err(|e| anyhow!("Failed to detect system timezone: {e}"))
}

/// Resolves the UTC `FixedOffset` for an IANA timezone name at a given unix timestamp.
/// Falls back to the machine's local offset if the timezone cannot be resolved.
pub fn iana_to_fixed_offset(iana_name: &str, unix_timestamp: i64) -> FixedOffset {
    let offset_opt = TimeZone::from_posix_tz(iana_name)
        .ok()
        .and_then(|tz| tz.find_local_time_type(unix_timestamp).ok().map(LocalTimeType::ut_offset))
        .and_then(FixedOffset::east_opt);

    offset_opt.unwrap_or_else(|| Local::now().offset().fix())
}

/// Validates whether an IANA timezone string exists on the system.
pub fn validate_iana_timezone(iana_name: &str) -> bool {
    TimeZone::from_posix_tz(iana_name).is_ok()
}
