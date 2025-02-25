/// Generates a unique trade ID
pub fn generate_uuid_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

use chrono::Utc;

pub fn get_utc_now_time_millisecond() -> i64 {
    Utc::now().timestamp_millis()
}
