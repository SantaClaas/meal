use std::sync::Arc;

use axum::http::HeaderValue;
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use hkdf::{self, Hkdf};
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub(super) struct Token([u8; Self::LENGTH]);
#[derive(thiserror::Error, Debug)]
#[error("Error creating token: {0}")]
pub(super) struct CreateError(#[from] getrandom::Error);
#[derive(thiserror::Error, Debug)]
pub(super) enum HashError {
    #[error("Error creating key: {0}")]
    CreateKeyError(hkdf::InvalidPrkLength),
    #[error("Error creating output key material: {0}")]
    CreateOutputKeyMaterialError(hkdf::InvalidLength),
}

#[derive(thiserror::Error, Debug)]
pub(super) enum ValidateError {
    #[error("Error creating hash: {0}")]
    HashError(#[from] HashError),
}

#[derive(Deserialize)]
pub(super) struct ContainerHash {
    token_hash: Arc<[u8]>,
}

impl Token {
    const LENGTH: usize = 128;
    pub(super) fn new() -> Result<Self, CreateError> {
        let mut token = [0; Self::LENGTH];
        getrandom::getrandom(&mut token)?;
        Ok(Self(token))
    }

    pub(super) fn hash(&self) -> Result<impl AsRef<[u8]>, HashError> {
        let hkdf = Hkdf::<Sha256>::from_prk(&self.0).map_err(HashError::CreateKeyError)?;

        let mut output_key_material = [0; 42];
        hkdf.expand(&[], &mut output_key_material)
            .map_err(HashError::CreateOutputKeyMaterialError)?;

        let mut hasher = Sha256::new();
        hasher.update(&output_key_material);

        let hash = hasher.finalize();

        Ok(hash)
    }

    pub(super) fn is_valid(
        &self,
        ContainerHash { token_hash: hash }: ContainerHash,
    ) -> Result<bool, ValidateError> {
        let expected_hash = self.hash()?;
        Ok(expected_hash.as_ref() == hash.as_ref())
    }

    pub(super) fn to_base64(&self) -> Arc<str> {
        BASE64_URL_SAFE_NO_PAD.encode(&self.0).into()
    }
}

#[derive(thiserror::Error, Debug)]
pub(super) enum BadTokenError {
    #[error("Error decoding token: {0}")]
    DecodeError(#[from] base64::DecodeError),
    #[error("Token has bad length. Expected {expected} bytes, got {0}", expected =Token::LENGTH)]
    BadLength(usize),
}

impl TryFrom<&HeaderValue> for Token {
    type Error = BadTokenError;

    fn try_from(header: &HeaderValue) -> Result<Self, Self::Error> {
        let decoded = BASE64_URL_SAFE_NO_PAD.decode(header.as_bytes())?;

        let result =
            <[u8; 128]>::try_from(decoded).map_err(|data| BadTokenError::BadLength(data.len()))?;
        Ok(Self(result))
    }
}
