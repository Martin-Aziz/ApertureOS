use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use http_body_util::BodyExt;
use pixelforge_backend::config::AppConfig;
use pixelforge_backend::{build_app, AppState};
use serde_json::{json, Value};
use tower::ServiceExt;

fn build_test_config() -> AppConfig {
    AppConfig {
        bind_addr: "127.0.0.1:8080".parse().expect("valid socket"),
        frontend_origin: "http://localhost:5173".to_owned(),
        database_url: "postgresql://aperture:aperture@localhost:5432/aperture".to_owned(),
        redis_url: "redis://localhost:6379".to_owned(),
        jwt_access_secret: "test-access-secret-which-is-long-enough-123".to_owned(),
        jwt_refresh_secret: "test-refresh-secret-which-is-long-enough-123".to_owned(),
        jwt_access_ttl_seconds: 900,
        jwt_refresh_ttl_seconds: 60 * 60,
        demo_admin_email: "admin@pixelforge.local".to_owned(),
        demo_admin_password: "ChangeMe123!".to_owned(),
        ai_shared_secret: "test-shared-secret-for-ai-service".to_owned(),
        ai_service_base_url: "http://127.0.0.1:18001".to_owned(),
        ai_request_timeout_seconds: 1,
        ai_max_image_bytes: 16,
    }
}

async fn parse_json(response: axum::response::Response) -> Value {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body should be readable")
        .to_bytes();

    serde_json::from_slice(&bytes).expect("body should be valid json")
}

async fn parse_text(response: axum::response::Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("response body should be readable")
        .to_bytes();

    String::from_utf8(bytes.to_vec()).expect("body should be valid utf-8")
}

#[tokio::test]
async fn health_live_should_return_ok() {
    let app = build_app(AppState::new(build_test_config()));

    let request = Request::builder()
        .method(Method::GET)
        .uri("/health/live")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .oneshot(request)
        .await
        .expect("health live request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let body = parse_json(response).await;
    assert_eq!(body["status"], "live");
}

#[tokio::test]
async fn metrics_should_return_prometheus_payload() {
    let app = build_app(AppState::new(build_test_config()));

    let request = Request::builder()
        .method(Method::GET)
        .uri("/metrics")
        .body(Body::empty())
        .expect("request should be built");

    let response = app
        .oneshot(request)
        .await
        .expect("metrics request should succeed");

    assert_eq!(response.status(), StatusCode::OK);
    let payload = parse_text(response).await;
    assert!(payload.contains("pixelforge_backend_info"));
    assert!(payload.contains("pixelforge_backend_dependency_configured"));
}

#[tokio::test]
async fn cors_should_allow_loopback_frontend_origin() {
    let app = build_app(AppState::new(build_test_config()));

    let request = Request::builder()
        .method(Method::OPTIONS)
        .uri("/api/v1/ai/remove-background")
        .header("origin", "http://127.0.0.1:5173")
        .header("access-control-request-method", "POST")
        .header("access-control-request-headers", "content-type")
        .body(Body::empty())
        .expect("cors preflight request should build");

    let response = app
        .oneshot(request)
        .await
        .expect("cors preflight request should succeed");

    assert!(response.status().is_success());
    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .and_then(|value| value.to_str().ok())
        .expect("allow origin should be present");

    assert_eq!(allow_origin, "http://127.0.0.1:5173");
}

#[tokio::test]
async fn login_then_project_flow_should_work() {
    let app = build_app(AppState::new(build_test_config()));

    let login_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "email": "admin@pixelforge.local",
                "password": "ChangeMe123!"
            })
            .to_string(),
        ))
        .expect("login request should build");

    let login_response = app
        .clone()
        .oneshot(login_request)
        .await
        .expect("login request should succeed");

    assert_eq!(login_response.status(), StatusCode::OK);
    let login_body = parse_json(login_response).await;
    let access_token = login_body["tokens"]["access_token"]
        .as_str()
        .expect("access token should exist")
        .to_owned();

    let create_project_request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/projects")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::from(
            json!({
                "name": "Landing Page Concepts",
                "description": "Initial campaign iterations"
            })
            .to_string(),
        ))
        .expect("create project request should build");

    let create_response = app
        .clone()
        .oneshot(create_project_request)
        .await
        .expect("create project request should succeed");

    assert_eq!(create_response.status(), StatusCode::OK);
    let create_body = parse_json(create_response).await;
    let project_id = create_body["id"]
        .as_str()
        .expect("project id should be present")
        .to_owned();

    let list_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/projects")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .expect("list projects request should build");

    let list_response = app
        .clone()
        .oneshot(list_request)
        .await
        .expect("list projects should succeed");

    assert_eq!(list_response.status(), StatusCode::OK);
    let list_body = parse_json(list_response).await;
    assert_eq!(list_body.as_array().expect("array expected").len(), 1);

    let delete_request = Request::builder()
        .method(Method::DELETE)
        .uri(format!("/api/v1/projects/{project_id}"))
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .expect("delete request should build");

    let delete_response = app
        .clone()
        .oneshot(delete_request)
        .await
        .expect("delete request should succeed");

    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let list_after_delete_request = Request::builder()
        .method(Method::GET)
        .uri("/api/v1/projects")
        .header("authorization", format!("Bearer {access_token}"))
        .body(Body::empty())
        .expect("list projects request should build");

    let list_after_delete = app
        .oneshot(list_after_delete_request)
        .await
        .expect("list projects should succeed");

    assert_eq!(list_after_delete.status(), StatusCode::OK);
    let after_delete_body = parse_json(list_after_delete).await;
    assert_eq!(after_delete_body.as_array().expect("array expected").len(), 0);
}

#[tokio::test]
async fn local_ai_remove_background_should_return_service_unavailable_when_offline() {
    let app = build_app(AppState::new(build_test_config()));

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/ai/remove-background")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "image_base64": "aW5wdXQ="
            })
            .to_string(),
        ))
        .expect("ai remove-background request should build");

    let response = app
        .oneshot(request)
        .await
        .expect("ai remove-background request should succeed");

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = parse_json(response).await;
    assert_eq!(body["error"]["code"], "service_unavailable");
}

#[tokio::test]
async fn local_ai_remove_background_should_reject_unsupported_output_format() {
    let app = build_app(AppState::new(build_test_config()));

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/ai/remove-background")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "image_base64": "aW5wdXQ=",
                "output_format": "jpeg"
            })
            .to_string(),
        ))
        .expect("ai remove-background request should build");

    let response = app
        .oneshot(request)
        .await
        .expect("ai remove-background request should succeed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = parse_json(response).await;
    assert_eq!(body["error"]["code"], "bad_request");
}

#[tokio::test]
async fn local_ai_remove_background_should_reject_payload_over_limit() {
    let app = build_app(AppState::new(build_test_config()));
    let oversized_payload = BASE64_STANDARD.encode(vec![0_u8; 32]);

    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/v1/ai/remove-background")
        .header("content-type", "application/json")
        .body(Body::from(
            json!({
                "image_base64": oversized_payload
            })
            .to_string(),
        ))
        .expect("ai remove-background request should build");

    let response = app
        .oneshot(request)
        .await
        .expect("ai remove-background request should succeed");

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    let body = parse_json(response).await;
    assert_eq!(body["error"]["code"], "payload_too_large");
}
