import { useSdkworkAuthControllerState } from "@sdkwork/auth-pc-react";
import { deploymentsModule as adminAudit } from "@sdkwork/deployments-pc-admin-audit";
import { deploymentsModule as infrastructure } from "@sdkwork/deployments-pc-admin-infrastructure";
import { deploymentsModule as adminNodes } from "@sdkwork/deployments-pc-admin-nodes";
import type { DeploymentsPcModuleDefinition } from "@sdkwork/deployments-pc-commons";
import { createDeploymentsConsoleRegistry, DeploymentsConsoleProvider } from "@sdkwork/deployments-pc-console-core";
import { deploymentsModule as delivery } from "@sdkwork/deployments-pc-console-delivery";
import { deploymentsModule as monitoring } from "@sdkwork/deployments-pc-console-monitoring";
import { deploymentsModule as publishing } from "@sdkwork/deployments-pc-console-publishing";
import { DeploymentsConsoleShell } from "@sdkwork/deployments-pc-console-shell";
import { deploymentsModule as configuration } from "@sdkwork/deployments-pc-console-site-configuration";
import { deploymentsModule as sites } from "@sdkwork/deployments-pc-console-sites";
import { lazy, Suspense, useMemo } from "react";
import { BrowserRouter, Navigate, Route, Routes } from "react-router-dom";

import { DeploymentsAuthGate } from "./auth/DeploymentsAuthGate.tsx";
import type { BootstrappedDeploymentsRuntime } from "./bootstrap/runtime.ts";

const consoleModules = [sites, configuration, delivery, publishing, monitoring] satisfies readonly DeploymentsPcModuleDefinition[];
const LazyDomainManagementPage = lazy(() => import("@sdkwork/deployments-pc-console-delivery/management").then((module) => ({ default: module.DomainManagementPage })));
const LazyCertificateManagementPage = lazy(() => import("@sdkwork/deployments-pc-console-delivery/management").then((module) => ({ default: module.CertificateManagementPage })));
const consoleResourcePages = { domains: LazyDomainManagementPage, certificates: LazyCertificateManagementPage } as const;
const adminModules = [infrastructure, adminNodes, adminAudit] satisfies readonly DeploymentsPcModuleDefinition[];
const LazyAuth = lazy(() => import("./auth/DeploymentsAuthRoutes.tsx").then((module) => ({ default: module.DeploymentsAuthRoutes })));
const LazyAdmin = lazy(() => import("./surfaces/DeploymentsAdminSurface.tsx").then((module) => ({ default: module.DeploymentsAdminSurface })));

export function App({ runtime }: { runtime: BootstrappedDeploymentsRuntime }) { return <BrowserRouter><Authenticated runtime={runtime} /></BrowserRouter>; }

function Authenticated({ runtime }: { runtime: BootstrappedDeploymentsRuntime }) {
  const state = useSdkworkAuthControllerState(runtime.authController);
  const registry = useMemo(() => createDeploymentsConsoleRegistry(runtime.clients), [runtime.clients]);
  const permissionScope = state.session?.context?.permissionScope ?? [];
  const userLabel = state.user?.displayName || state.user?.email;
  const signOut = () => { void runtime.authController.signOut(); };
  return <DeploymentsAuthGate controller={runtime.authController} authRoutes={<Suspense fallback={<div className="bootstrap-state">SDKWork Deployments</div>}><LazyAuth controller={runtime.authController} /></Suspense>}><DeploymentsConsoleProvider clients={runtime.clients}><Routes><Route path="/console/*" element={<DeploymentsConsoleShell locale={runtime.locale} modules={consoleModules} permissionScope={permissionScope} registry={registry} resourcePages={consoleResourcePages} userLabel={userLabel} onSignOut={signOut} />} /><Route path="/admin/*" element={<Suspense fallback={<div className="bootstrap-state">SDKWork Deployments</div>}><LazyAdmin backendApiBaseUrl={runtime.config.backendApiBaseUrl} locale={runtime.locale} modules={adminModules} permissionScope={permissionScope} tokenManager={runtime.tokenManager} userLabel={userLabel} onSignOut={signOut} /></Suspense>} /><Route path="*" element={<Navigate to="/console" replace />} /></Routes></DeploymentsConsoleProvider></DeploymentsAuthGate>;
}
