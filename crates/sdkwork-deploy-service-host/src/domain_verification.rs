use std::time::Duration;

use async_trait::async_trait;
use hickory_resolver::{name_server::TokioConnectionProvider, TokioResolver};
use sdkwork_deploy_contract::{DeployServiceError, DeployServiceResult};
use sdkwork_intelligence_deploy_service::{dns_txt_record_name, DomainOwnershipVerifierPort};

const DOMAIN_VERIFICATION_TIMEOUT: Duration = Duration::from_secs(5);

pub struct DnsTxtDomainOwnershipVerifier {
    resolver: TokioResolver,
}

impl DnsTxtDomainOwnershipVerifier {
    pub fn from_system_config() -> Result<Self, String> {
        let resolver = TokioResolver::builder(TokioConnectionProvider::default())
            .map_err(|error| format!("load system DNS resolver configuration failed: {error}"))?
            .build();
        Ok(Self { resolver })
    }
}

#[async_trait]
impl DomainOwnershipVerifierPort for DnsTxtDomainOwnershipVerifier {
    async fn verify_dns_txt(&self, hostname: &str, token: &str) -> DeployServiceResult<bool> {
        let record_name = dns_txt_record_name(hostname)?;
        let lookup = tokio::time::timeout(
            DOMAIN_VERIFICATION_TIMEOUT,
            self.resolver.txt_lookup(record_name),
        )
        .await
        .map_err(|_| {
            DeployServiceError::Internal("domain ownership DNS lookup timed out".to_owned())
        })?;

        let lookup = match lookup {
            Ok(lookup) => lookup,
            Err(error) if error.is_nx_domain() || error.is_no_records_found() => return Ok(false),
            Err(_) => {
                return Err(DeployServiceError::Internal(
                    "domain ownership DNS lookup failed".to_owned(),
                ))
            }
        };

        Ok(lookup
            .iter()
            .any(|record| txt_chunks_match_token(record.txt_data(), token)))
    }
}

fn txt_chunks_match_token(chunks: &[Box<[u8]>], token: &str) -> bool {
    let expected = token.as_bytes();
    let mut offset = 0usize;
    for chunk in chunks {
        let bytes = chunk.as_ref();
        let Some(end) = offset.checked_add(bytes.len()) else {
            return false;
        };
        if end > expected.len() || expected[offset..end] != *bytes {
            return false;
        }
        offset = end;
    }
    offset == expected.len()
}

#[cfg(test)]
mod tests {
    use super::txt_chunks_match_token;

    #[test]
    fn matches_one_dns_txt_record_across_chunks_only_when_exact() {
        let chunks = vec![
            b"token-".to_vec().into_boxed_slice(),
            b"123".to_vec().into_boxed_slice(),
        ];
        assert!(txt_chunks_match_token(&chunks, "token-123"));
        assert!(!txt_chunks_match_token(&chunks, "token-1234"));
        assert!(!txt_chunks_match_token(&chunks, "prefix-token-123"));
    }
}
