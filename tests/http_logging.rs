use std::{
    collections::VecDeque,
    io::{self, Write},
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use axum::{
    body::{to_bytes, Body, Bytes, HttpBody},
    http::{HeaderMap, Request, StatusCode},
    middleware::from_fn,
    response::Response,
    routing::get,
    Router,
};
use http_body::Frame;
use serde_json::{json, Value};
use tmj_rust_core::middleware::http_logging::log_http;
use tower::ServiceExt;
use tracing::instrument::WithSubscriber;

#[derive(Clone, Default)]
struct Logs(Arc<Mutex<Vec<u8>>>);

impl Write for Logs {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Logs {
    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }

    fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync {
        let writer = self.clone();
        tracing_subscriber::fmt()
            .with_env_filter("tmj_http=info")
            .without_time()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish()
    }
}

struct Frames(VecDeque<Result<Frame<Bytes>, io::Error>>);

impl HttpBody for Frames {
    type Data = Bytes;
    type Error = io::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, io::Error>>> {
        Poll::Ready(self.0.pop_front())
    }

    fn is_end_stream(&self) -> bool {
        self.0.is_empty()
    }
}

fn json_response(body: Body) -> Response {
    Response::builder()
        .status(StatusCode::CREATED)
        .header("content-type", "application/json; charset=utf-8")
        .header("set-cookie", "synthetic-session")
        .body(body)
        .unwrap()
}

fn body_from_log(text: &str) -> Value {
    let body = text
        .split("Response Body  : ")
        .nth(1)
        .unwrap()
        .split("\n-->")
        .next()
        .unwrap();
    serde_json::from_str(body).unwrap()
}

#[tokio::test]
async fn redacts_nested_credentials_and_private_headers_without_changing_response() {
    let logs = Logs::default();
    let text = async {
        let router = Router::new().route("/probe", get(|| async {
            let payload = json!({
                "statusCode": "00",
                "data": [{
                    "access_token": "synthetic-access",
                    "refreshToken": "synthetic-refresh",
                    "new-password": "synthetic-password",
                    "nested": {"otp_code": "synthetic-otp", "resetToken": "synthetic-reset"},
                    "variables": {"custom": "synthetic-email-content"},
                    "email": "synthetic@example.invalid",
                    "subject": "synthetic-subject",
                    "bodyHtml": "synthetic-html",
                    "reference": "synthetic-reference",
                    "count": 7
                }]
            });
            json_response(Body::from(payload.to_string()))
        })).layer(from_fn(log_http));
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/probe?token=synthetic-query&synthetic-bare-secret")
                    .header("authorization", "synthetic-auth")
                    .header("x-api-key", "synthetic-key")
                    .header("upstash-signature", "synthetic-signature")
                    .header("x-custom-private", "synthetic-custom")
                    .header("x-request-id", "smoke-request-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.headers()["set-cookie"], "synthetic-session");
        let wire: Value =
            serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
                .unwrap();
        assert_eq!(wire["data"][0]["access_token"], "synthetic-access");
        assert_eq!(wire["data"][0]["nested"]["otp_code"], "synthetic-otp");
        logs.text()
    }
    .with_subscriber(logs.subscriber())
    .await;
    assert_eq!(text.matches("INBOUND >>").count(), 1);
    assert_eq!(text.matches("OUTBOUND <<").count(), 1);
    assert!(text.contains("smoke-request-1"));
    assert!(text.contains("{201}"));
    for secret in [
        "synthetic-access",
        "synthetic-refresh",
        "synthetic-password",
        "synthetic-otp",
        "synthetic-reset",
        "synthetic-email-content",
        "synthetic@example.invalid",
        "synthetic-subject",
        "synthetic-html",
        "synthetic-reference",
        "synthetic-query",
        "synthetic-bare-secret",
        "synthetic-auth",
        "synthetic-key",
        "synthetic-signature",
        "synthetic-custom",
        "synthetic-session",
    ] {
        assert!(!text.contains(secret), "leaked {secret}");
    }
    let logged = body_from_log(&text);
    assert_eq!(logged["statusCode"], "00");
    assert_eq!(logged["data"][0]["count"], 7);
    assert_eq!(logged["data"][0]["nested"]["otp_code"], "[REDACTED]");
}

#[tokio::test]
async fn captures_split_utf8_json_and_preserves_trailers() {
    let logs = Logs::default();
    async {
        let router = Router::new()
            .route(
                "/probe",
                get(|| async {
                    let data =
                        "{\"message\":\"café\",\"resetToken\":\"synthetic-reset\"}".as_bytes();
                    let split = data.iter().position(|byte| *byte == 0xc3).unwrap() + 1;
                    let mut trailers = HeaderMap::new();
                    trailers.insert("x-stream-result", "complete".parse().unwrap());
                    json_response(Body::new(Frames(VecDeque::from([
                        Ok(Frame::data(Bytes::copy_from_slice(&data[..split]))),
                        Ok(Frame::data(Bytes::copy_from_slice(&data[split..]))),
                        Ok(Frame::trailers(trailers)),
                    ]))))
                }),
            )
            .layer(from_fn(log_http));
        let mut response = router
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let mut wire = Vec::new();
        let mut trailer = None;
        while let Some(frame) =
            std::future::poll_fn(|cx| Pin::new(response.body_mut()).poll_frame(cx)).await
        {
            let frame = frame.unwrap();
            if let Some(data) = frame.data_ref() {
                wire.extend_from_slice(data);
            }
            if let Some(headers) = frame.trailers_ref() {
                trailer = Some(headers.clone());
            }
        }
        let wire: Value = serde_json::from_slice(&wire).unwrap();
        assert_eq!(wire["resetToken"], "synthetic-reset");
        assert_eq!(trailer.unwrap()["x-stream-result"], "complete");
    }
    .with_subscriber(logs.subscriber())
    .await;
    assert_eq!(body_from_log(&logs.text())["message"], "café");
    assert!(!logs.text().contains("synthetic-reset"));
    assert_eq!(logs.text().matches("OUTBOUND <<").count(), 1);
}

#[tokio::test]
async fn enforces_body_limit_at_boundary_for_unknown_length_streams() {
    for size in [16384, 16385] {
        let logs = Logs::default();
        async {
            let router = Router::new()
                .route(
                    "/probe",
                    get(move || async move {
                        let payload = format!("{{\"padding\":\"{}\"}}", "x".repeat(size - 14));
                        assert_eq!(payload.len(), size);
                        json_response(Body::new(Frames(VecDeque::from([Ok(Frame::data(
                            Bytes::from(payload),
                        ))]))))
                    }),
                )
                .layer(from_fn(log_http));
            let response = router
                .oneshot(
                    Request::builder()
                        .uri("/probe")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            let wire = to_bytes(response.into_body(), usize::MAX).await.unwrap();
            assert_eq!(wire.len(), size);
        }
        .with_subscriber(logs.subscriber())
        .await;
        let text = logs.text();
        assert_eq!(text.matches("OUTBOUND <<").count(), 1);
        if size == 16384 {
            assert_eq!(
                body_from_log(&text)["padding"].as_str().unwrap().len(),
                size - 14
            );
        } else {
            assert!(text.contains("omitted"));
            assert!(!text.contains(&"x".repeat(20)));
        }
    }
}

#[tokio::test]
async fn never_logs_partial_or_malformed_json_and_preserves_body_errors() {
    for mode in ["error", "drop", "invalid"] {
        let logs = Logs::default();
        async {
            let router = Router::new()
                .route(
                    "/probe",
                    get(move || async move {
                        let first = Ok(Frame::data(Bytes::from_static(
                            b"{\"access_token\":\"synthetic-partial",
                        )));
                        let frames = if mode == "error" {
                            VecDeque::from([first, Err(io::Error::other("synthetic-stream-error"))])
                        } else {
                            VecDeque::from([first, Ok(Frame::data(Bytes::from_static(b"-tail")))])
                        };
                        json_response(Body::new(Frames(frames)))
                    }),
                )
                .layer(from_fn(log_http));
            let mut response = router
                .oneshot(
                    Request::builder()
                        .uri("/probe")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            if mode == "drop" {
                std::future::poll_fn(|cx| Pin::new(response.body_mut()).poll_frame(cx))
                    .await
                    .unwrap()
                    .unwrap();
                drop(response);
            } else {
                let result = to_bytes(response.into_body(), usize::MAX).await;
                assert_eq!(result.is_err(), mode == "error");
            }
        }
        .with_subscriber(logs.subscriber())
        .await;
        let text = logs.text();
        assert_eq!(text.matches("OUTBOUND <<").count(), 1);
        assert!(text.contains("omitted"));
        assert!(!text.contains("synthetic-partial"));
        assert!(!text.contains("synthetic-stream-error"));
    }
}
