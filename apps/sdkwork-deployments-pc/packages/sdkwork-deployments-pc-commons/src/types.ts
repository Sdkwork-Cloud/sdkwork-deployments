import type { ComponentType } from "react";

export type DeploymentsSurface = "app-console" | "backend-admin";
export type DeploymentsResourceKey = "sites" | "configuration" | "domains" | "certificates" | "artifacts" | "releases" | "deployments" | "monitoring" | "nginx" | "clusters" | "nodes" | "audit";
export interface DeploymentsModuleEntry { description: string; label: string; order: number; permission?: string; resource: DeploymentsResourceKey; }
export interface DeploymentsPcModuleDefinition { entries: readonly DeploymentsModuleEntry[]; id: string; label: string; surface: DeploymentsSurface; }
export interface DeploymentsQuery { page: number; pageSize: number; scopeId?: string; search?: string; }
export interface DeploymentsPage { items: readonly Record<string, unknown>[]; pageInfo: { page: number; pageSize: number; hasMore: boolean; total?: number }; }
export interface DeploymentsActionContext { body: Record<string, unknown>; file?: File; scopeId?: string; selectedItem?: Record<string, unknown>; }
export interface DeploymentsAction { bodyTemplate: Record<string, unknown>; dangerous?: boolean; execute(context: DeploymentsActionContext): Promise<unknown>; id: string; label: string; requiresFile?: boolean; requiresScope?: boolean; requiresSelection?: boolean; }
export interface DeploymentsDataSource { actions: readonly DeploymentsAction[]; load(query: DeploymentsQuery): Promise<DeploymentsPage>; requiresScope?: boolean; }
export type DeploymentsRegistry = Partial<Record<DeploymentsResourceKey, DeploymentsDataSource>>;
export interface DeploymentsResourcePageProps { locale: import("./i18n/index.ts").DeploymentsLocale; }
export type DeploymentsResourcePages = Partial<Record<DeploymentsResourceKey, ComponentType<DeploymentsResourcePageProps>>>;
