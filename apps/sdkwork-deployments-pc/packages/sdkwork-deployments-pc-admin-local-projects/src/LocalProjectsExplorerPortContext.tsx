import type { SandboxExplorerPort } from "@sdkwork/drive-pc-sandbox-contracts";
import { createContext, useContext, type ReactNode } from "react";

const LocalProjectsExplorerPortContext = createContext<SandboxExplorerPort | null>(null);

export function LocalProjectsExplorerPortProvider({
  children,
  port,
}: {
  children: ReactNode;
  port: SandboxExplorerPort | null;
}) {
  return (
    <LocalProjectsExplorerPortContext.Provider value={port}>
      {children}
    </LocalProjectsExplorerPortContext.Provider>
  );
}

export function useLocalProjectsExplorerPort(): SandboxExplorerPort | null {
  return useContext(LocalProjectsExplorerPortContext);
}
