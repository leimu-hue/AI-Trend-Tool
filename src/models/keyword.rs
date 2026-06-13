use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, FromRow, Serialize)]
pub struct Keyword {
    pub id: i64,
    pub word: String,
    pub case_sensitive: bool,
    pub enabled: bool,
    pub std_multiplier: f64,
    pub min_hot_count: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateKeywordRequest {
    #[validate(length(min = 1, message = "word must not be empty"))]
    pub word: String,
    pub case_sensitive: Option<bool>,
    #[validate(custom(function = "validate_opt_positive_f64"))]
    pub std_multiplier: Option<f64>,
    #[validate(custom(function = "validate_opt_positive_i32"))]
    pub min_hot_count: Option<i32>,
}

fn validate_opt_positive_f64(v: f64) -> Result<(), validator::ValidationError> {
    if v <= 0.0 {
        let mut err = validator::ValidationError::new("range");
        err.message = Some("std_multiplier must be positive".into());
        return Err(err);
    }
    Ok(())
}

fn validate_opt_positive_i32(v: i32) -> Result<(), validator::ValidationError> {
    if v < 1 {
        let mut err = validator::ValidationError::new("range");
        err.message = Some("min_hot_count must be >= 1".into());
        return Err(err);
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct UpdateKeywordRequest {
    pub word: Option<String>,
    pub case_sensitive: Option<bool>,
    pub enabled: Option<bool>,
    pub std_multiplier: Option<f64>,
    pub min_hot_count: Option<i32>,
}
