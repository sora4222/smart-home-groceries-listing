//! AES-256-GCM encryption for store credentials and intake-channel tokens.
//!
//! The key is a base64-encoded 32 bytes, supplied as `CREDENTIAL_ENCRYPTION_KEY`
//! (generate with `openssl rand -base64 32`). It is never hardcoded, never
//! logged, and never returned to the frontend.
//!
//! GCM is authenticated, so a ciphertext that has been tampered with fails to
//! decrypt rather than yielding altered plaintext. A fresh random nonce is
//! generated per message and stored alongside the ciphertext — reusing a nonce
//! under the same key would be catastrophic for GCM, so nonces are never
//! derived from anything predictable.

use aes_gcm::aead::{Aead, Generate, Key, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;

/// Length of an AES-256 key in bytes.
const KEY_LEN: usize = 32;
/// Length of the GCM nonce in bytes, matching `Aes256Gcm`'s nonce size.
const NONCE_LEN: usize = 12;

/// Why an encrypt or decrypt call could not be completed.
///
/// `Display` deliberately says nothing about the key or the plaintext: these
/// messages reach logs, and one of them reaches an operator.
#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("CREDENTIAL_ENCRYPTION_KEY is not set")]
    MissingKey,

    #[error("CREDENTIAL_ENCRYPTION_KEY must be base64-encoded 32 bytes")]
    InvalidKey,

    #[error("stored ciphertext is malformed")]
    MalformedCiphertext,

    #[error("stored ciphertext failed authentication")]
    NotAuthentic,
}

/// Encrypts and decrypts short secrets with one configured key.
#[derive(Clone)]
pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    /// Builds an encryptor from the base64-encoded key in configuration.
    pub fn from_base64_key(encoded: &str) -> Result<Self, EncryptionError> {
        let encoded = encoded.trim();
        if encoded.is_empty() {
            return Err(EncryptionError::MissingKey);
        }
        let bytes = BASE64
            .decode(encoded)
            .map_err(|_| EncryptionError::InvalidKey)?;
        if bytes.len() != KEY_LEN {
            return Err(EncryptionError::InvalidKey);
        }

        let key = Key::<Aes256Gcm>::try_from(bytes.as_slice())
            .map_err(|_| EncryptionError::InvalidKey)?;
        Ok(Self {
            cipher: Aes256Gcm::new(&key),
        })
    }

    /// Encrypts `plaintext`, returning base64 of `nonce || ciphertext || tag`.
    pub fn encrypt(&self, plaintext: &str) -> Result<String, EncryptionError> {
        let nonce = Nonce::generate();
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            // The only documented failure is a plaintext beyond GCM's length
            // limit, which a credential never is.
            .map_err(|_| EncryptionError::MalformedCiphertext)?;

        let mut envelope = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        envelope.extend_from_slice(&nonce);
        envelope.extend_from_slice(&ciphertext);
        Ok(BASE64.encode(envelope))
    }

    /// Reverses [`Self::encrypt`], rejecting anything that fails its tag.
    pub fn decrypt(&self, encoded: &str) -> Result<String, EncryptionError> {
        let envelope = BASE64
            .decode(encoded.trim())
            .map_err(|_| EncryptionError::MalformedCiphertext)?;
        if envelope.len() <= NONCE_LEN {
            return Err(EncryptionError::MalformedCiphertext);
        }

        let (nonce_bytes, ciphertext) = envelope.split_at(NONCE_LEN);
        let nonce =
            Nonce::try_from(nonce_bytes).map_err(|_| EncryptionError::MalformedCiphertext)?;
        let plaintext = self
            .cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| EncryptionError::NotAuthentic)?;

        String::from_utf8(plaintext).map_err(|_| EncryptionError::MalformedCiphertext)
    }
}

impl std::fmt::Debug for Encryptor {
    /// Prints no key material, so an accidental `{:?}` cannot leak the key.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Encryptor { cipher: <redacted> }")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A deterministic test key. Never used outside tests.
    fn test_key() -> String {
        BASE64.encode([7u8; KEY_LEN])
    }

    #[test]
    fn round_trips_a_credential() {
        let enc = Encryptor::from_base64_key(&test_key()).unwrap();
        let ciphertext = enc.encrypt("hunter2").unwrap();
        assert_ne!(ciphertext, "hunter2");
        assert_eq!(enc.decrypt(&ciphertext).unwrap(), "hunter2");
    }

    #[test]
    fn encrypting_twice_gives_different_ciphertext() {
        let enc = Encryptor::from_base64_key(&test_key()).unwrap();
        assert_ne!(enc.encrypt("same").unwrap(), enc.encrypt("same").unwrap());
    }

    #[test]
    fn tampered_ciphertext_is_rejected() {
        let enc = Encryptor::from_base64_key(&test_key()).unwrap();
        let mut bytes = BASE64.decode(enc.encrypt("hunter2").unwrap()).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0x01;

        assert!(matches!(
            enc.decrypt(&BASE64.encode(bytes)),
            Err(EncryptionError::NotAuthentic)
        ));
    }

    #[test]
    fn a_different_key_cannot_decrypt() {
        let ciphertext = Encryptor::from_base64_key(&test_key())
            .unwrap()
            .encrypt("hunter2")
            .unwrap();
        let other = Encryptor::from_base64_key(&BASE64.encode([9u8; KEY_LEN])).unwrap();

        assert!(matches!(
            other.decrypt(&ciphertext),
            Err(EncryptionError::NotAuthentic)
        ));
    }

    #[test]
    fn rejects_a_missing_or_wrong_sized_key() {
        assert!(matches!(
            Encryptor::from_base64_key("  "),
            Err(EncryptionError::MissingKey)
        ));
        assert!(matches!(
            Encryptor::from_base64_key(&BASE64.encode([0u8; 16])),
            Err(EncryptionError::InvalidKey)
        ));
        assert!(matches!(
            Encryptor::from_base64_key("not base64!!"),
            Err(EncryptionError::InvalidKey)
        ));
    }

    #[test]
    fn rejects_truncated_envelopes() {
        let enc = Encryptor::from_base64_key(&test_key()).unwrap();
        assert!(matches!(
            enc.decrypt(&BASE64.encode([0u8; NONCE_LEN])),
            Err(EncryptionError::MalformedCiphertext)
        ));
    }

    #[test]
    fn debug_output_does_not_contain_key_material() {
        let enc = Encryptor::from_base64_key(&test_key()).unwrap();
        assert_eq!(format!("{enc:?}"), "Encryptor { cipher: <redacted> }");
    }
}
