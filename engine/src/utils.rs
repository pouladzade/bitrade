/// Generates a unique trade ID
pub fn generate_uuid_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

use bigdecimal::BigDecimal;
use chrono::Utc;

pub fn get_utc_now_time_millisecond() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn is_zero(value: &BigDecimal) -> bool {
    value.with_prec(8) == BigDecimal::from(0)
}

pub fn is_zero_with_precision(value: &BigDecimal, precision: u64) -> bool {
    value.with_prec(precision) == BigDecimal::from(0)
}
