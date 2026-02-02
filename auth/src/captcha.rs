// ============================================
// Proof-of-Work CAPTCHA
// CPU-based challenge without external services
// ============================================

use anyhow::{anyhow, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// CAPTCHA challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptchaChallenge {
    pub challenge: String,
    pub difficulty: u8,
    pub timestamp: i64,
}

/// Proof of work data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfWork {
    pub challenge: String,
    pub solution: u64,
}

/// CAPTCHA verifier
pub struct CaptchaVerifier {
    difficulty: u8,
}

impl CaptchaVerifier {
    /// Create a new CAPTCHA verifier
    pub fn new(difficulty: u8) -> Self {
        Self { difficulty }
    }

    /// Generate a new challenge
    pub fn generate_challenge(&self) -> Result<CaptchaChallenge> {
        let random_bytes: [u8; 32] = rand::thread_rng().gen();
        let challenge = hex::encode(random_bytes);

        Ok(CaptchaChallenge {
            challenge,
            difficulty: self.difficulty,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    /// Verify a proof-of-work solution
    pub fn verify(&self, challenge: &str, solution: u64) -> Result<bool> {
        let pow = ProofOfWork {
            challenge: challenge.to_string(),
            solution,
        };

        let hash = self.compute_hash(&pow);
        Ok(self.check_difficulty(&hash))
    }

    /// Compute hash for proof-of-work
    fn compute_hash(&self, pow: &ProofOfWork) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(pow.challenge.as_bytes());
        hasher.update(&pow.solution.to_le_bytes());
        hasher.finalize().to_vec()
    }

    /// Check if hash meets difficulty requirement
    fn check_difficulty(&self, hash: &[u8]) -> bool {
        let leading_zeros = self.count_leading_zeros(hash);
        leading_zeros >= self.difficulty
    }

    /// Count leading zero bits in hash
    fn count_leading_zeros(&self, hash: &[u8]) -> u8 {
        let mut count = 0u8;
        for &byte in hash {
            if byte == 0 {
                count += 8;
            } else {
                count += byte.leading_zeros() as u8;
                break;
            }
        }
        count
    }

    /// Solve the challenge (for testing/demonstration only)
    /// In production, this should be done client-side
    #[allow(dead_code)]
    pub fn solve(&self, challenge: &str) -> Result<u64> {
        let mut solution = 0u64;
        loop {
            if self.verify(challenge, solution)? {
                return Ok(solution);
            }
            solution += 1;
            if solution > 10_000_000 {
                return Err(anyhow!("Failed to solve challenge"));
            }
        }
    }
}

/// Rate limiter for CAPTCHA challenges
pub struct CaptchaRateLimiter {
    max_attempts: u32,
    window_seconds: u64,
}

impl CaptchaRateLimiter {
    /// Create a new rate limiter
    pub fn new(max_attempts: u32, window_seconds: u64) -> Self {
        Self {
            max_attempts,
            window_seconds,
        }
    }

    /// Check if an IP address is allowed to attempt
    /// In production, this would integrate with Redis
    pub fn is_allowed(&self, _ip_hash: &str) -> bool {
        // TODO: Implement Redis-based rate limiting
        true
    }

    /// Record an attempt
    pub fn record_attempt(&self, _ip_hash: &str) -> Result<()> {
        // TODO: Implement Redis-based attempt tracking
        Ok(())
    }
}

// Note: Using hex crate for encoding
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_generation() {
        let verifier = CaptchaVerifier::new(4);
        let challenge = verifier.generate_challenge().unwrap();
        assert!(!challenge.challenge.is_empty());
        assert_eq!(challenge.difficulty, 4);
    }

    #[test]
    fn test_difficulty_checking() {
        let verifier = CaptchaVerifier::new(4);
        
        // Hash with leading zeros
        let hash_with_zeros = vec![0u8, 0u8, 0u8, 0u8, 0xFF];
        assert!(verifier.check_difficulty(&hash_with_zeros));
        
        // Hash without enough leading zeros
        let hash_without_zeros = vec![0xFF, 0xFF, 0xFF, 0xFF];
        assert!(!verifier.check_difficulty(&hash_without_zeros));
    }

    #[test]
    fn test_proof_of_work() {
        let verifier = CaptchaVerifier::new(2); // Low difficulty for testing
        let challenge = verifier.generate_challenge().unwrap();
        
        // Try to solve (with low difficulty)
        let solution = verifier.solve(&challenge.challenge).unwrap();
        
        // Verify solution
        assert!(verifier.verify(&challenge.challenge, solution).unwrap());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = CaptchaRateLimiter::new(5, 60);
        let ip_hash = "test-ip-hash";
        
        assert!(limiter.is_allowed(ip_hash));
        limiter.record_attempt(ip_hash).unwrap();
    }
}
