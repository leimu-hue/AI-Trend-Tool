use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ApiToken {
    pub id: i64,
    pub name: String,
    pub token: String,
    pub last_used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub revoked: bool,
}

/// 列表响应中隐藏 token 明文
#[derive(Debug, Serialize)]
pub struct ApiTokenInfo {
    pub id: i64,
    pub name: String,
    pub last_used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
    pub revoked: bool,
}

impl From<ApiToken> for ApiTokenInfo {
    fn from(t: ApiToken) -> Self {
        ApiTokenInfo {
            id: t.id,
            name: t.name,
            last_used_at: t.last_used_at,
            created_at: t.created_at,
            expires_at: t.expires_at,
            revoked: t.revoked,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
    pub expires_at: Option<NaiveDateTime>,
}
