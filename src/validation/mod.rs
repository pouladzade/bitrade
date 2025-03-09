use crate::grpc::spot::{AddOrderRequest, CreateMarketRequest, DepositRequest, WithdrawRequest};
use anyhow::{anyhow, Context, Result};
use bigdecimal::BigDecimal;
use std::str::FromStr;

/// Validates that a string represents a positive non-zero BigDecimal
pub fn validate_positive_decimal(value: &str, field_name: &str) -> Result<BigDecimal> {
    let decimal = BigDecimal::from_str(value)
        .context(format!("Failed to parse {} as decimal", field_name))?;

    if decimal <= BigDecimal::from(0) {
        return Err(anyhow!("{} must be greater than zero", field_name));
    }

    Ok(decimal)
}

pub fn validate_add_order_request(req: &AddOrderRequest) -> Result<()> {
    // Validate price is positive
    let price = validate_positive_decimal(&req.price, "price")?;

    // Validate base amount is positive
    let base_amount = validate_positive_decimal(&req.base_amount, "base_amount")?;

    // If quote_amount is provided, validate it equals price * base_amount
    if !req.quote_amount.is_empty() {
        let quote_amount = validate_positive_decimal(&req.quote_amount, "quote_amount")?;
        let calculated_quote = &price * &base_amount;

        // Use a small epsilon for floating-point comparison
        let epsilon = BigDecimal::from_str("0.0000001").unwrap();
        if (&calculated_quote - &quote_amount).abs() > epsilon {
            return Err(anyhow!(
                "Quote amount ({}) does not match price * base_amount ({})",
                quote_amount,
                calculated_quote
            ));
        }
    }

    // Validate market ID is not empty
    if req.market_id.is_empty() {
        return Err(anyhow!("Market ID cannot be empty"));
    }

    // Validate user ID is not empty
    if req.user_id.is_empty() {
        return Err(anyhow!("User ID cannot be empty"));
    }

    Ok(())
}

pub fn validate_create_market_request(req: &CreateMarketRequest) -> Result<()> {
    // Validate market ID is not empty
    if req.market_id.is_empty() {
        return Err(anyhow!("Market ID cannot be empty"));
    }

    // Validate base asset is not empty
    if req.base_asset.is_empty() {
        return Err(anyhow!("Base asset cannot be empty"));
    }

    // Validate quote asset is not empty
    if req.quote_asset.is_empty() {
        return Err(anyhow!("Quote asset cannot be empty"));
    }

    // Validate maker fee
    validate_positive_decimal(&req.default_maker_fee, "default_maker_fee")?;

    // Validate taker fee
    validate_positive_decimal(&req.default_taker_fee, "default_taker_fee")?;

    Ok(())
}

// Add more validation functions for other request types as needed
