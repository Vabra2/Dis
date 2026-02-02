// ============================================
// Anonymous Token Generation
// JWT-based tokens with anonymous IDs
// ============================================

use anyhow::{anyhow, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

/// Anonymous token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousToken {
    pub token: String,
    pub user_id: String,
    pub expires_at: i64,
}

/// JWT header
#[derive(Debug, Serialize, Deserialize)]
struct JwtHeader {
    alg: String,
    typ: String,
}

/// JWT payload
#[derive(Debug, Serialize, Deserialize)]
struct JwtPayload {
    sub: String,      // User ID
    iat: i64,         // Issued at
    exp: i64,         // Expiration
    jti: String,      // JWT ID (random)
}

/// Token generator
pub struct TokenGenerator {
    jwt_secret: Vec<u8>,
}

impl TokenGenerator {
    /// Create a new token generator
    pub fn new(secret: &str) -> Result<Self> {
        Ok(Self {
            jwt_secret: secret.as_bytes().to_vec(),
        })
    }

    /// Generate a new anonymous user ID
    pub fn generate_user_id() -> String {
        format!("anon_{}", Uuid::new_v4().to_string())
    }

    /// Hash a password with Argon2
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash =
            PasswordHash::new(hash).map_err(|e| anyhow!("Invalid password hash: {}", e))?;

        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Generate an anonymous token
    pub fn generate(&self, user_id: &str) -> Result<AnonymousToken> {
        let now = Utc::now();
        let expiration = now + Duration::days(30);

        let header = JwtHeader {
            alg: "HS256".to_string(),
            typ: "JWT".to_string(),
        };

        let payload = JwtPayload {
            sub: user_id.to_string(),
            iat: now.timestamp(),
            exp: expiration.timestamp(),
            jti: Uuid::new_v4().to_string(),
        };

        // Encode header and payload
        let header_json = serde_json::to_string(&header)?;
        let payload_json = serde_json::to_string(&payload)?;

        let header_b64 = base64::encode_config(header_json, base64::URL_SAFE_NO_PAD);
        let payload_b64 = base64::encode_config(payload_json, base64::URL_SAFE_NO_PAD);

        // Create signature
        let message = format!("{}.{}", header_b64, payload_b64);
        let signature = self.sign(&message)?;
        let signature_b64 = base64::encode_config(signature, base64::URL_SAFE_NO_PAD);

        let token = format!("{}.{}.{}", header_b64, payload_b64, signature_b64);

        Ok(AnonymousToken {
            token,
            user_id: user_id.to_string(),
            expires_at: expiration.timestamp(),
        })
    }

    /// Verify a token and extract user ID
    pub fn verify(&self, token: &str) -> Result<String> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("Invalid token format"));
        }

        let header_b64 = parts[0];
        let payload_b64 = parts[1];
        let signature_b64 = parts[2];

        // Verify signature
        let message = format!("{}.{}", header_b64, payload_b64);
        let expected_signature = self.sign(&message)?;
        let expected_signature_b64 = base64::encode_config(expected_signature, base64::URL_SAFE_NO_PAD);

        if signature_b64 != expected_signature_b64 {
            return Err(anyhow!("Invalid token signature"));
        }

        // Decode payload
        let payload_json = base64::decode_config(payload_b64, base64::URL_SAFE_NO_PAD)
            .map_err(|e| anyhow!("Failed to decode payload: {}", e))?;
        let payload: JwtPayload = serde_json::from_slice(&payload_json)?;

        // Check expiration
        let now = Utc::now().timestamp();
        if payload.exp < now {
            return Err(anyhow!("Token expired"));
        }

        Ok(payload.sub)
    }

    /// Sign a message with HMAC-SHA256
    fn sign(&self, message: &str) -> Result<Vec<u8>> {
        let mut mac = HmacSha256::new_from_slice(&self.jwt_secret)
            .map_err(|e| anyhow!("Failed to create HMAC: {}", e))?;
        mac.update(message.as_bytes());
        let result = mac.finalize();
        Ok(result.into_bytes().to_vec())
    }
}

/// Anonymous registration data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymousRegistration {
    pub user_id: String,
    pub password_hash: String,
    pub encrypted_email: Option<String>,  // Optional, encrypted with AES-256-GCM
}

impl AnonymousRegistration {
    /// Create a new anonymous registration
    pub fn new(password: &str, email: Option<&str>) -> Result<Self> {
        let user_id = TokenGenerator::generate_user_id();
        let password_hash = TokenGenerator::hash_password(password)?;

        let encrypted_email = if let Some(email) = email {
            Some(Self::encrypt_email(email)?)
        } else {
            None
        };

        Ok(Self {
            user_id,
            password_hash,
            encrypted_email,
        })
    }

    /// Encrypt an email address
    fn encrypt_email(email: &str) -> Result<String> {
        use aes_gcm::{
            aead::{Aead, KeyInit},
            Aes256Gcm, Nonce,
        };

        // In production, this should use the EMAIL_ENCRYPTION_KEY from env
        let key = rand::thread_rng().gen::<[u8; 32]>();
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        let nonce_bytes = rand::thread_rng().gen::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, email.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Combine nonce and ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(base64::encode(&result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_id_generation() {
        let id = TokenGenerator::generate_user_id();
        assert!(id.starts_with("anon_"));
    }

    #[test]
    fn test_password_hashing() {
        let password = "secure-password-123";
        let hash = TokenGenerator::hash_password(password).unwrap();
        assert!(TokenGenerator::verify_password(password, &hash).unwrap());
        assert!(!TokenGenerator::verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_token_generation_and_verification() {
        let generator = TokenGenerator::new("test-secret-key").unwrap();
        let user_id = "anon_test_user";

        let token = generator.generate(user_id).unwrap();
        let verified_id = generator.verify(&token.token).unwrap();

        assert_eq!(verified_id, user_id);
    }

    #[test]
    fn test_anonymous_registration() {
        let reg = AnonymousRegistration::new("password123", Some("user@example.com")).unwrap();
        assert!(reg.user_id.starts_with("anon_"));
        assert!(reg.encrypted_email.is_some());
    }
}
