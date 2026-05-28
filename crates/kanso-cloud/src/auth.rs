//! JWT issuing and verification (HS256).
//!
//! Tokens carry the user id (`sub`) and the device id, so push/pull derive both
//! from the verified token rather than trusting the request body.

use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// User id.
    pub sub: String,
    pub device_id: String,
    /// Expiry, seconds since the Unix epoch.
    pub exp: usize,
}

/// HMAC keys derived from the server secret. Cheap to clone.
#[derive(Clone)]
pub struct JwtKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl JwtKeys {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }

    /// Issue a token valid for `ttl_secs` seconds.
    pub fn issue(
        &self,
        user_id: &str,
        device_id: &str,
        ttl_secs: i64,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let exp = (now_secs() + ttl_secs).max(0) as usize;
        let claims = Claims {
            sub: user_id.to_string(),
            device_id: device_id.to_string(),
            exp,
        };
        encode(&Header::default(), &claims, &self.encoding)
    }

    /// Verify a token's signature and expiry, returning its claims.
    pub fn verify(&self, token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let data = decode::<Claims>(token, &self.decoding, &Validation::default())?;
        Ok(data.claims)
    }
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issued_token_verifies_and_carries_claims() {
        let keys = JwtKeys::new(b"test-secret");
        let token = keys.issue("user:1", "device:1", 3600).unwrap();
        let claims = keys.verify(&token).unwrap();
        assert_eq!(claims.sub, "user:1");
        assert_eq!(claims.device_id, "device:1");
    }

    #[test]
    fn tampered_or_foreign_token_is_rejected() {
        let keys = JwtKeys::new(b"secret-a");
        let other = JwtKeys::new(b"secret-b");
        let token = keys.issue("user:1", "device:1", 3600).unwrap();
        assert!(other.verify(&token).is_err());
    }

    #[test]
    fn expired_token_is_rejected() {
        let keys = JwtKeys::new(b"secret");
        // Well past the default 60s clock-skew leeway.
        let token = keys.issue("user:1", "device:1", -3600).unwrap();
        assert!(keys.verify(&token).is_err());
    }
}
