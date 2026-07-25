import { DeploymentsWorkspace, type DeploymentsWorkspaceProps } from "@sdkwork/deployments-pc-commons";
export function DeploymentsAdminShell(props: Omit<DeploymentsWorkspaceProps, "surface">) { return <DeploymentsWorkspace {...props} surface="backend-admin" />; }
