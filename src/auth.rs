use actix_web::dev::ServiceRequest;
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use crate::models::TokenClaims;

const SECRET: &[u8] = b"super_secret_key_change_me"; // Env var in prod

pub fn create_jwt(id: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp();

    let claims = TokenClaims {
        sub: id.to_owned(),
        role: role.to_owned(),
        exp: expiration as usize,
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(SECRET))
}

pub fn validate_jwt(token: &str) -> Result<TokenClaims, jsonwebtoken::errors::Error> {
    let token_data = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(SECRET),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

// Helper to extract role from request extensions (set by middleware if we had one, or manual check)
pub fn check_role(req: &ServiceRequest, required_role: &str) -> bool {
    // For simplicity in this demo, we'll implement auth checking directly in handlers or a simple extractor
    // But this function exists for architectural completeness
    true
}
