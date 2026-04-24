use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Utc, Duration};
use anyhow::{Result, anyhow};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // User ID
    pub is_admin: bool,
    pub exp: usize,       // Expiration time
}

pub struct AuthManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthManager {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_bytes()),
            decoding_key: DecodingKey::from_secret(secret.as_bytes()),
        }
    }

    /// Hash a password using bcrypt
    pub fn hash_password(password: &str) -> Result<String> {
        hash(password, DEFAULT_COST).map_err(|e| anyhow!("Hashing failed: {}", e))
    }

    /// Verify a password against a hash
    pub fn verify_password(password: &str, hash: &str) -> bool {
        verify(password, hash).unwrap_or(false)
    }

    /// Generate a new JWT token
    pub fn generate_token(&self, user_id: &str, is_admin: bool) -> Result<String> {
        let expiration = Utc::now()
            .checked_add_signed(Duration::days(7))
            .expect("valid timestamp")
            .timestamp();

        let claims = Claims {
            sub: user_id.to_owned(),
            is_admin,
            exp: expiration as usize,
        };

        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| anyhow!("Token generation failed: {}", e))
    }

    /// Validate a JWT token and return claims
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        let validation = Validation::new(Algorithm::HS256);
        let token_data = decode::<Claims>(token, &self.decoding_key, &validation)
            .map_err(|e| anyhow!("Token validation failed: {}", e))?;
        
        Ok(token_data.claims)
    }
}
