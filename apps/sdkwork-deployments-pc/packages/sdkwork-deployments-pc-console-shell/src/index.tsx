import { DeploymentsWorkspace, type DeploymentsWorkspaceProps } from "@sdkwork/deployments-pc-commons";
export function DeploymentsConsoleShell(props: Omit<DeploymentsWorkspaceProps, "surface">) { return <DeploymentsWorkspace {...props} surface="app-console" />; }
