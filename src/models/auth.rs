use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshToken {
    pub token: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
    pub user_id: String, // Added user_id to simplify lookup without reverse index
}
