import type { DeploymentsPcModuleDefinition } from "@sdkwork/deployments-pc-commons";
export const deploymentsModule = { id: "delivery", label: "delivery", surface: "app-console", entries: [
    { resource: "domains", label: "Domains", description: "Domain verification and routing", permission: "deploy.domains.read", order: 1 },
    { resource: "certificates", label: "Certificates", description: "Managed and custom TLS certificates", permission: "deploy.certificates.read", order: 2 }
] } as const satisfies DeploymentsPcModuleDefinition;
