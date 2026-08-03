import { createClient, type SdkworkDeployBackendClient } from "@sdkwork/deployments-backend-sdk";
import {
  normalizeDeploymentsPage,
  type DeploymentsAction,
  type DeploymentsActionContext,
  type DeploymentsDataSource,
  type DeploymentsRegistry,
} from "@sdkwork/deployments-pc-commons";
import type { AuthTokenManager } from "@sdkwork/sdk-common";
import { createContext, useContext, type ReactNode } from "react";

const Context = createContext<SdkworkDeployBackendClient | null>(null);

export function createDeploymentsAdminClient(baseUrl: string, tokenManager: AuthTokenManager) {
  return createClient({ baseUrl, authMode: "dual-token", platform: "pc", tokenManager });
}

export function DeploymentsAdminProvider({ children, client }: { children: ReactNode; client: SdkworkDeployBackendClient }) {
  return <Context.Provider value={client}>{children}</Context.Provider>;
}

export function useDeploymentsAdminClient() {
  const value = useContext(Context);
  if (!value) throw new Error("DeploymentsAdminProvider is required");
  return value;
}

export function createDeploymentsAdminRegistry(client: SdkworkDeployBackendClient): DeploymentsRegistry {
  return {
    nginx: source(
      (query) => client.nginx.configs.list({ page: query.page, pageSize: query.pageSize }),
      [
        action("create", "Create config", { name: "", content: "", description: "" }, (context) =>
          client.nginx.configs.create(
            context.body as unknown as Parameters<typeof client.nginx.configs.create>[0],
            idempotencyParams(),
          )),
        action("validate", "Validate", {}, (context) =>
          client.nginx.configs.validate(selected(context, "id"), idempotencyParams()), { selection: true }),
        action("deploy", "Deploy", {}, (context) =>
          client.nginx.configs.deploy(selected(context, "id"), idempotencyParams()), { dangerous: true, selection: true }),
        action("reload", "Reload", {}, () => client.nginx.runtime.reload(idempotencyParams()), { dangerous: true }),
      ],
      ["configName", "siteId", "configType"],
    ),
    nodes: source(
      (query) => client.server.list({ page: query.page, pageSize: query.pageSize }),
      [
        action("create", "Register node", { name: "", host: "", sshPort: 22, sshUser: "root", clusterId: "", description: "" }, (context) =>
          client.server.create(
            context.body as unknown as Parameters<typeof client.server.create>[0],
            idempotencyParams(),
          )),
        action("update", "Update node", { status: 1, clusterId: "", description: "" }, (context) =>
          client.server.update(
            selected(context, "id"),
            context.body as unknown as Parameters<typeof client.server.update>[1],
          ), { selection: true }),
      ],
      ["name", "host", "clusterName"],
    ),
    clusters: source(
      (query) => client.cluster.list({ page: query.page, pageSize: query.pageSize }),
      [
        action("create", "Create cluster", { name: "", description: "", region: "" }, (context) =>
          client.cluster.create(
            context.body as unknown as Parameters<typeof client.cluster.create>[0],
            idempotencyParams(),
          )),
        action("update", "Update cluster", { status: 1, description: "" }, (context) =>
          client.cluster.update(
            selected(context, "id"),
            context.body as unknown as Parameters<typeof client.cluster.update>[1],
          ), { selection: true }),
      ],
      ["name", "region", "description"],
    ),
    audit: source((query) => client.audit.auditLogs.list({ page: query.page, pageSize: query.pageSize }), [], ["action", "resource"]),
  };
}

function source(
  load: (query: Parameters<DeploymentsDataSource["load"]>[0]) => Promise<unknown>,
  actions: readonly DeploymentsAction[],
  searchFields: readonly string[] = [],
): DeploymentsDataSource {
  return {
    actions,
    async load(query) {
      const page = normalizeDeploymentsPage(await load(query));
      const needle = query.search?.trim().toLowerCase();
      if (!needle || searchFields.length === 0) return page;
      return {
        ...page,
        items: page.items.filter((item) =>
          searchFields.some((field) => String(item[field] ?? "").toLowerCase().includes(needle))),
      };
    },
  };
}

function action(
  id: string,
  label: string,
  bodyTemplate: Record<string, unknown>,
  execute: DeploymentsAction["execute"],
  options: { dangerous?: boolean; selection?: boolean } = {},
): DeploymentsAction {
  return {
    id,
    label,
    bodyTemplate,
    execute,
    dangerous: options.dangerous,
    requiresSelection: options.selection,
  };
}

function selected(context: DeploymentsActionContext, field: string): string {
  const value = context.selectedItem?.[field] ?? context.selectedItem?.configId;
  if (typeof value !== "string" && typeof value !== "number") throw new Error(`${field} is unavailable`);
  return String(value);
}

function idempotencyParams(): { idempotencyKey: string } {
  if (typeof globalThis.crypto?.randomUUID !== "function") {
    throw new Error("crypto.randomUUID is required for idempotent deployment operations");
  }
  return { idempotencyKey: globalThis.crypto.randomUUID() };
}
