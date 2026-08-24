import {
  createClient as createDeployClient,
  type CertificateResponse,
  type CreateCertificateRequest,
  type CreateDomainHostnameRequest,
  type CreateDomainZoneRequest,
  type DomainHostnameResponse,
  type DomainVerifyResponse,
  type DomainZoneResponse,
  type PageInfo,
  type SdkworkDeployAppClient,
  type UpdateDomainHostnameRequest,
  type UpdateDomainZoneRequest,
} from "@sdkwork/deployments-app-sdk";
import { createDriveAppClient, type SdkworkDriveAppClient } from "@sdkwork/drive-app-sdk";
import {
  normalizeDeploymentsPage,
  type DeploymentsAction,
  type DeploymentsActionContext,
  type DeploymentsDataSource,
  type DeploymentsRegistry,
} from "@sdkwork/deployments-pc-commons";
import type { AuthTokenManager } from "@sdkwork/sdk-common";
import { uuid } from "@sdkwork/utils/id";
import { createContext, useContext, useMemo, type ReactNode } from "react";

export type {
  CertificateResponse,
  CreateCertificateRequest,
  CreateDomainHostnameRequest,
  CreateDomainZoneRequest,
  DomainHostnameResponse,
  DomainVerifyResponse,
  DomainZoneResponse,
  PageInfo,
  UpdateDomainHostnameRequest,
  UpdateDomainZoneRequest,
};

export interface DeploymentsConsoleClients {
  deploy: SdkworkDeployAppClient;
  drive: SdkworkDriveAppClient;
}

export interface DeploymentsDeliveryService {
  listDomainZones(params?: { page?: number; pageSize?: number; status?: "ACTIVE" | "PAUSED"; keyword?: string }): Promise<{ items: DomainZoneResponse[]; pageInfo: PageInfo }>;
  createDomainZone(body: CreateDomainZoneRequest): Promise<DomainZoneResponse>;
  retrieveDomainZone(zoneId: string): Promise<DomainZoneResponse>;
  updateDomainZone(zoneId: string, body: UpdateDomainZoneRequest): Promise<DomainZoneResponse>;
  deleteDomainZone(zoneId: string): Promise<void>;
  listDomainHostnames(zoneId: string, params?: { page?: number; pageSize?: number }): Promise<{ items: DomainHostnameResponse[]; pageInfo: PageInfo }>;
  createDomainHostname(zoneId: string, body: CreateDomainHostnameRequest): Promise<DomainHostnameResponse>;
  updateDomainHostname(zoneId: string, hostnameId: string, body: UpdateDomainHostnameRequest): Promise<DomainHostnameResponse>;
  verifyDomainHostname(zoneId: string, hostnameId: string): Promise<DomainVerifyResponse>;
  deleteDomainHostname(zoneId: string, hostnameId: string): Promise<void>;
  listCertificates(params?: { page?: number; pageSize?: number }): Promise<{ items: CertificateResponse[]; pageInfo: PageInfo }>;
  createCertificate(body: CreateCertificateRequest): Promise<CertificateResponse>;
  renewCertificate(certificateId: string): Promise<CertificateResponse>;
  deleteCertificate(certificateId: string): Promise<void>;
}

const Context = createContext<DeploymentsConsoleClients | null>(null);

export function createDeploymentsConsoleClients(config: {
  deployBaseUrl: string;
  driveBaseUrl: string;
  tokenManager: AuthTokenManager;
}): DeploymentsConsoleClients {
  const common = {
    authMode: "dual-token" as const,
    platform: "pc",
    tokenManager: config.tokenManager,
  };
  return {
    deploy: createDeployClient({ ...common, baseUrl: config.deployBaseUrl }),
    drive: createDriveAppClient({ ...common, baseUrl: config.driveBaseUrl }),
  };
}

export function DeploymentsConsoleProvider({ children, clients }: { children: ReactNode; clients: DeploymentsConsoleClients }) {
  return <Context.Provider value={clients}>{children}</Context.Provider>;
}

export function useDeploymentsConsoleClients(): DeploymentsConsoleClients {
  const value = useContext(Context);
  if (!value) throw new Error("DeploymentsConsoleProvider is required");
  return value;
}

export function createDeploymentsDeliveryService(client: SdkworkDeployAppClient): DeploymentsDeliveryService {
  const zones = client.domain.domainZones;
  return {
    listDomainZones: (params) => zones.list(params),
    createDomainZone: (body) => zones.create(body, idempotencyParams()),
    retrieveDomainZone: (zoneId) => zones.retrieve(zoneId),
    updateDomainZone: (zoneId, body) => zones.update(zoneId, body),
    deleteDomainZone: (zoneId) => zones.delete(zoneId),
    listDomainHostnames: (zoneId, params) => zones.hostnames.list(zoneId, params),
    createDomainHostname: (zoneId, body) => zones.hostnames.create(zoneId, body, idempotencyParams()),
    updateDomainHostname: (zoneId, hostnameId, body) => zones.hostnames.update(zoneId, hostnameId, body),
    verifyDomainHostname: (zoneId, hostnameId) => zones.hostnames.verify(zoneId, hostnameId, idempotencyParams()),
    deleteDomainHostname: (zoneId, hostnameId) => zones.hostnames.delete(zoneId, hostnameId),
    listCertificates: (params) => client.certificate.list(params),
    createCertificate: (body) => client.certificate.create(body, idempotencyParams()),
    renewCertificate: (certificateId) => client.certificate.renew(certificateId, idempotencyParams()),
    deleteCertificate: (certificateId) => client.certificate.delete(certificateId),
  };
}

export function useDeploymentsDeliveryService(): DeploymentsDeliveryService {
  const { deploy } = useDeploymentsConsoleClients();
  return useMemo(() => createDeploymentsDeliveryService(deploy), [deploy]);
}

export function createDeploymentsConsoleRegistry(clients: DeploymentsConsoleClients): DeploymentsRegistry {
  const client = clients.deploy;
  return {
    sites: source(
      (query) => client.site.list({ page: query.page, pageSize: query.pageSize, keyword: query.search }),
      [
        action("create", "Create application", { name: "", slug: "", description: "", siteType: 1 }, (context) =>
          client.site.create(
            context.body as unknown as Parameters<typeof client.site.create>[0],
            idempotencyParams(),
          )),
        action("update", "Update", { name: "", description: "" }, (context) =>
          client.site.update(selected(context, "id"), context.body as unknown as Parameters<typeof client.site.update>[1]), { selection: true }),
        action("activate", "Activate", {}, (context) =>
          client.site.activate(selected(context, "id"), idempotencyParams()), { selection: true }),
        action("pause", "Disable", {}, (context) =>
          client.site.pause(selected(context, "id"), idempotencyParams()), { dangerous: true, selection: true }),
        action("delete", "Delete", {}, (context) => client.site.delete(selected(context, "id")), { dangerous: true, selection: true }),
      ],
    ),
    configuration: scoped(
      (query) => client.envVariable.sites.envVariables.list(requiredSiteId(query.scopeId), { environment: query.search }),
      [
        action("variable", "Add variable", { key: "", value: "", environment: "production", isSecret: false }, (context) =>
          client.envVariable.sites.envVariables.create(
            requiredSiteId(context.scopeId),
            context.body as unknown as Parameters<typeof client.envVariable.sites.envVariables.create>[1],
            idempotencyParams(),
          ), { scope: true }),
        action("check", "Add health check", { name: "", url: "", checkInterval: 30 }, (context) =>
          client.monitor.sites.healthChecks.create(
            requiredSiteId(context.scopeId),
            context.body as unknown as Parameters<typeof client.monitor.sites.healthChecks.create>[1],
            idempotencyParams(),
          ), { scope: true }),
      ],
    ),
    domains: source(
      (query) => client.domain.domainZones.list({ page: query.page, pageSize: query.pageSize, keyword: query.search }),
      [],
    ),
    certificates: source((query) => client.certificate.list({ page: query.page, pageSize: query.pageSize }), []),
    artifacts: source(
      (query) => client.artifact.list({ page: query.page, pageSize: query.pageSize }),
      [
        action("upload", "Upload application", { packageType: 1, checksumSha256: "" }, async (context) => {
          const file = context.file;
          const siteId = requiredSiteId(context.scopeId);
          if (!file) throw new Error("Package file is required");
          const idempotencyKey = uuid();
          const uploaded = await clients.drive.uploader.uploadArchive({
            file,
            appResourceType: "deploy.artifact",
            appResourceId: siteId,
            scene: "deployment-package",
            source: "sdkwork-deployments-pc",
            originalFileName: file.name,
            contentType: file.type || "application/octet-stream",
          });
          return client.artifact.create({
            siteId,
            packageType: Number(context.body.packageType ?? 1),
            fileName: file.name,
            contentType: file.type || "application/octet-stream",
            contentLength: String(file.size),
            checksumSha256: stringValue(context.body.checksumSha256) || uploaded.uploadItem.checksumSha256Hex,
            driveUploadSessionId: uploaded.uploadSession.id,
            driveUploadItemId: uploaded.uploadItem.id,
            driveSpaceId: uploaded.uploadItem.spaceId,
            driveNodeId: uploaded.uploadItem.nodeId,
            idempotencyKey,
          }, { idempotencyKey });
        }, { file: true, scope: true }),
        action("delete", "Retain and remove", {}, (context) => client.artifact.delete(selected(context, "id")), { dangerous: true, selection: true }),
      ],
    ),
    releases: scoped(
      (query) => client.release.sites.releases.list(requiredSiteId(query.scopeId), { page: query.page, pageSize: query.pageSize }),
      [action("create", "Create release", { artifactId: "", versionTag: "" }, (context) => {
        const idempotencyKey = uuid();
        return client.release.sites.releases.create(
          requiredSiteId(context.scopeId),
          { ...context.body, idempotencyKey } as unknown as Parameters<typeof client.release.sites.releases.create>[1],
          { idempotencyKey },
        );
      }, { scope: true })],
    ),
    deployments: scoped(
      (query) => client.deployment.sites.deployments.list(requiredSiteId(query.scopeId), { page: query.page, pageSize: query.pageSize }),
      [
        action("deploy", "Start deployment", { deployType: 1, releaseId: "", environment: "production" }, (context) => {
          const idempotencyKey = uuid();
          return client.deployment.sites.deployments.create(
            requiredSiteId(context.scopeId),
            { ...context.body, idempotencyKey } as unknown as Parameters<typeof client.deployment.sites.deployments.create>[1],
            { idempotencyKey },
          );
        }, { scope: true }),
        action("rollback", "Rollback", {}, (context) =>
          client.deployment.sites.deployments.rollback(
            requiredSiteId(context.scopeId),
            selected(context, "id"),
            idempotencyParams(),
          ), { dangerous: true, scope: true, selection: true }),
      ],
    ),
    monitoring: scoped(
      (query) => client.monitor.sites.healthChecks.list(requiredSiteId(query.scopeId)),
      [action("create", "Add health check", { name: "", url: "", checkInterval: 30 }, (context) =>
        client.monitor.sites.healthChecks.create(
          requiredSiteId(context.scopeId),
          context.body as unknown as Parameters<typeof client.monitor.sites.healthChecks.create>[1],
          idempotencyParams(),
        ), { scope: true })],
    ),
  };
}

function source(
  load: (query: Parameters<DeploymentsDataSource["load"]>[0]) => Promise<unknown>,
  actions: readonly DeploymentsAction[],
): DeploymentsDataSource {
  return {
    actions,
    async load(query) {
      return normalizeDeploymentsPage(await load(query));
    },
  };
}

function scoped(load: Parameters<typeof source>[0], actions: readonly DeploymentsAction[]): DeploymentsDataSource {
  return { ...source(load, actions), requiresScope: true };
}

function action(
  id: string,
  label: string,
  bodyTemplate: Record<string, unknown>,
  execute: DeploymentsAction["execute"],
  options: { dangerous?: boolean; file?: boolean; scope?: boolean; selection?: boolean } = {},
): DeploymentsAction {
  return {
    id,
    label,
    bodyTemplate,
    execute,
    dangerous: options.dangerous,
    requiresFile: options.file,
    requiresScope: options.scope,
    requiresSelection: options.selection,
  };
}

function requiredSiteId(value: string | undefined): string {
  if (!value?.trim()) throw new Error("Site ID is required");
  return value.trim();
}

function selected(context: DeploymentsActionContext, field: string): string {
  const value = context.selectedItem?.[field];
  if (typeof value !== "string" && typeof value !== "number") throw new Error(`${field} is unavailable`);
  return String(value);
}

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" && value.trim() ? value.trim() : undefined;
}

function idempotencyParams(): { idempotencyKey: string } {
  return { idempotencyKey: uuid() };
}
