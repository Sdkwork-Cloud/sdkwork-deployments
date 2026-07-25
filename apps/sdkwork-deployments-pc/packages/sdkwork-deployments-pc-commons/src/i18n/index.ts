import { deploymentsWorkspaceEnUs } from "./en-US/deployments/workspace/workspace.ts";
import { deploymentsWorkspaceZhCn } from "./zh-CN/deployments/workspace/workspace.ts";

export type DeploymentsLocale = "en-US" | "zh-CN";
export type DeploymentsMessageKey = keyof typeof deploymentsWorkspaceEnUs;

const catalogs: Record<DeploymentsLocale, Record<DeploymentsMessageKey, string>> = {
  "en-US": deploymentsWorkspaceEnUs,
  "zh-CN": deploymentsWorkspaceZhCn,
};

export function translateDeployments(locale: DeploymentsLocale, key: DeploymentsMessageKey, values: Record<string, string | number> = {}): string {
  return Object.entries(values).reduce((message, [name, value]) => message.replaceAll(`{${name}}`, String(value)), catalogs[locale][key]);
}
