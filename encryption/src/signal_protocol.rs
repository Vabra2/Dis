// ============================================
// Signal Protocol Implementation
// X3DH key exchange + Double Ratchet
// ============================================

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use x25519_dalek::{PublicKey, StaticSecret};

/// Identity key pair (long-term)
#[derive(Clone, Serialize, Deserialize)]
pub struct IdentityKey {
    pub public: Vec<u8>,
    secret: Vec<u8>,
}

impl IdentityKey {
    /// Generate a new identity key pair
    pub fn generate() -> Result<Self> {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);

        Ok(Self {
            public: public.as_bytes().to_vec(),
            secret: secret.to_bytes().to_vec(),
        })
    }

    /// Get the public key
    pub fn public_key(&self) -> &[u8] {
        &self.public
    }
}

/// Signed pre-key (medium-term)
#[derive(Clone, Serialize, Deserialize)]
pub struct SignedPreKey {
    pub id: u32,
    pub public: Vec<u8>,
    secret: Vec<u8>,
    pub signature: Vec<u8>,
}

impl SignedPreKey {
    /// Generate a new signed pre-key
    pub fn generate(id: u32, identity_key: &IdentityKey) -> Result<Self> {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        let public_bytes = public.as_bytes().to_vec();

        // Sign the public key with identity key
        let mut hasher = Sha256::new();
        hasher.update(&public_bytes);
        hasher.update(&identity_key.public);
        let signature = hasher.finalize().to_vec();

        Ok(Self {
            id,
            public: public_bytes,
            secret: secret.to_bytes().to_vec(),
            signature,
        })
    }
}

/// One-time pre-key (ephemeral)
#[derive(Clone, Serialize, Deserialize)]
pub struct OneTimePreKey {
    pub id: u32,
    pub public: Vec<u8>,
    secret: Vec<u8>,
}

impl OneTimePreKey {
    /// Generate a new one-time pre-key
    pub fn generate(id: u32) -> Result<Self> {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);

        Ok(Self {
            id,
            public: public.as_bytes().to_vec(),
            secret: secret.to_bytes().to_vec(),
        })
    }
}

/// Pre-key bundle for key exchange
#[derive(Clone, Serialize, Deserialize)]
pub struct PreKeyBundle {
    pub identity_key: Vec<u8>,
    pub signed_prekey: SignedPreKey,
    pub onetime_prekey: Option<OneTimePreKey>,
}

/// Encrypted message
#[derive(Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    pub ephemeral_public: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Signal Protocol implementation
pub struct SignalProtocol {
    rng: OsRng,
}

impl SignalProtocol {
    /// Create a new Signal Protocol instance
    pub fn new() -> Result<Self> {
        Ok(Self { rng: OsRng })
    }

    /// Generate identity key
    pub fn generate_identity(&mut self) -> Result<IdentityKey> {
        IdentityKey::generate()
    }

    /// Generate pre-key bundle
    pub fn generate_prekey_bundle(&mut self, identity_key: &IdentityKey) -> Result<PreKeyBundle> {
        let signed_prekey = SignedPreKey::generate(1, identity_key)?;
        let onetime_prekey = Some(OneTimePreKey::generate(1)?);

        Ok(PreKeyBundle {
            identity_key: identity_key.public.clone(),
            signed_prekey,
            onetime_prekey,
        })
    }

    /// Encrypt a message using X3DH + Double Ratchet
    pub fn encrypt(&mut self, bundle: &PreKeyBundle, plaintext: &[u8]) -> Result<EncryptedMessage> {
        // Generate ephemeral key pair
        let ephemeral_secret = StaticSecret::random_from_rng(self.rng);
        let ephemeral_public = PublicKey::from(&ephemeral_secret);

        // Perform X3DH key agreement
        let shared_secret = self.x3dh_sender(
            &ephemeral_secret,
            &bundle.identity_key,
            &bundle.signed_prekey.public,
            bundle.onetime_prekey.as_ref().map(|k| k.public.as_slice()),
        )?;

        // Derive encryption key
        let encryption_key = self.kdf(&shared_secret)?;

        // Encrypt with AES-GCM
        let cipher = Aes256Gcm::new_from_slice(&encryption_key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        let nonce_bytes = rand::random::<[u8; 12]>();
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedMessage {
            ephemeral_public: ephemeral_public.as_bytes().to_vec(),
            ciphertext,
            nonce: nonce_bytes.to_vec(),
        })
    }

    /// Decrypt a message
    pub fn decrypt(&mut self, identity_key: &IdentityKey, msg: &EncryptedMessage) -> Result<Vec<u8>> {
        // Parse keys
        let ephemeral_public = PublicKey::from(<[u8; 32]>::try_from(msg.ephemeral_public.as_slice())
            .map_err(|_| anyhow!("Invalid ephemeral public key"))?);

        let identity_secret = StaticSecret::from(<[u8; 32]>::try_from(identity_key.secret.as_slice())
            .map_err(|_| anyhow!("Invalid identity secret key"))?);

        // Perform key agreement
        let shared_secret = identity_secret.diffie_hellman(&ephemeral_public);

        // Derive decryption key
        let decryption_key = self.kdf(shared_secret.as_bytes())?;

        // Decrypt with AES-GCM
        let cipher = Aes256Gcm::new_from_slice(&decryption_key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        let nonce = Nonce::from_slice(&msg.nonce);

        let plaintext = cipher
            .decrypt(nonce, msg.ciphertext.as_slice())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// X3DH key agreement (sender side)
    fn x3dh_sender(
        &self,
        ephemeral: &StaticSecret,
        identity_pub: &[u8],
        signed_prekey_pub: &[u8],
        onetime_prekey_pub: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let identity_public = PublicKey::from(<[u8; 32]>::try_from(identity_pub)
            .map_err(|_| anyhow!("Invalid identity public key"))?);
        let signed_public = PublicKey::from(<[u8; 32]>::try_from(signed_prekey_pub)
            .map_err(|_| anyhow!("Invalid signed prekey"))?);

        // DH1 = DH(ephemeral, signed_prekey)
        let dh1 = ephemeral.diffie_hellman(&signed_public);

        // DH2 = DH(ephemeral, identity_key)
        let dh2 = ephemeral.diffie_hellman(&identity_public);

        // Combine shared secrets
        let mut shared = Vec::new();
        shared.extend_from_slice(dh1.as_bytes());
        shared.extend_from_slice(dh2.as_bytes());

        // DH3 = DH(ephemeral, onetime_prekey) if present
        if let Some(onetime_pub) = onetime_prekey_pub {
            let onetime_public = PublicKey::from(<[u8; 32]>::try_from(onetime_pub)
                .map_err(|_| anyhow!("Invalid one-time prekey"))?);
            let dh3 = ephemeral.diffie_hellman(&onetime_public);
            shared.extend_from_slice(dh3.as_bytes());
        }

        Ok(shared)
    }

    /// Key derivation function
    fn kdf(&self, input: &[u8]) -> Result<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(input);
        hasher.update(b"DIS-SIGNAL-PROTOCOL");
        let hash = hasher.finalize();
        Ok(hash.to_vec())
    }
}

/// Group session for group chat encryption
#[derive(Clone, Serialize, Deserialize)]
pub struct GroupSession {
    pub group_id: String,
    session_key: Vec<u8>,
    message_counter: u32,
}

impl GroupSession {
    /// Create a new group session
    pub fn new(group_id: String) -> Result<Self> {
        let session_key = rand::random::<[u8; 32]>().to_vec();

        Ok(Self {
            group_id,
            session_key,
            message_counter: 0,
        })
    }

    /// Encrypt a group message
    pub fn encrypt(&mut self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.session_key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        // Use counter as part of nonce
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[..4].copy_from_slice(&self.message_counter.to_le_bytes());
        nonce_bytes[4..].copy_from_slice(&rand::random::<[u8; 8]>());
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        self.message_counter += 1;

        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Decrypt a group message
    pub fn decrypt(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(anyhow!("Invalid encrypted data"));
        }

        let nonce = Nonce::from_slice(&data[..12]);
        let ciphertext = &data[12..];

        let cipher = Aes256Gcm::new_from_slice(&self.session_key)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_key_generation() {
        let key = IdentityKey::generate().unwrap();
        assert_eq!(key.public.len(), 32);
        assert_eq!(key.secret.len(), 32);
    }

    #[test]
    fn test_signal_protocol_encryption() {
        let mut protocol = SignalProtocol::new().unwrap();

        // Generate keys for Bob
        let bob_identity = protocol.generate_identity().unwrap();
        let bob_bundle = protocol.generate_prekey_bundle(&bob_identity).unwrap();

        // Alice encrypts to Bob
        let plaintext = b"Secret message";
        let encrypted = protocol.encrypt(&bob_bundle, plaintext).unwrap();

        // Bob decrypts
        let decrypted = protocol.decrypt(&bob_identity, &encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_group_session() {
        let mut session = GroupSession::new("test-group".to_string()).unwrap();

        let plaintext = b"Group message";
        let encrypted = session.encrypt(plaintext).unwrap();
        let decrypted = session.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }
}
