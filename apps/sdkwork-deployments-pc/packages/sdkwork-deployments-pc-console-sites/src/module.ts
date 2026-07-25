import type { DeploymentsPcModuleDefinition } from "@sdkwork/deployments-pc-commons";
export const deploymentsModule = { id: "sites", label: "sites", surface: "app-console", entries: [
    { resource: "sites", label: "Applications", description: "Application lifecycle and availability", permission: "deploy.sites.read", order: 1 }
] } as const satisfies DeploymentsPcModuleDefinition;
