import type { DeploymentsPcModuleDefinition } from "@sdkwork/deployments-pc-commons";
export const deploymentsModule = { id: "site-configuration", label: "site configuration", surface: "app-console", entries: [
    { resource: "configuration", label: "Configuration", description: "Environment variables and health checks", permission: "deploy.sites.read", order: 1 }
] } as const satisfies DeploymentsPcModuleDefinition;
