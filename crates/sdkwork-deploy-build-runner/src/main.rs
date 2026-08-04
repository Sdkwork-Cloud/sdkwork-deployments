//! Build runner binary: claims QUEUED builds through the repository port,
//! plans bounded commands, executes them on the executor host, and reports
//! state transitions. Template commands come from
//! `SDKWORK_DEPLOY_BUILD_TEMPLATE_COMMANDS` (JSON array of bounded command
//! strings) for the governed command-executor path; platform constructors are
//! exercised by tests and enabled as credential integration lands.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use sdkwork_deploy_build_runner::{
    bounded_wait_timeout, BuildExecutor, CommandExecutor, ExecutionContext,
};
use sdkwork_deploy_contract::{BuildStatus, DeployServiceResult, UpdateBuildStateRequest};
use sdkwork_deploy_service_host::bootstrap_deploy_repository_from_env;
use sdkwork_intelligence_deploy_service::DeployRepositoryPort;
use tokio::time::{interval, MissedTickBehavior};

const DEFAULT_POLL_INTERVAL_MILLIS: u64 = 2_000;
const MAXIMUM_POLL_INTERVAL_MILLIS: u64 = 60_000;
const DEFAULT_TENANT_ID: i64 = 0;
const DEFAULT_TIMEOUT_SECONDS: u64 = 1800;

#[derive(Clone, Debug)]
struct RunnerConfig {
    tenant_id: i64,
    runner_node_uuid: String,
    runner_version: String,
    poll_interval: Duration,
    timeout: Duration,
    workspace_root: PathBuf,
    template_commands: Vec<String>,
}

impl RunnerConfig {
    fn from_env() -> Result<Self, String> {
        let tenant_id = match std::env::var("SDKWORK_DEPLOY_BUILD_TENANT_ID") {
            Ok(value) => value
                .parse::<i64>()
                .map_err(|error| format!("invalid SDKWORK_DEPLOY_BUILD_TENANT_ID: {error}"))?,
            Err(_) => DEFAULT_TENANT_ID,
        };
        let runner_node_uuid = std::env::var("SDKWORK_DEPLOY_BUILD_RUNNER_NODE_UUID")
            .unwrap_or_else(|_| "build-runner-local".to_owned());
        let runner_version = std::env::var("SDKWORK_DEPLOY_BUILD_RUNNER_VERSION")
            .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned());
        let poll_interval = match std::env::var("SDKWORK_DEPLOY_BUILD_POLL_INTERVAL_MILLIS") {
            Ok(value) => {
                let millis = value
                    .parse::<u64>()
                    .map_err(|error| format!("invalid poll interval: {error}"))?;
                if !(100..=MAXIMUM_POLL_INTERVAL_MILLIS).contains(&millis) {
                    return Err("poll interval must be 100..=60000 milliseconds".into());
                }
                Duration::from_millis(millis)
            }
            Err(_) => Duration::from_millis(DEFAULT_POLL_INTERVAL_MILLIS),
        };
        let timeout = bounded_wait_timeout(Duration::from_secs(
            std::env::var("SDKWORK_DEPLOY_BUILD_TIMEOUT_SECONDS")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(DEFAULT_TIMEOUT_SECONDS),
        ));
        let workspace_root = PathBuf::from(
            std::env::var("SDKWORK_DEPLOY_BUILD_WORKSPACE").unwrap_or_else(|_| {
                std::env::temp_dir()
                    .join("sdkwork-builds")
                    .to_string_lossy()
                    .into_owned()
            }),
        );
        let template_commands: Vec<String> =
            match std::env::var("SDKWORK_DEPLOY_BUILD_TEMPLATE_COMMANDS") {
                Ok(value) => serde_json::from_str(&value).map_err(|error| {
                    format!("SDKWORK_DEPLOY_BUILD_TEMPLATE_COMMANDS is invalid: {error}")
                })?,
                Err(_) => Vec::new(),
            };
        Ok(Self {
            tenant_id,
            runner_node_uuid,
            runner_version,
            poll_interval,
            timeout,
            workspace_root,
            template_commands,
        })
    }
}

async fn run_one_cycle(
    repository: &Arc<dyn DeployRepositoryPort>,
    config: &RunnerConfig,
    executor: &dyn BuildExecutor,
) -> DeployServiceResult<()> {
    let Some(build) = repository
        .claim_next_build(
            config.tenant_id,
            &config.runner_node_uuid,
            &config.runner_version,
        )
        .await?
    else {
        return Ok(());
    };
    tracing::info!(
        build = %build.id,
        app = %build.app_id,
        build_number = build.build_number,
        "claimed build"
    );

    let context = ExecutionContext {
        build_uuid: build.id.clone(),
        app_uuid: build.app_id.clone(),
        platform: build.platform_target_id.clone(),
        tech_stack: String::new(),
        semantic_version: None,
        working_directory: config.workspace_root.join(&build.id),
        runner_node_uuid: config.runner_node_uuid.clone(),
        runner_version: config.runner_version.clone(),
    };

    let outcome = match executor.plan(&context, &config.template_commands) {
        Ok(plan) => match tokio::time::timeout(config.timeout, executor.execute(&plan)).await {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(error)) => {
                report_terminal(
                    repository,
                    config,
                    &build.app_id,
                    &build.id,
                    BuildStatus::Failed,
                    &format!("BUILD_EXECUTE_FAILED:{error}"),
                )
                .await?;
                return Err(sdkwork_deploy_contract::DeployServiceError::Internal(
                    format!("execute build: {error}"),
                ));
            }
            Err(_) => {
                report_terminal(
                    repository,
                    config,
                    &build.app_id,
                    &build.id,
                    BuildStatus::TimedOut,
                    "BUILD_TIMEOUT",
                )
                .await?;
                return Err(sdkwork_deploy_contract::DeployServiceError::Internal(
                    "build timed out".into(),
                ));
            }
        },
        Err(error) => {
            report_terminal(
                repository,
                config,
                &build.app_id,
                &build.id,
                BuildStatus::Failed,
                &format!("BUILD_PLAN_FAILED:{error}"),
            )
            .await?;
            return Err(sdkwork_deploy_contract::DeployServiceError::Internal(
                format!("plan build: {error}"),
            ));
        }
    };

    if outcome.succeeded() {
        tracing::info!(build = %build.id, duration_ms = outcome.duration_ms, "build succeeded");
        repository
            .update_build_state(
                config.tenant_id,
                &build.app_id,
                &build.id,
                &UpdateBuildStateRequest {
                    build_status: BuildStatus::Succeeded,
                    runner_node_uuid: config.runner_node_uuid.clone(),
                    runner_version: Some(config.runner_version.clone()),
                    log_ref: Some(format!("local://{}", build.id)),
                    source_snapshot: None,
                    quality_gate: Some(
                        serde_json::json!({ "commands": config.template_commands.len() }),
                    ),
                    error_code: None,
                    started_at: None,
                    finished_at: None,
                },
            )
            .await?;
    } else {
        tracing::warn!(build = %build.id, exit_code = outcome.exit_code, "build failed");
        repository
            .update_build_state(
                config.tenant_id,
                &build.app_id,
                &build.id,
                &UpdateBuildStateRequest {
                    build_status: BuildStatus::Failed,
                    runner_node_uuid: config.runner_node_uuid.clone(),
                    runner_version: Some(config.runner_version.clone()),
                    log_ref: Some(format!("local://{}", build.id)),
                    source_snapshot: None,
                    quality_gate: None,
                    error_code: Some(outcome.error_code.unwrap_or_else(|| "BUILD_FAILED".into())),
                    started_at: None,
                    finished_at: None,
                },
            )
            .await?;
    }
    Ok(())
}

async fn report_terminal(
    repository: &Arc<dyn DeployRepositoryPort>,
    config: &RunnerConfig,
    app_id: &str,
    build_id: &str,
    status: BuildStatus,
    error_code: &str,
) -> DeployServiceResult<()> {
    tracing::error!(build = %build_id, error_code, "build failed in runner");
    repository
        .update_build_state(
            config.tenant_id,
            app_id,
            build_id,
            &UpdateBuildStateRequest {
                build_status: status,
                runner_node_uuid: config.runner_node_uuid.clone(),
                runner_version: Some(config.runner_version.clone()),
                log_ref: Some(format!("local://{build_id}")),
                source_snapshot: None,
                quality_gate: None,
                error_code: Some(error_code.to_owned()),
                started_at: None,
                finished_at: None,
            },
        )
        .await
        .map(|_| ())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}

#[tokio::main]
async fn main() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    let config = match RunnerConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            tracing::error!(error, "build runner configuration failed");
            std::process::exit(1);
        }
    };
    let repository = match bootstrap_deploy_repository_from_env().await {
        Ok(repository) => repository,
        Err(error) => {
            tracing::error!(error, "build runner bootstrap failed");
            std::process::exit(1);
        }
    };
    let repository_port: Arc<dyn DeployRepositoryPort> = repository;
    let executor = CommandExecutor::new(config.workspace_root.clone());

    tracing::info!(
        runner = %config.runner_node_uuid,
        tenant = config.tenant_id,
        poll_interval_ms = config.poll_interval.as_millis() as u64,
        "build runner started"
    );

    let mut ticker = interval(config.poll_interval);
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    tokio::select! {
        _ = shutdown_signal() => {
            tracing::info!("build runner shutting down");
        }
        _ = async {
            loop {
                ticker.tick().await;
                if let Err(error) = run_one_cycle(&repository_port, &config, &executor).await {
                    tracing::warn!(error = %error, "build runner cycle failed");
                }
            }
        } => {}
    }
}
