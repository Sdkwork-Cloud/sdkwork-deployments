import type { DeploymentsPcModuleDefinition } from "@sdkwork/deployments-pc-commons";
export const deploymentsModule = { id: "publishing", label: "publishing", surface: "app-console", entries: [
    { resource: "artifacts", label: "Artifacts", description: "Drive-backed application packages", permission: "deploy.artifacts.read", order: 1 },
    { resource: "releases", label: "Releases", description: "Immutable release versions", permission: "deploy.releases.read", order: 2 },
    { resource: "deployments", label: "Deployments", description: "Rollout history and rollback", permission: "deploy.deployments.read", order: 3 }
] } as const satisfies DeploymentsPcModuleDefinition;
