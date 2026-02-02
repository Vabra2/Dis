// ============================================
// Dis Encryption Library
// E2E encryption using Signal Protocol
// ============================================

pub mod signal_protocol;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub use signal_protocol::{
    EncryptedMessage, GroupSession, IdentityKey, PreKeyBundle, SignalProtocol, SignedPreKey,
};

/// Main encryption service
pub struct EncryptionService {
    protocol: SignalProtocol,
    group_sessions: HashMap<String, GroupSession>,
}

impl EncryptionService {
    /// Create a new encryption service
    pub fn new() -> Result<Self> {
        Ok(Self {
            protocol: SignalProtocol::new()?,
            group_sessions: HashMap::new(),
        })
    }

    /// Generate a new identity key pair
    pub fn generate_identity(&mut self) -> Result<IdentityKey> {
        self.protocol.generate_identity()
    }

    /// Generate pre-key bundle for key exchange
    pub fn generate_prekey_bundle(&mut self, identity_key: &IdentityKey) -> Result<PreKeyBundle> {
        self.protocol.generate_prekey_bundle(identity_key)
    }

    /// Encrypt a one-to-one message
    pub fn encrypt_message(
        &mut self,
        recipient_bundle: &PreKeyBundle,
        plaintext: &[u8],
    ) -> Result<EncryptedMessage> {
        self.protocol.encrypt(recipient_bundle, plaintext)
    }

    /// Decrypt a one-to-one message
    pub fn decrypt_message(
        &mut self,
        identity_key: &IdentityKey,
        encrypted: &EncryptedMessage,
    ) -> Result<Vec<u8>> {
        self.protocol.decrypt(identity_key, encrypted)
    }

    /// Create a group session for group chat encryption
    pub fn create_group_session(&mut self, group_id: String) -> Result<GroupSession> {
        let session = GroupSession::new(group_id.clone())?;
        self.group_sessions.insert(group_id, session.clone());
        Ok(session)
    }

    /// Encrypt a group message
    pub fn encrypt_group_message(
        &mut self,
        group_id: &str,
        plaintext: &[u8],
    ) -> Result<Vec<u8>> {
        let session = self
            .group_sessions
            .get_mut(group_id)
            .ok_or_else(|| anyhow::anyhow!("Group session not found"))?;
        session.encrypt(plaintext)
    }

    /// Decrypt a group message
    pub fn decrypt_group_message(
        &mut self,
        group_id: &str,
        ciphertext: &[u8],
    ) -> Result<Vec<u8>> {
        let session = self
            .group_sessions
            .get_mut(group_id)
            .ok_or_else(|| anyhow::anyhow!("Group session not found"))?;
        session.decrypt(ciphertext)
    }
}

impl Default for EncryptionService {
    fn default() -> Self {
        Self::new().expect("Failed to create encryption service")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_to_one_encryption() {
        let mut alice = EncryptionService::new().unwrap();
        let mut bob = EncryptionService::new().unwrap();

        // Generate keys
        let alice_identity = alice.generate_identity().unwrap();
        let bob_identity = bob.generate_identity().unwrap();

        // Generate pre-key bundles
        let bob_bundle = bob.generate_prekey_bundle(&bob_identity).unwrap();

        // Alice encrypts message to Bob
        let plaintext = b"Hello Bob!";
        let encrypted = alice.encrypt_message(&bob_bundle, plaintext).unwrap();

        // Bob decrypts message from Alice
        let decrypted = bob.decrypt_message(&bob_identity, &encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_group_encryption() {
        let mut service = EncryptionService::new().unwrap();
        let group_id = "test-group-123".to_string();

        // Create group session
        service.create_group_session(group_id.clone()).unwrap();

        // Encrypt message
        let plaintext = b"Hello everyone!";
        let encrypted = service
            .encrypt_group_message(&group_id, plaintext)
            .unwrap();

        // Decrypt message
        let decrypted = service
            .decrypt_group_message(&group_id, &encrypted)
            .unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }
}
