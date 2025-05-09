use axum::extract::FromRef;
use axum_extra::extract::cookie::{Cookie, Key as AxumKey};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use cookie::CookieBuilder;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::OffsetDateTime;

use super::TugState;

#[derive(Clone)]
pub(crate) struct Key(AxumKey);

pub(super) const NAME: &str = "session";
impl Key {
    pub(crate) const LENGTH: usize = 64;
}

impl From<[u8; Key::LENGTH]> for Key {
    fn from(key: [u8; Key::LENGTH]) -> Self {
        Self(AxumKey::from(&key))
    }
}

impl FromRef<TugState> for AxumKey {
    fn from_ref(state: &TugState) -> Self {
        state.cookie_key.0.clone()
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct Session {
    #[serde(with = "time::serde::timestamp")]
    expires_at: OffsetDateTime,
}

impl Session {
    const NAME: &str = "session";
    const LIFETIME: time::Duration = time::Duration::days(30);

    pub(super) fn build<'a>() -> Result<CookieBuilder<'a>, postcard::Error> {
        let expires_at = OffsetDateTime::now_utc() + Self::LIFETIME;
        let cookie = Self::new(expires_at);
        let serialized = postcard::to_allocvec(&cookie)?;

        //TODO this does not need to be valid and printable characters.
        // Just UTF-8 as it is encrypted and base64 encoded again by the private cookie jar
        let encoded = BASE64_URL_SAFE_NO_PAD.encode(serialized);
        Ok(Cookie::build((Self::NAME, encoded)).expires(expires_at))
    }

    fn new(expires_at: OffsetDateTime) -> Self {
        Self { expires_at }
    }

    pub(super) fn is_expired(&self) -> bool {
        OffsetDateTime::now_utc() > self.expires_at
    }
}

#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Bad cookie encoding: {0}")]
    BadCookieEncoding(#[from] base64::DecodeError),
    #[error("Bad cookie: {0}")]
    BadCookieFormat(#[from] postcard::Error),
}

impl TryFrom<Cookie<'_>> for Session {
    type Error = Error;

    fn try_from(cookie: Cookie) -> Result<Self, Self::Error> {
        let encoded = cookie.value();
        let serialized = BASE64_URL_SAFE_NO_PAD.decode(encoded)?;
        let value = postcard::from_bytes(&serialized)?;
        Ok(value)
    }
}
