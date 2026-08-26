//! Platform app publishing domain catalog.
//!
//! Every App is automatically published on `<slug>.app.<suffix>` for each
//! platform suffix (the same 14-domain catalog the IM/drive/knowledgebase
//! modules use, `SDKWORK_WEBSERVER_SPEC.md` domain inventory). Users can
//! additionally bind custom domains to their site; the Web Server resolves
//! unmatched hosts against the Deploy control plane
//! (`sdkwork-webserver` `appDomainFallback` section).

/// Platform app-domain suffixes (lowercase, no leading dot). Keep in sync
/// with the sdkwork-im `deployments/webserver/server.*.toml` serverName
/// catalog: sdkwork.com, sdkwork.cn, birdcoder.com, birdcoder.cn, dtupay.com,
/// dtupay.cn, skubc.com, skubc.cn, zowalk.com, zowalk.cn, offer86.com,
/// offer86.cn, 86offer.com, 86offer.cn.
pub const PLATFORM_APP_DOMAIN_SUFFIXES: [&str; 14] = [
    "sdkwork.com",
    "sdkwork.cn",
    "birdcoder.com",
    "birdcoder.cn",
    "dtupay.com",
    "dtupay.cn",
    "skubc.com",
    "skubc.cn",
    "zowalk.com",
    "zowalk.cn",
    "offer86.com",
    "offer86.cn",
    "86offer.com",
    "86offer.cn",
];

/// Subdomain label placed between the app slug and the platform suffix.
/// Production uses `app`; non-production environments use `app-<env>` so
/// every lifecycle environment gets its own publishable hostname
/// (`myapp.app.sdkwork.com` / `myapp.app-dev.sdkwork.com`).
pub fn app_domain_label(environment: &str) -> &'static str {
    match environment {
        "development" => "app-dev",
        "test" => "app-test",
        "staging" => "app-staging",
        _ => "app",
    }
}

/// The default publishable hostname for an app in one environment, for one
/// platform suffix: `<slug>.app[-<env>].<suffix>`.
pub fn default_app_hostname(slug: &str, suffix: &str, environment: &str) -> String {
    format!("{slug}.{}.{suffix}", app_domain_label(environment))
}

/// Parsed default app hostname: the slug (app id) and the platform suffix
/// behind `<slug>.app[-<env>].<suffix>`. `None` when the hostname is not a
/// platform default app hostname (custom domains do not parse).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DefaultAppHostname {
    pub slug: String,
    pub environment: String,
    pub suffix: String,
}

/// Parse a hostname into its default-app parts. The app label must be one of
/// the platform labels (`app`, `app-dev`, `app-test`, `app-staging`) and the
/// suffix must be in the platform catalog. Hostnames are compared
/// case-insensitively and must be ASCII.
pub fn parse_default_app_hostname(hostname: &str) -> Option<DefaultAppHostname> {
    let hostname = hostname.trim().to_ascii_lowercase();
    if hostname.is_empty()
        || hostname.len() > 253
        || hostname.ends_with('.')
        || !hostname.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-')
        })
    {
        return None;
    }
    let mut labels = hostname.split('.');
    let slug = labels.next()?;
    let label = labels.next()?;
    let suffix = labels.collect::<Vec<_>>().join(".");
    if !label_is_sane(slug) || suffix.is_empty() || !labels_are_sane(&suffix) {
        return None;
    }
    let environment = match label {
        "app" => "production",
        "app-dev" => "development",
        "app-test" => "test",
        "app-staging" => "staging",
        _ => return None,
    };
    if !PLATFORM_APP_DOMAIN_SUFFIXES.contains(&suffix.as_str()) {
        return None;
    }
    Some(DefaultAppHostname {
        slug: slug.to_owned(),
        environment: environment.to_owned(),
        suffix,
    })
}

/// Wildcard form used for DNS and TLS planning: `*.app[-<env>].<suffix>`.
pub fn default_app_domain_pattern(environment: &str, suffix: &str) -> String {
    format!("*.{}.{suffix}", app_domain_label(environment))
}

fn labels_are_sane(suffix: &str) -> bool {
    let labels = suffix.split('.');
    let mut count = 0;
    for label in labels {
        count += 1;
        if !label_is_sane(label) {
            return false;
        }
    }
    count >= 2
}

fn label_is_sane(label: &str) -> bool {
    !label.is_empty() && label.len() <= 63 && !label.starts_with('-') && !label.ends_with('-')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_matches_the_im_module_domain_inventory() {
        assert_eq!(PLATFORM_APP_DOMAIN_SUFFIXES.len(), 14);
        assert!(PLATFORM_APP_DOMAIN_SUFFIXES.contains(&"sdkwork.com"));
        assert!(PLATFORM_APP_DOMAIN_SUFFIXES.contains(&"sdkwork.cn"));
        assert!(PLATFORM_APP_DOMAIN_SUFFIXES.contains(&"86offer.cn"));
        for suffix in PLATFORM_APP_DOMAIN_SUFFIXES {
            assert!(suffix.is_ascii() && !suffix.starts_with('.') && !suffix.ends_with('.'));
            assert_eq!(suffix, suffix.to_ascii_lowercase());
        }
    }

    #[test]
    fn production_and_environment_hostnames_are_distinct() {
        assert_eq!(
            default_app_hostname("myapp", "sdkwork.com", "production"),
            "myapp.app.sdkwork.com"
        );
        assert_eq!(
            default_app_hostname("myapp", "sdkwork.com", "development"),
            "myapp.app-dev.sdkwork.com"
        );
        assert_eq!(
            default_app_hostname("myapp", "sdkwork.com", "test"),
            "myapp.app-test.sdkwork.com"
        );
        assert_eq!(
            default_app_hostname("myapp", "sdkwork.com", "staging"),
            "myapp.app-staging.sdkwork.com"
        );
        assert_eq!(
            default_app_hostname("myapp", "86offer.cn", "production"),
            "myapp.app.86offer.cn"
        );
    }

    #[test]
    fn parse_round_trips_default_hostnames() {
        for (hostname, slug, environment, suffix) in [
            (
                "myapp.app.sdkwork.com",
                "myapp",
                "production",
                "sdkwork.com",
            ),
            (
                "myapp.app-dev.sdkwork.cn",
                "myapp",
                "development",
                "sdkwork.cn",
            ),
            (
                "shop.app-test.birdcoder.com",
                "shop",
                "test",
                "birdcoder.com",
            ),
            (
                "wiki.app-staging.86offer.cn",
                "wiki",
                "staging",
                "86offer.cn",
            ),
        ] {
            let parsed = parse_default_app_hostname(hostname).expect("must parse");
            assert_eq!(
                parsed,
                DefaultAppHostname {
                    slug: slug.to_owned(),
                    environment: environment.to_owned(),
                    suffix: suffix.to_owned(),
                }
            );
        }
        // Case-insensitive.
        assert_eq!(
            parse_default_app_hostname("MyApp.APP.Sdkwork.COM").map(|value| value.slug),
            Some("myapp".to_owned())
        );
    }

    #[test]
    fn custom_and_malformed_hostnames_do_not_parse() {
        for hostname in [
            "mysite.example.com",
            "myapp.example.com",
            "myapp.app.example.com",
            "myapp.app.",
            ".app.sdkwork.com",
            "myapp.app",
            "myapp.other.sdkwork.com",
            "myapp.app.sdkwork.com.evil.com",
            "myapp.app.sdkwork.com.",
            "-myapp.app.sdkwork.com",
            "myapp.app.sdkwork.com/path",
            "",
        ] {
            assert_eq!(
                parse_default_app_hostname(hostname),
                None,
                "hostname must not parse: {hostname}"
            );
        }
    }

    #[test]
    fn wildcard_patterns_cover_the_whole_catalog() {
        assert_eq!(
            default_app_domain_pattern("production", "sdkwork.com"),
            "*.app.sdkwork.com"
        );
        assert_eq!(
            default_app_domain_pattern("development", "sdkwork.cn"),
            "*.app-dev.sdkwork.cn"
        );
    }
}
