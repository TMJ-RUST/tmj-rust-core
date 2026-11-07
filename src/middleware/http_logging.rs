use std::{
    fmt::Write,
    pin::Pin,
    task::{Context, Poll},
    time::Instant,
};

use axum::{
    body::{Body, Bytes, HttpBody},
    extract::Request,
    http::{HeaderMap, Uri},
    middleware::Next,
    response::Response,
};
use http_body::{Frame, SizeHint};
use serde_json::Value;
use uuid::Uuid;

const BODY_LIMIT: usize = 16 * 1024;
const REDACTED: &str = "[REDACTED]";

/// Emit one redacted INBOUND/OUTBOUND event without consuming request bodies or
/// delaying response frames. Enable with `RUST_LOG=tmj_http=info`.
pub async fn log_http(request: Request, next: Next) -> Response {
    if !tracing::enabled!(target: "tmj_http", tracing::Level::INFO) {
        return next.run(request).await;
    }

    let started = Instant::now();
    let mut text = String::with_capacity(1024);
    let request_id = ["x-request-id", "x-correlation-id"]
        .into_iter()
        .filter_map(|name| request.headers().get(name)?.to_str().ok())
        .find(|value| {
            !value.is_empty()
                && value.len() <= 128
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || b"-_.".contains(&byte))
        });
    writeln!(
        text,
        "========================================================================\nINBOUND >>"
    )
    .unwrap();
    match request_id {
        Some(id) => writeln!(text, "Request ID     : {{{id}}}").unwrap(),
        None => writeln!(text, "Request ID     : {{{}}}", Uuid::new_v4()).unwrap(),
    }
    write!(text, "URI            : {{").unwrap();
    write_uri(&mut text, request.uri());
    writeln!(text, "}}\nMethod         : {{{}}}", request.method()).unwrap();
    write!(text, "Headers        : ").unwrap();
    write_headers(&mut text, request.headers());

    let response = next.run(request).await;
    writeln!(
        text,
        "\n\nOUTBOUND <<\nStatus code    : {{{}}}",
        response.status().as_u16()
    )
    .unwrap();
    write!(text, "Headers        : ").unwrap();
    write_headers(&mut text, response.headers());
    write!(text, "\nResponse Body  : ").unwrap();
    let mut exchange = Exchange { text, started };

    // Unsupported/encoded/large bodies are never polled just for logging.
    let omitted = if response.body().is_end_stream() {
        Some("[empty]")
    } else if response
        .headers()
        .get_all("content-encoding")
        .iter()
        .any(|value| value != "identity")
    {
        Some("[omitted: encoded body]")
    } else if !is_json(response.headers()) {
        Some("[omitted: non-JSON or streaming body]")
    } else if response.body().size_hint().lower() > BODY_LIMIT as u64 {
        Some("[omitted: body exceeds 16384 bytes]")
    } else {
        None
    };
    if let Some(reason) = omitted {
        exchange.emit(reason);
        return response;
    }

    let (parts, body) = response.into_parts();
    Response::from_parts(
        parts,
        Body::new(LoggingBody {
            inner: body,
            exchange: Some(exchange),
            captured: Vec::new(),
            oversized: false,
        }),
    )
}

fn write_uri(text: &mut String, uri: &Uri) {
    text.push_str(uri.path());
    if let Some(query) = uri.query() {
        text.push('?');
        for (index, parameter) in query.split('&').enumerate() {
            if index != 0 {
                text.push('&');
            }
            // Bare parameters can themselves be secrets; only named keys survive.
            if let Some((name, _)) = parameter.split_once('=') {
                write!(text, "{name:?}={REDACTED}").unwrap();
            } else {
                text.push_str(REDACTED);
            }
        }
    }
}

fn write_headers(text: &mut String, headers: &HeaderMap) {
    text.push('{');
    for (name, value) in headers {
        write!(text, "{{{name}=[").unwrap();
        // Unknown headers are private by default, including future auth schemes.
        if !value.is_sensitive()
            && matches!(
                name.as_str(),
                "accept"
                    | "accept-encoding"
                    | "accept-language"
                    | "content-type"
                    | "content-length"
                    | "content-encoding"
                    | "host"
                    | "connection"
                    | "user-agent"
                    | "traceparent"
                    | "vary"
                    | "x-request-id"
                    | "x-correlation-id"
                    | "access-control-allow-origin"
                    | "access-control-allow-methods"
                    | "access-control-allow-headers"
                    | "access-control-expose-headers"
                    | "access-control-max-age"
            )
        {
            write!(text, "{value:?}").unwrap();
        } else {
            text.push_str(REDACTED);
        }
        text.push_str("]}");
    }
    text.push('}');
}

fn is_json(headers: &HeaderMap) -> bool {
    headers
        .get("content-type")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            let media_type = value.split(';').next().unwrap_or_default().trim();
            media_type.eq_ignore_ascii_case("application/json")
                || media_type.split_once('/').is_some_and(|(kind, subtype)| {
                    kind.eq_ignore_ascii_case("application")
                        && subtype
                            .get(subtype.len().saturating_sub(5)..)
                            .is_some_and(|suffix| suffix.eq_ignore_ascii_case("+json"))
                })
        })
}

fn private_field(key: &str) -> bool {
    // Normalize camelCase/snake_case/kebab-case without allocating per field.
    let mut normalized = [0_u8; 128];
    let mut length = 0;
    for byte in key.bytes().filter(|byte| !matches!(byte, b'_' | b'-')) {
        if length == normalized.len() || !byte.is_ascii() {
            return true;
        }
        normalized[length] = byte.to_ascii_lowercase();
        length += 1;
    }
    let key = std::str::from_utf8(&normalized[..length]).unwrap();
    [
        "password",
        "token",
        "secret",
        "otp",
        "signature",
        "apikey",
        "authorization",
        "cookie",
    ]
    .iter()
    .any(|sensitive| key.contains(sensitive))
        || matches!(
            key,
            "reference"
                | "variables"
                | "subject"
                | "body"
                | "bodyhtml"
                | "bodytext"
                | "errormessage"
                | "messageerror"
                | "email"
                | "username"
                | "identifier"
                | "recipient"
                | "cc"
                | "bcc"
                | "replyto"
                | "destinationurl"
        )
}

fn redact_json(value: &mut Value) {
    match value {
        Value::Object(fields) => {
            for (key, value) in fields {
                if private_field(key) {
                    *value = Value::String(REDACTED.into());
                } else {
                    redact_json(value);
                }
            }
        }
        Value::Array(values) => values.iter_mut().for_each(redact_json),
        _ => {}
    }
}

struct Exchange {
    text: String,
    started: Instant,
}

impl Exchange {
    fn emit(&mut self, body: &str) {
        self.text.push_str(body);
        write!(self.text, "\n-->{}ms", self.started.elapsed().as_millis()).unwrap();
        tracing::info!(target: "tmj_http", "{}", self.text);
    }

    fn complete(mut self, captured: &[u8]) {
        if captured.is_empty() {
            self.emit("[empty]");
        } else if let Ok(mut value) = serde_json::from_slice::<Value>(captured) {
            redact_json(&mut value);
            write!(self.text, "{value}").unwrap();
            self.emit("");
        } else {
            // Never log a raw prefix: malformed/truncated JSON may contain secrets.
            self.emit("[omitted: invalid JSON]");
        }
    }
}

struct LoggingBody {
    inner: Body,
    exchange: Option<Exchange>,
    captured: Vec<u8>,
    oversized: bool,
}

impl LoggingBody {
    fn finish(&mut self, reason: &str) {
        if let Some(mut exchange) = self.exchange.take() {
            exchange.emit(reason);
        }
    }
}

impl HttpBody for LoggingBody {
    type Data = Bytes;
    type Error = axum::Error;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        context: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Self::Error>>> {
        let frame = Pin::new(&mut self.inner).poll_frame(context);
        match &frame {
            Poll::Ready(Some(Ok(frame))) => {
                if let Some(data) = frame.data_ref() {
                    if !self.oversized && data.len() <= BODY_LIMIT - self.captured.len() {
                        self.captured.extend_from_slice(data);
                    } else if !self.oversized {
                        self.oversized = true;
                        self.captured = Vec::new();
                        self.finish("[omitted: body exceeds 16384 bytes]");
                    }
                }
                if self.inner.is_end_stream() {
                    if let Some(exchange) = self.exchange.take() {
                        exchange.complete(&self.captured);
                    }
                }
            }
            Poll::Ready(None) => {
                if let Some(exchange) = self.exchange.take() {
                    exchange.complete(&self.captured);
                }
            }
            Poll::Ready(Some(Err(_))) => self.finish("[omitted: body stream error]"),
            Poll::Pending => {}
        }
        frame
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> SizeHint {
        self.inner.size_hint()
    }
}

impl Drop for LoggingBody {
    fn drop(&mut self) {
        self.finish("[omitted: body not fully consumed]");
    }
}
