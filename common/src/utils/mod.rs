use anyhow::{anyhow, Context, Result};
use bigdecimal::BigDecimal;
use chrono::Utc;
use std::str::FromStr;

pub fn generate_uuid_id() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

pub fn get_utc_now_millis() -> i64 {
    Utc::now().timestamp_millis()
}

pub fn get_uuid_string() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn is_zero(value: &BigDecimal) -> bool {
    value.with_prec(8) == BigDecimal::from(0)
}

pub fn is_zero_with_precision(value: &BigDecimal, precision: u64) -> bool {
    value.with_prec(precision) == BigDecimal::from(0)
}

pub fn validate_positive_decimal(value: &str, field_name: &str) -> Result<BigDecimal> {
    let decimal = BigDecimal::from_str(value)
        .context(format!("Failed to parse {} as decimal", field_name))?;

    if decimal <= BigDecimal::from(0) {
        return Err(anyhow!("{} must be greater than zero", field_name));
    }

    Ok(decimal)
}

pub fn bigdecimal_from_str(value: &str, field_name: &str) -> Result<BigDecimal> {
    let decimal = BigDecimal::from_str(value)
        .context(format!("Failed to parse {} as decimal", field_name))?;

    Ok(decimal)
}
