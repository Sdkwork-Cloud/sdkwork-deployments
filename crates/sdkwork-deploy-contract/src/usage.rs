//! Traffic usage metering contract shared by the Web Server data plane and
//! the Deploy control plane.
//!
//! The Web Server records per-domain / per-server-IP traffic facts
//! (`traffic.requests`, `traffic.ingress_bytes`, `traffic.egress_bytes`)
//! attributed to the serving tenant and app; the control plane ingests them
//! into `deploy_usage_event` (deduplicated) and rolls them up into the daily
//! billing tables.

use serde::{Deserialize, Serialize};

/// Traffic usage dimensions recorded by the Web Server data plane.
pub const USAGE_DIMENSION_TRAFFIC_REQUESTS: &str = "traffic.requests";
pub const USAGE_DIMENSION_TRAFFIC_INGRESS_BYTES: &str = "traffic.ingress_bytes";
pub const USAGE_DIMENSION_TRAFFIC_EGRESS_BYTES: &str = "traffic.egress_bytes";

/// Traffic attribution recorded with every usage event: the serving domain,
/// the server's local IP/port, and — when the request was served through the
/// Deploy control plane — the app identity and app/binding references.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageEventAttribution {
    /// Normalized request hostname (domain dimension).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    /// Local server IP that served the request (server dimension).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listener_id: Option<String>,
    /// App public uuid (`deploy_app.uuid`) when attributable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    /// App slug when attributable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_slug: Option<String>,
    /// Site public uuid (`deploy_app.uuid`) when attributable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_uuid: Option<String>,
    /// Binding public uuid (`deploy_app_binding.uuid`) when attributable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub binding_uuid: Option<String>,
    /// Response status class (`2xx`, `3xx`, `4xx`, `5xx`) when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_class: Option<String>,
}

/// One traffic usage event submitted by a Web Server node.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageEventIngestItem {
    /// Event tenant when the node could attribute it; `0` means unmanaged
    /// (the control plane resolves the tenant from the binding when
    /// possible).
    #[serde(rename = "tenantId", default)]
    pub tenant_id: i64,
    #[serde(rename = "organizationId", default)]
    pub organization_id: i64,
    /// Site public uuid when attributable; resolved to `app_id` by the
    /// control plane.
    #[serde(rename = "appUuid", default, skip_serializing_if = "Option::is_none")]
    pub app_uuid: Option<String>,
    /// Binding public uuid when attributable; resolved to `binding_id` and
    /// used for tenant attribution by the control plane.
    #[serde(
        rename = "bindingUuid",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub binding_uuid: Option<String>,
    /// Aggregation window start (RFC 3339).
    #[serde(rename = "periodStart")]
    pub period_start: String,
    /// Usage dimension (`traffic.requests`, `traffic.ingress_bytes`,
    /// `traffic.egress_bytes`).
    pub dimension: String,
    /// Aggregated quantity over the window.
    pub quantity: i64,
    pub unit: String,
    /// Idempotency key (`traffic:<window>:<tenant>:<app>:<binding>:<host>:<ip>:<dim>`).
    #[serde(rename = "deduplicationKey")]
    pub deduplication_key: String,
    /// Traffic attribution (domain, server IP, app, status class).
    #[serde(rename = "attribution", default)]
    pub attribution: UsageEventAttribution,
    /// When the events were observed on the node (RFC 3339).
    #[serde(rename = "observedAt")]
    pub observed_at: String,
}

/// Batch traffic usage ingest request from a Web Server node.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IngestUsageEventsRequest {
    /// Node identity for audit (`SDKWORK_WEBSERVER_NODE_UUID`).
    #[serde(rename = "nodeUuid", default, skip_serializing_if = "Option::is_none")]
    pub node_uuid: Option<String>,
    pub events: Vec<UsageEventIngestItem>,
}

/// Result of a batch ingest.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageIngestResult {
    #[serde(rename = "ingested")]
    pub ingested: usize,
    #[serde(rename = "duplicates")]
    pub duplicates: usize,
    #[serde(rename = "rejected")]
    pub rejected: usize,
}
