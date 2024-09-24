use axum::{
    async_trait,
    body::Bytes,
    extract::{rejection::BytesRejection, FromRequest, Request},
    http::StatusCode,
    response::IntoResponse,
};
use thiserror::Error;

pub(crate) struct TlsCodec<T: tls_codec::DeserializeBytes>(pub(crate) T);

#[derive(Error, Debug)]
pub(crate) enum TlsCodecRejection {
    #[error("Error reading bytes from body")]
    BytesError(#[from] BytesRejection),
    #[error("Error deserializing TLS codec body")]
    DeserializationError(#[from] tls_codec::Error),
}

impl IntoResponse for TlsCodecRejection {
    fn into_response(self) -> axum::response::Response {
        match self {
            TlsCodecRejection::BytesError(error) => error.into_response(),
            TlsCodecRejection::DeserializationError(error) => {
                tracing::trace!("Error during TLS codec deserialization: {:?}", error);
                StatusCode::BAD_REQUEST.into_response()
            }
        }
    }
}

#[async_trait]
impl<S, T> FromRequest<S> for TlsCodec<T>
where
    S: Send + Sync,
    T: tls_codec::DeserializeBytes,
{
    type Rejection = TlsCodecRejection;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let bytes = Bytes::from_request(request, state).await?;
        let value = T::tls_deserialize_exact_bytes(&bytes)?;
        Ok(Self(value))
    }
}
