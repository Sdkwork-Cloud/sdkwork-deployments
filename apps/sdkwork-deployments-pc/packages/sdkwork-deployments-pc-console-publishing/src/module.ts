import type { DeploymentsPcModuleDefinition } from "@sdkwork/deployments-pc-commons";
export const deploymentsModule = { id: "publishing", label: "publishing", surface: "app-console", entries: [
    { resource: "apps", label: "Apps", description: "Create and publish deploy_app applications", permission: "deploy.apps.read", order: 1 },
    { resource: "artifacts", label: "Artifacts", description: "Drive-backed application packages", permission: "deploy.artifacts.read", order: 2 },
    { resource: "releases", label: "Releases", description: "Immutable release versions", permission: "deploy.releases.read", order: 3 },
    { resource: "deployments", label: "Deployments", description: "Rollout history and rollback", permission: "deploy.deployments.read", order: 4 }
] } as const satisfies DeploymentsPcModuleDefinition;
