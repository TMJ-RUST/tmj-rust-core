use axum::{
    extract::FromRequestParts,
    http::{header::HeaderName, request::Parts},
};
use uuid::Uuid;

pub static X_REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");
pub static X_CORRELATION_ID: HeaderName = HeaderName::from_static("x-correlation-id");

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

impl<S> FromRequestParts<S> for RequestId
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let req_id = parts
            .headers
            .get(&X_REQUEST_ID)
            .or_else(|| parts.headers.get(&X_CORRELATION_ID))
            .and_then(|val| val.to_str().ok())
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string());

        Ok(RequestId(req_id))
    }
}
