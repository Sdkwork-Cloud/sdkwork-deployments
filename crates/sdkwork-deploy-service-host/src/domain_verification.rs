use std::time::Duration;

use async_trait::async_trait;
use hickory_resolver::{name_server::TokioConnectionProvider, TokioResolver};
use sdkwork_deploy_contract::{DeployServiceError, DeployServiceResult};
use sdkwork_intelligence_deploy_service::{
    dns_txt_record_name, DomainOwnershipVerifierPort, DomainVerificationObservation,
};
use sdkwork_utils_rust::crypto::{secure_compare, sha256_hash};

const DOMAIN_VERIFICATION_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_VERIFICATION_RECORD_BYTES: usize = 512;

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
    async fn verify_dns_txt(
        &self,
        hostname: &str,
        expected_sha256: &str,
    ) -> DeployServiceResult<DomainVerificationObservation> {
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
            Err(error) if error.is_nx_domain() || error.is_no_records_found() => {
                return Ok(DomainVerificationObservation {
                    matched: false,
                    observed_sha256: None,
                    verifier_identity: "system-dns".to_owned(),
                })
            }
            Err(_) => {
                return Err(DeployServiceError::Internal(
                    "domain ownership DNS lookup failed".to_owned(),
                ))
            }
        };

        let observed_sha256 = lookup.iter().find_map(|record| {
            txt_chunks_sha256(record.txt_data())
                .filter(|observed| secure_compare(observed.as_str(), expected_sha256))
        });
        Ok(DomainVerificationObservation {
            matched: observed_sha256.is_some(),
            observed_sha256,
            verifier_identity: "system-dns".to_owned(),
        })
    }
}

fn txt_chunks_sha256(chunks: &[Box<[u8]>]) -> Option<String> {
    let mut value = Vec::new();
    for chunk in chunks {
        let bytes = chunk.as_ref();
        let total = value.len().checked_add(bytes.len())?;
        if total > MAX_VERIFICATION_RECORD_BYTES {
            return None;
        }
        value.extend_from_slice(bytes);
    }
    Some(sha256_hash(&value))
}

#[cfg(test)]
mod tests {
    use super::txt_chunks_sha256;
    use sdkwork_utils_rust::crypto::sha256_hash;

    #[test]
    fn matches_one_dns_txt_record_across_chunks_only_when_exact() {
        let chunks = vec![
            b"token-".to_vec().into_boxed_slice(),
            b"123".to_vec().into_boxed_slice(),
        ];
        let expected = sha256_hash(b"token-123");
        assert_eq!(
            txt_chunks_sha256(&chunks).as_deref(),
            Some(expected.as_str())
        );
        let oversized = vec![vec![b'x'; 513].into_boxed_slice()];
        assert!(txt_chunks_sha256(&oversized).is_none());
    }
}
