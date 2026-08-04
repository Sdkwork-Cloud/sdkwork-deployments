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

/// Path patterns whose list operations declare cursor (keyset) pagination in
/// their OpenAPI contract. `cursor` on any other endpoint fails closed.
const CURSOR_PAGINATED_PATH_PATTERNS: [&str; 2] = [
    "/backend/v3/api/audit_logs",
    "/app/v3/api/sites/{siteId}/deployments",
];

/// Reject malformed or non-canonical pagination query parameters before handlers run.
pub async fn validate_pagination_query(request: Request, next: Next) -> Response {
    if let Err(detail) = validate_query(request.uri().query(), request.uri().path()) {
        let problem = ApiProblem::bad_request(detail);
        let (request_id, trace_id) = match DeployProblemCorrelation::current() {
            Some(value) => (Some(value.request_id), value.trace_id),
            None => (None, None),
        };
        let correlation = ProblemCorrelation::new(request_id.as_deref(), trace_id.as_deref());
        return problem_response(&problem.framework_error(), correlation).into_response();
    }
    next.run(request).await
}

fn validate_query(query: Option<&str>, path: &str) -> Result<(), String> {
    let Some(query) = query else {
        return Ok(());
    };
    let mut page: Option<String> = None;
    let mut page_size: Option<String> = None;
    let mut cursor = false;
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
                if cursor {
                    return Err("cursor must be specified at most once".to_string());
                }
                cursor = true;
                let value = value.into_owned();
                if value.is_empty() || value.len() > 512 {
                    return Err("cursor must contain 1..512 bytes".to_string());
                }
            }
            "pageSize" | "limit" | "page_no" | "pageNo" | "per_page" | "size" => {
                return Err(format!(
                    "{key} is not a supported pagination parameter; use page_size"
                ));
            }
            _ => {}
        }
    }
    if cursor && page.is_some() {
        return Err("page and cursor cannot be combined".to_string());
    }
    if cursor && !path_matches_cursor_patterns(path) {
        return Err("cursor pagination is not supported by this endpoint".to_string());
    }
    Ok(())
}

/// Matches the request path against the cursor-paginated operation patterns.
fn path_matches_cursor_patterns(path: &str) -> bool {
    CURSOR_PAGINATED_PATH_PATTERNS.iter().any(|pattern| {
        let segments = pattern.split('/').collect::<Vec<_>>();
        let path_segments = path.split('/').collect::<Vec<_>>();
        segments.len() == path_segments.len()
            && segments
                .iter()
                .zip(path_segments.iter())
                .all(|(pattern, actual)| {
                    pattern.starts_with('{') && pattern.ends_with('}') || pattern == actual
                })
    })
}

#[cfg(test)]
mod tests {
    use super::validate_query;

    #[test]
    fn accepts_canonical_values_and_rejects_aliases() {
        assert!(validate_query(Some("page=2&page_size=20"), "/backend/v3/api/audit_logs").is_ok());
        assert!(validate_query(None, "/backend/v3/api/sites").is_ok());
        assert!(validate_query(Some("pageSize=20"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("limit=20"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("page_no=1"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("per_page=20"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("size=20"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("page_size=201"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("page_size=0"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("page_size=-5"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("page=0"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("page=-1"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(Some("page=1&page=2"), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(
            Some("page_size=20&page_size=30"),
            "/backend/v3/api/audit_logs"
        )
        .is_err());
        assert!(validate_query(Some("page=abc"), "/backend/v3/api/audit_logs").is_err());
        // 非分页参数不受影响
        assert!(validate_query(Some("page=1&page_size=20&status=1"), "/app/v3/api/sites").is_ok());
    }

    #[test]
    fn cursor_is_whitelisted_only_on_keyset_endpoints() {
        assert!(validate_query(Some("cursor=opaque-token"), "/backend/v3/api/audit_logs").is_ok());
        assert!(validate_query(
            Some("cursor=opaque-token"),
            "/app/v3/api/sites/site-1/deployments"
        )
        .is_ok());
        assert!(validate_query(Some("cursor="), "/backend/v3/api/audit_logs").is_err());
        assert!(validate_query(
            Some("cursor=opaque-token"),
            "/app/v3/api/sites/site-1/releases"
        )
        .is_err());
        assert!(validate_query(Some("cursor=x&page=1"), "/backend/v3/api/audit_logs").is_err());
    }
}
