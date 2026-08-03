//! PAGINATION_SPEC-aligned query validation for Deploy list operations.
//!
//! Rejects malformed or non-canonical pagination query parameters before
//! handlers run (PAGINATION_SPEC §10.1): `page_size` beyond the declared
//! maximum is rejected with 400 instead of being silently clamped, aliases
//! (`pageSize`/`limit`/...) are refused, and `cursor` fails closed until the
//! keyset upgrade lands per endpoint.

use axum::{
    extract::Request,
    middleware::Next,
    response::{IntoResponse, Response},
};
use sdkwork_web_core::{problem_response, ProblemCorrelation};

use crate::correlation::DeployProblemCorrelation;
use crate::ApiProblem;

const MAXIMUM_PAGE_SIZE: i64 = 200;

/// Reject malformed or non-canonical pagination query parameters before handlers run.
pub async fn validate_pagination_query(request: Request, next: Next) -> Response {
    if let Err(detail) = validate_query(request.uri().query()) {
        let problem = ApiProblem::bad_request(detail);
        let (request_id, trace_id) = match DeployProblemCorrelation::current() {
            Some(value) => (Some(value.request_id), value.trace_id),
            None => (None, None),
        };
        let correlation =
            ProblemCorrelation::new(request_id.as_deref(), trace_id.as_deref());
        return problem_response(&problem.framework_error(), correlation).into_response();
    }
    next.run(request).await
}

fn validate_query(query: Option<&str>) -> Result<(), String> {
    let Some(query) = query else {
        return Ok(());
    };
    let mut page: Option<String> = None;
    let mut page_size: Option<String> = None;
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        match key.as_ref() {
            "page" => {
                if page.replace(value.into_owned()).is_some() {
                    return Err("page must be specified at most once".to_string());
                }
                let parsed = page
                    .as_deref()
                    .unwrap_or_default()
                    .parse::<i64>()
                    .map_err(|_| {
                        "page must be an integer greater than or equal to 1".to_string()
                    })?;
                if parsed < 1 {
                    return Err("page must be greater than or equal to 1".to_string());
                }
            }
            "page_size" => {
                if page_size.replace(value.into_owned()).is_some() {
                    return Err("page_size must be specified at most once".to_string());
                }
                let parsed = page_size
                    .as_deref()
                    .unwrap_or_default()
                    .parse::<i64>()
                    .map_err(|_| "page_size must be an integer between 1 and 200".to_string())?;
                if !(1..=MAXIMUM_PAGE_SIZE).contains(&parsed) {
                    return Err("page_size must be between 1 and 200".to_string());
                }
            }
            "cursor" => {
                // Deploy list endpoints do not support keyset pagination yet;
                // cursor fails closed instead of being silently ignored.
                return Err("cursor pagination is not supported by this endpoint".to_string());
            }
            "pageSize" | "limit" | "page_no" | "pageNo" | "per_page" | "size" => {
                return Err(format!(
                    "{key} is not a supported pagination parameter; use page_size"
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_query;

    #[test]
    fn accepts_canonical_values_and_rejects_aliases() {
        assert!(validate_query(Some("page=2&page_size=20")).is_ok());
        assert!(validate_query(None).is_ok());
        assert!(validate_query(Some("pageSize=20")).is_err());
        assert!(validate_query(Some("limit=20")).is_err());
        assert!(validate_query(Some("page_no=1")).is_err());
        assert!(validate_query(Some("per_page=20")).is_err());
        assert!(validate_query(Some("size=20")).is_err());
        assert!(validate_query(Some("page_size=201")).is_err());
        assert!(validate_query(Some("page_size=0")).is_err());
        assert!(validate_query(Some("page_size=-5")).is_err());
        assert!(validate_query(Some("page=0")).is_err());
        assert!(validate_query(Some("page=-1")).is_err());
        assert!(validate_query(Some("page=1&page=2")).is_err());
        assert!(validate_query(Some("page_size=20&page_size=30")).is_err());
        assert!(validate_query(Some("page=abc")).is_err());
        assert!(validate_query(Some("cursor=opaque-token")).is_err());
        assert!(validate_query(Some("page=1&cursor=x")).is_err());
        // 非分页参数不受影响
        assert!(validate_query(Some("page=1&page_size=20&status=1")).is_ok());
    }
}
