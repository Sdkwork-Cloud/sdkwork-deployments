/** Stable Drive sandbox volume id for `/opt/deploy` (server-owned; not a physical path). */
export const DEPLOY_LOCAL_SANDBOX_ID = "deploy.local.opt_deploy";

/** Default logical folder that holds cloned workspace modules. */
export const DEPLOY_SPACE_MODULE_PARENT = "sdkwork-space";

export interface LocalDeployNode {
  readonly id: string;
  readonly environment: "development" | "test" | "production" | "host";
  readonly labelKey: "node.development" | "node.test" | "node.production" | "node.host";
  readonly descriptionKey:
    | "node.development.description"
    | "node.test.description"
    | "node.production.description"
    | "node.host.description";
}

/** Local Docker / host runtimes for this phase (not remote SSH inventory). */
export const LOCAL_DEPLOY_NODES: readonly LocalDeployNode[] = [
  {
    id: "local.docker.development",
    environment: "development",
    labelKey: "node.development",
    descriptionKey: "node.development.description",
  },
  {
    id: "local.docker.test",
    environment: "test",
    labelKey: "node.test",
    descriptionKey: "node.test.description",
  },
  {
    id: "local.docker.production",
    environment: "production",
    labelKey: "node.production",
    descriptionKey: "node.production.description",
  },
  {
    id: "local.host",
    environment: "host",
    labelKey: "node.host",
    descriptionKey: "node.host.description",
  },
] as const;
