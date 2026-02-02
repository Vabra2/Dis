// ============================================
// Dis Authentication Library
// Anonymous authentication without personal data
// ============================================

pub mod anonymous_token;
pub mod captcha;

use anyhow::Result;

pub use anonymous_token::{AnonymousToken, TokenGenerator};
pub use captcha::{CaptchaChallenge, CaptchaVerifier, ProofOfWork};

/// Authentication service
pub struct AuthService {
    token_generator: TokenGenerator,
    captcha_verifier: CaptchaVerifier,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(jwt_secret: &str, pow_difficulty: u8) -> Result<Self> {
        Ok(Self {
            token_generator: TokenGenerator::new(jwt_secret)?,
            captcha_verifier: CaptchaVerifier::new(pow_difficulty),
        })
    }

    /// Generate a new anonymous token
    pub fn generate_token(&self, user_id: &str) -> Result<AnonymousToken> {
        self.token_generator.generate(user_id)
    }

    /// Verify a token
    pub fn verify_token(&self, token: &str) -> Result<String> {
        self.token_generator.verify(token)
    }

    /// Generate a CAPTCHA challenge
    pub fn generate_captcha(&self) -> Result<CaptchaChallenge> {
        self.captcha_verifier.generate_challenge()
    }

    /// Verify a CAPTCHA solution
    pub fn verify_captcha(&self, challenge: &str, solution: u64) -> Result<bool> {
        self.captcha_verifier.verify(challenge, solution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_service() {
        let service = AuthService::new("test-secret-key", 4).unwrap();
        let user_id = "anonymous-user-123";

        // Generate token
        let token = service.generate_token(user_id).unwrap();
        assert!(!token.token.is_empty());

        // Verify token
        let verified_id = service.verify_token(&token.token).unwrap();
        assert_eq!(verified_id, user_id);
    }

    #[test]
    fn test_captcha_flow() {
        let service = AuthService::new("test-secret-key", 4).unwrap();

        // Generate challenge
        let challenge = service.generate_captcha().unwrap();
        assert!(!challenge.challenge.is_empty());

        // Verify solution (this would normally be solved by the client)
        // For testing, we skip the actual PoW computation
    }
}
