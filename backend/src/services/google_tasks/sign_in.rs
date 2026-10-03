//! Keeping the Google refresh token: encrypted on the way into the database,
//! decrypted only for the call that needs it. Never logged or returned.

use crate::error::ApiError;
use crate::models::db::GoogleTasksLink;
use crate::services::encryption::Encryptor;

/// The encryptor, or the error that says which setting is missing.
pub fn require(encryptor: Option<&Encryptor>) -> Result<&Encryptor, ApiError> {
    encryptor.ok_or_else(|| ApiError::Misconfigured("CREDENTIAL_ENCRYPTION_KEY is not set".into()))
}

/// Encrypts a refresh token for storage.
pub fn seal(encryptor: Option<&Encryptor>, refresh_token: &str) -> Result<String, ApiError> {
    require(encryptor)?
        .encrypt(refresh_token)
        .map_err(|err| ApiError::Internal(err.into()))
}

/// The stored refresh token, decrypted. A 409 when nobody has connected.
pub fn open(encryptor: Option<&Encryptor>, link: &GoogleTasksLink) -> Result<String, ApiError> {
    let sealed = link
        .refresh_token_encrypted
        .as_deref()
        .ok_or_else(not_connected)?;
    require(encryptor)?
        .decrypt(sealed)
        .map_err(|err| ApiError::Internal(err.into()))
}

/// The 409 for an action that needs a Google sign-in first.
pub fn not_connected() -> ApiError {
    ApiError::Conflict("Google Tasks is not connected".into())
}
