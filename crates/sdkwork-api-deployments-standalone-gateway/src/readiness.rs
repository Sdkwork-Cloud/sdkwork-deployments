use std::sync::Arc;

use sdkwork_intelligence_deploy_service::DeployService;
use sdkwork_web_bootstrap::{ReadinessCheck, ReadinessFuture};

pub struct DeployServiceReadinessCheck {
    service: Arc<DeployService>,
}

impl DeployServiceReadinessCheck {
    pub fn new(service: Arc<DeployService>) -> Self {
        Self { service }
    }
}

impl ReadinessCheck for DeployServiceReadinessCheck {
    fn check(&self) -> ReadinessFuture<'_> {
        let service = self.service.clone();
        Box::pin(async move {
            service
                .ready_check()
                .await
                .map_err(|error| error.to_string())
        })
    }
}
