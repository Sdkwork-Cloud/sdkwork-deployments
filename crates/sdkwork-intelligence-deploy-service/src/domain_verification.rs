//! Domain ownership normalization and verification port.

use std::{net::IpAddr, str::FromStr};

use async_trait::async_trait;
use sdkwork_deploy_contract::{DeployServiceError, DeployServiceResult, DomainVerifyResponse};

pub const DOMAIN_VERIFICATION_METHOD_DNS_TXT: &str = "DNS_TXT";
const DOMAIN_VERIFICATION_RECORD_LABEL: &str = "_sdkwork-verification";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomainVerificationChallenge {
    pub hostname: String,
    pub verified: bool,
    pub token: Option<String>,
}

impl DomainVerificationChallenge {
    pub fn response(&self) -> DomainVerifyResponse {
        DomainVerifyResponse {
            verified: self.verified,
            method: DOMAIN_VERIFICATION_METHOD_DNS_TXT.to_owned(),
            token: if self.verified {
                None
            } else {
                self.token.clone()
            },
        }
    }
}

#[async_trait]
pub trait DomainOwnershipVerifierPort: Send + Sync {
    async fn verify_dns_txt(&self, hostname: &str, token: &str) -> DeployServiceResult<bool>;
}

pub struct UnconfiguredDomainOwnershipVerifier;

#[async_trait]
impl DomainOwnershipVerifierPort for UnconfiguredDomainOwnershipVerifier {
    async fn verify_dns_txt(&self, _hostname: &str, _token: &str) -> DeployServiceResult<bool> {
        Ok(false)
    }
}

pub fn normalize_domain_hostname(value: &str) -> DeployServiceResult<String> {
    let value = value.trim();
    let (wildcard, domain) = match value.strip_prefix("*.") {
        Some(domain) => (true, domain),
        None => (false, value),
    };
    let domain = domain.strip_suffix('.').unwrap_or(domain);
    if domain.is_empty() || domain.contains('*') {
        return Err(DeployServiceError::validation("hostname is invalid"));
    }

    let ascii = idna::domain_to_ascii_strict(domain)
        .map_err(|_| DeployServiceError::validation("hostname is invalid"))?
        .to_ascii_lowercase();
    if ascii.is_empty()
        || ascii.len() > 231
        || !ascii.contains('.')
        || IpAddr::from_str(&ascii).is_ok()
        || ascii
            .split('.')
            .any(|label| label.is_empty() || label.len() > 63)
    {
        return Err(DeployServiceError::validation("hostname is invalid"));
    }

    Ok(if wildcard {
        format!("*.{ascii}")
    } else {
        ascii
    })
}

pub fn dns_txt_record_name(hostname: &str) -> DeployServiceResult<String> {
    let hostname = hostname.strip_prefix("*.").unwrap_or(hostname);
    let record_name = format!("{DOMAIN_VERIFICATION_RECORD_LABEL}.{hostname}");
    if record_name.len() > 253 {
        return Err(DeployServiceError::validation(
            "hostname is too long for domain verification",
        ));
    }
    Ok(record_name)
}

#[cfg(test)]
mod tests {
    use super::{dns_txt_record_name, normalize_domain_hostname};

    #[test]
    fn normalizes_idna_case_trailing_dot_and_wildcard() {
        assert_eq!(
            normalize_domain_hostname(" WWW.Example.COM. ").unwrap(),
            "www.example.com"
        );
        assert_eq!(
            normalize_domain_hostname("*.BUECHER.example").unwrap(),
            "*.buecher.example"
        );
        assert_eq!(
            dns_txt_record_name("*.example.com").unwrap(),
            "_sdkwork-verification.example.com"
        );
    }

    #[test]
    fn rejects_ip_single_label_and_ambiguous_wildcards() {
        for hostname in ["127.0.0.1", "localhost", "*.*.example.com", ""] {
            assert!(normalize_domain_hostname(hostname).is_err(), "{hostname}");
        }
    }
}
