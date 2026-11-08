use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use serde_json::Value;
use tmj_rust_core::{
    error::{catalog_error_response, AppError},
    middleware::error_catalog::resolve_error_message,
    parameters::{
        default_error_message, NumericParameter, ParameterCache, ERROR_CODE_CACHE_KEY,
        SYSTEM_PARAMETER_CACHE_KEY,
    },
};
use tower::ServiceExt;

async fn body(response: Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap()
}

#[tokio::test]
async fn standalone_errors_preserve_status_and_reference_with_per_code_defaults() {
    let invalid = AppError::InvalidToken.into_response();
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
    let reference = invalid.headers()["x-request-id"]
        .to_str()
        .unwrap()
        .to_owned();
    let invalid = body(invalid).await;
    let expired = body(AppError::TokenExpired.into_response()).await;
    assert_eq!(invalid["statusCode"], "02");
    assert_eq!(invalid["reffCode"], reference);
    assert!(uuid::Uuid::parse_str(&reference).is_ok());
    assert_eq!(invalid["messageError"], expired["messageError"]);
    assert!(invalid["data"].is_null());

    let validation =
        body(AppError::ValidationError("synthetic-private-detail".into()).into_response()).await;
    assert_eq!(validation["statusCode"], "03");
    assert!(!validation.to_string().contains("synthetic-private-detail"));
    let unknown = body(catalog_error_response(
        StatusCode::BAD_REQUEST,
        "custom",
        "Custom Error",
    ))
    .await;
    assert_eq!(unknown["messageError"], default_error_message("custom"));
}

#[tokio::test]
#[ignore = "requires isolated Redis at TMJ_PARAMETER_TEST_REDIS_URL"]
async fn redis_catalog_reads_each_error_and_numeric_group_falls_back_per_field() {
    let url =
        std::env::var("TMJ_PARAMETER_TEST_REDIS_URL").expect("isolated test Redis URL required");
    let mut redis = tmj_rust_core::config::init_redis(&url).await.unwrap();
    redis::cmd("DEL")
        .arg(SYSTEM_PARAMETER_CACHE_KEY)
        .arg(ERROR_CODE_CACHE_KEY)
        .query_async::<()>(&mut redis)
        .await
        .unwrap();
    let parameters = ParameterCache::new(redis.clone());
    let specs = [
        NumericParameter {
            key: "TEST_A",
            default: 10,
            min: 1,
            max: 100,
        },
        NumericParameter {
            key: "TEST_B",
            default: 20,
            min: 1,
            max: 100,
        },
        NumericParameter {
            key: "TEST_C",
            default: 30,
            min: 1,
            max: 100,
        },
    ];
    redis::cmd("HSET")
        .arg(SYSTEM_PARAMETER_CACHE_KEY)
        .arg("TEST_A")
        .arg("12")
        .arg("TEST_B")
        .arg("invalid")
        .query_async::<()>(&mut redis)
        .await
        .unwrap();
    assert_eq!(parameters.numbers(&specs).await, [12, 20, 30]);
    redis::cmd("HSET")
        .arg(SYSTEM_PARAMETER_CACHE_KEY)
        .arg("TEST_A")
        .arg("0")
        .query_async::<()>(&mut redis)
        .await
        .unwrap();
    assert_eq!(parameters.numbers(&specs).await, [10, 20, 30]);

    let router = Router::new()
        .route(
            "/error",
            get(|| async { Err::<(), _>(AppError::InvalidCredentials) }),
        )
        .layer(from_fn_with_state(parameters, resolve_error_message));
    for message in ["synthetic-first-message", "synthetic-next-message"] {
        redis::cmd("HSET")
            .arg(ERROR_CODE_CACHE_KEY)
            .arg("01")
            .arg(message)
            .query_async::<()>(&mut redis)
            .await
            .unwrap();
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/error")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let payload = body(response).await;
        assert_eq!(payload["statusCode"], "01");
        assert_eq!(payload["message"], "Authentication Failed");
        assert_eq!(payload["messageError"], message);
    }
    redis::cmd("HDEL")
        .arg(ERROR_CODE_CACHE_KEY)
        .arg("01")
        .query_async::<()>(&mut redis)
        .await
        .unwrap();
    let response = router
        .oneshot(
            Request::builder()
                .uri("/error")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        body(response).await["messageError"],
        default_error_message("01")
    );
    redis::cmd("DEL")
        .arg(SYSTEM_PARAMETER_CACHE_KEY)
        .arg(ERROR_CODE_CACHE_KEY)
        .query_async::<()>(&mut redis)
        .await
        .unwrap();
}
