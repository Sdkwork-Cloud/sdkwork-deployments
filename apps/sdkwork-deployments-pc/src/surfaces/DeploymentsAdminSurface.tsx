import { createDeploymentsAdminClient, createDeploymentsAdminRegistry, DeploymentsAdminProvider } from "@sdkwork/deployments-pc-admin-core";
import { DeploymentsAdminShell } from "@sdkwork/deployments-pc-admin-shell";
import type { DeploymentsLocale, DeploymentsPcModuleDefinition } from "@sdkwork/deployments-pc-commons";
import type { AuthTokenManager } from "@sdkwork/sdk-common";
import { useMemo } from "react";

export interface DeploymentsAdminSurfaceProps {
  backendApiBaseUrl: string;
  locale: DeploymentsLocale;
  modules: readonly DeploymentsPcModuleDefinition[];
  onSignOut(): void;
  permissionScope: readonly string[];
  tokenManager: AuthTokenManager;
  userLabel?: string;
}

export function DeploymentsAdminSurface({ backendApiBaseUrl, locale, modules, onSignOut, permissionScope, tokenManager, userLabel }: DeploymentsAdminSurfaceProps) {
  const client = useMemo(() => createDeploymentsAdminClient(backendApiBaseUrl, tokenManager), [backendApiBaseUrl, tokenManager]);
  const registry = useMemo(() => createDeploymentsAdminRegistry(client), [client]);
  return <DeploymentsAdminProvider client={client}><DeploymentsAdminShell locale={locale} modules={modules} permissionScope={permissionScope} registry={registry} userLabel={userLabel} onSignOut={onSignOut} /></DeploymentsAdminProvider>;
}
