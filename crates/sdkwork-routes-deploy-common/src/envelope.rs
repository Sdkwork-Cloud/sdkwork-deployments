//! Map deploy domain DTOs to SdkWork HTTP API v3 envelope payloads.

use sdkwork_deploy_contract::{
    ArtifactPage, ArtifactResponse, AuditLogPage, AuditLogResponse, CertificatePage,
    CertificateResponse, DeploymentPage, DeploymentResponse, DomainHostnamePage,
    DomainHostnameResponse, DomainPage, DomainResponse, DomainVerifyResponse, DomainZonePage,
    DomainZoneResponse, EnvVariablePage, EnvVariableResponse, HealthCheckPage,
    HealthCheckResponse, NginxConfigPage, NginxConfigResponse, NginxReloadResponse,
    NginxStatusResponse, NginxValidateResponse, ReleasePage, ReleaseResponse, ServerPage,
    ServerResponse, SitePage, SiteResponse,
};
use sdkwork_deploy_core::normalize_pagination;
use sdkwork_utils_rust::{PageInfo, PageMode, SdkWorkPageData, SdkWorkResourceData};

pub fn resource<T>(item: T) -> SdkWorkResourceData<T> {
    SdkWorkResourceData { item }
}

pub fn site_page(page: SitePage) -> SdkWorkPageData<SiteResponse> {
    offset_page(page.items, page.page, page.page_size, page.total)
}

pub fn domain_zone_page(page: DomainZonePage) -> SdkWorkPageData<DomainZoneResponse> {
    offset_page(page.items, page.page, page.page_size, page.total)
}

pub fn domain_hostname_page(page: DomainHostnamePage) -> SdkWorkPageData<DomainHostnameResponse> {
    offset_page(page.items, page.page, page.page_size, page.total)
}

pub fn domain_page(data: DomainPage, page: i32, page_size: i32) -> SdkWorkPageData<DomainResponse> {
    offset_page(data.items, page, page_size, data.total)
}

pub fn deployment_page(page: DeploymentPage) -> SdkWorkPageData<DeploymentResponse> {
    offset_page(page.items, page.page, page.page_size, page.total)
}

pub fn env_variable_page(data: EnvVariablePage) -> SdkWorkPageData<EnvVariableResponse> {
    let page_size = data.items.len().max(1) as i32;
    offset_page(data.items, 1, page_size, data.total)
}

pub fn certificate_page(
    page: CertificatePage,
    page_num: i32,
    page_size: i32,
) -> SdkWorkPageData<CertificateResponse> {
    offset_page(page.items, page_num, page_size, page.total)
}

pub fn artifact_page(
    page: ArtifactPage,
    page_num: i32,
    page_size: i32,
) -> SdkWorkPageData<ArtifactResponse> {
    offset_page(page.items, page_num, page_size, page.total)
}

pub fn release_page(page: ReleasePage) -> SdkWorkPageData<ReleaseResponse> {
    offset_page(page.items, page.page, page.page_size, page.total)
}

pub fn health_check_page(data: HealthCheckPage) -> SdkWorkPageData<HealthCheckResponse> {
    let page_size = data.items.len().max(1) as i32;
    offset_page(data.items, 1, page_size, data.total)
}

pub fn nginx_config_page(page: NginxConfigPage) -> SdkWorkPageData<NginxConfigResponse> {
    offset_page(page.items, page.page, page.page_size, page.total)
}

pub fn server_page(
    page: ServerPage,
    page_num: i32,
    page_size: i32,
) -> SdkWorkPageData<ServerResponse> {
    offset_page(page.items, page_num, page_size, page.total)
}

pub fn audit_log_page(page: AuditLogPage) -> SdkWorkPageData<AuditLogResponse> {
    offset_page(page.items, page.page, page.page_size, page.total)
}

pub fn domain_verify(item: DomainVerifyResponse) -> SdkWorkResourceData<DomainVerifyResponse> {
    resource(item)
}

pub fn nginx_validate(item: NginxValidateResponse) -> SdkWorkResourceData<NginxValidateResponse> {
    resource(item)
}

pub fn nginx_reload(item: NginxReloadResponse) -> SdkWorkResourceData<NginxReloadResponse> {
    resource(item)
}

pub fn nginx_status(item: NginxStatusResponse) -> SdkWorkResourceData<NginxStatusResponse> {
    resource(item)
}

fn offset_page<T>(items: Vec<T>, page: i32, page_size: i32, total: i64) -> SdkWorkPageData<T> {
    let (page, page_size) = normalize_pagination(page, page_size);
    let total_pages = if page_size > 0 {
        Some(((total as f64) / page_size as f64).ceil() as i32)
    } else {
        None
    };
    SdkWorkPageData {
        items,
        page_info: PageInfo {
            mode: PageMode::Offset,
            page: Some(page),
            page_size: Some(page_size),
            total_items: Some(total.to_string()),
            total_pages,
            next_cursor: None,
            has_more: None,
        },
    }
}
