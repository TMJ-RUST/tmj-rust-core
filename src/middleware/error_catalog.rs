use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use axum::{
    body::{Body, Bytes, HttpBody},
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use http_body::Frame;
use uuid::Uuid;

use crate::{
    dto::ApiResponse,
    parameters::{default_error_message, ParameterCache},
};

type ErrorPayload = Arc<Mutex<Option<ApiResponse<()>>>>;

#[derive(Clone)]
struct CatalogError {
    status_code: &'static str,
    payload: ErrorPayload,
}

/// Resolve only marked application errors. Framework rejections retain their
/// existing contract; successful responses never perform a catalogue lookup.
pub async fn resolve_error_message(
    State(parameters): State<ParameterCache>,
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    if let Some(error) = response.extensions_mut().remove::<CatalogError>() {
        let message = parameters.message_error(error.status_code).await;
        if let Some(payload) = error
            .payload
            .lock()
            .expect("error payload mutex poisoned")
            .as_mut()
        {
            payload.message_error = Some(message);
        }
    }
    response
}

pub(crate) fn response(
    status: StatusCode,
    status_code: &'static str,
    message: &'static str,
) -> Response {
    let reference = Uuid::new_v4().to_string();
    let payload = Arc::new(Mutex::new(Some(ApiResponse::error(
        status_code,
        reference.clone(),
        message,
        default_error_message(status_code),
    ))));
    let mut response = Response::new(Body::new(ErrorBody {
        payload: payload.clone(),
    }));
    *response.status_mut() = status;
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    response.headers_mut().insert(
        "x-request-id",
        reference.parse().expect("UUID is a valid header"),
    );
    response.extensions_mut().insert(CatalogError {
        status_code,
        payload,
    });
    response
}

// IntoResponse is synchronous. Defer serialization until after the async
// middleware has resolved the message, rather than parsing/rebuilding JSON.
// Without the middleware, the same body emits its constant default normally.
struct ErrorBody {
    payload: ErrorPayload,
}

impl HttpBody for ErrorBody {
    type Data = Bytes;
    type Error = serde_json::Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let payload = self
            .payload
            .lock()
            .expect("error payload mutex poisoned")
            .take();
        Poll::Ready(payload.map(|payload| {
            serde_json::to_vec(&payload).map(|bytes| Frame::data(Bytes::from(bytes)))
        }))
    }

    fn is_end_stream(&self) -> bool {
        self.payload
            .lock()
            .expect("error payload mutex poisoned")
            .is_none()
    }
}
