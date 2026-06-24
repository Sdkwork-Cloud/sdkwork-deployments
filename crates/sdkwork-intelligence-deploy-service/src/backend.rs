//! Backend-api service surface implementation.

use async_trait::async_trait;
use sdkwork_deploy_contract::{
    CreateNginxConfigRequest, CreateServerRequest, DeployBackendApi, DeployBackendRequestContext,
    DeployServiceError, DeployServiceResult, ListNginxConfigsQuery, UpdateNginxConfigRequest,
};

use crate::DeployService;

impl DeployService {
    fn backend_tenant_scope(
        context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<Option<i64>> {
        Ok(context.tenant_id)
    }

    fn backend_write_tenant(context: &DeployBackendRequestContext) -> DeployServiceResult<i64> {
        context
            .tenant_id
            .filter(|tenant_id| *tenant_id > 0)
            .ok_or(DeployServiceError::validation(
                "tenant context is required for backend write operations",
            ))
    }
}

#[async_trait]
impl DeployBackendApi for DeployService {
    async fn list_nginx_configs(
        &self,
        context: &DeployBackendRequestContext,
        query: &ListNginxConfigsQuery,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigPage> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository.list_nginx_configs(tenant_id, query).await
    }

    async fn create_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateNginxConfigRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .create_nginx_config(tenant_id, request)
            .await
    }

    async fn retrieve_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .retrieve_nginx_config(tenant_id, config_id)
            .await
    }

    async fn update_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
        request: &UpdateNginxConfigRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .update_nginx_config(tenant_id, config_id, request)
            .await
    }

    async fn validate_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxValidateResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .validate_nginx_config(tenant_id, config_id)
            .await
    }

    async fn deploy_nginx_config(
        &self,
        context: &DeployBackendRequestContext,
        config_id: &str,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxConfigResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .deploy_nginx_config(tenant_id, config_id)
            .await
    }

    async fn reload_nginx(
        &self,
        _context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxReloadResponse> {
        self.repository.reload_nginx().await
    }

    async fn retrieve_nginx_status(
        &self,
        context: &DeployBackendRequestContext,
    ) -> DeployServiceResult<sdkwork_deploy_contract::NginxStatusResponse> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository.retrieve_nginx_status(tenant_id).await
    }

    async fn list_servers(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ServerPage> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository
            .list_servers(tenant_id, page, page_size)
            .await
    }

    async fn create_server(
        &self,
        context: &DeployBackendRequestContext,
        request: &CreateServerRequest,
    ) -> DeployServiceResult<sdkwork_deploy_contract::ServerResponse> {
        let tenant_id = Self::backend_write_tenant(context)?;
        self.repository.create_server(tenant_id, request).await
    }

    async fn list_audit_logs(
        &self,
        context: &DeployBackendRequestContext,
        page: i32,
        page_size: i32,
    ) -> DeployServiceResult<sdkwork_deploy_contract::AuditLogPage> {
        let tenant_id = Self::backend_tenant_scope(context)?;
        self.repository
            .list_audit_logs(tenant_id, page, page_size)
            .await
    }
}
