use axum::body::to_bytes;
use sdkwork_deploy_contract::DeployServiceError;
use sdkwork_routes_deploy_common::response::{finish_api_json, ok_json, ApiProblem};
use sdkwork_utils_rust::SdkWorkResourceData;
use sdkwork_web_core::{
    ServerRequestId, WebApiSurface, WebAuthMode, WebRequestContext, WebTransportFacts,
};

fn test_context() -> WebRequestContext {
    WebRequestContext {
        request_id: ServerRequestId("test-req".to_owned()),
        api_surface: WebApiSurface::AppApi,
        auth_mode: WebAuthMode::DualToken,
        principal: None,
        transport: WebTransportFacts {
            path: "/app/v3/api/sites/site-1".to_owned(),
            method: "GET".to_owned(),
            auth_token_present: true,
            access_token_present: true,
            api_key_present: false,
            oauth_bearer_present: false,
            agent_token_present: false,
        },
        locale: None,
        client_kind: None,
        operation: None,
        trace_id: Some("trace-deploy-test".to_owned()),
    }
}

#[test]
fn service_error_maps_to_numeric_problem_code() {
    let ctx = test_context();
    let response = finish_api_json(
        &ctx,
        Err::<SdkWorkResourceData<String>, _>(DeployServiceError::not_found("site missing").into()),
    );
    let body = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { to_bytes(response.into_body(), usize::MAX).await.unwrap() });
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(404, payload["status"].as_u64().unwrap());
    assert_eq!(40401, payload["code"].as_i64().unwrap());
    assert_eq!("trace-deploy-test", payload["traceId"].as_str().unwrap());
}

#[test]
fn success_envelope_includes_code_data_trace_id() {
    let ctx = test_context();
    let response = finish_api_json(
        &ctx,
        ok_json(SdkWorkResourceData {
            item: "demo".to_string(),
        }),
    );
    let body = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { to_bytes(response.into_body(), usize::MAX).await.unwrap() });
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(0, payload["code"].as_i64().unwrap());
    assert_eq!("trace-deploy-test", payload["traceId"].as_str().unwrap());
    assert_eq!("demo", payload["data"]["item"].as_str().unwrap());
}

#[test]
fn api_problem_uses_problem_json_semantics() {
    let ctx = test_context();
    let response = ApiProblem::forbidden("tenant access denied").into_response_for(&ctx);
    let body = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async { to_bytes(response.into_body(), usize::MAX).await.unwrap() });
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(40301, payload["code"].as_i64().unwrap());
    assert!(payload.get("requestId").is_none());
}
