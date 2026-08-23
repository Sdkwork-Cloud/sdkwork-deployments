import type { DeploymentsLocale } from "@sdkwork/deployments-pc-commons";
import type { SandboxEntry } from "@sdkwork/drive-pc-sandbox-contracts";
import { SandboxExplorerView } from "@sdkwork/drive-pc-sandbox-explorer";
import { FolderTree, RefreshCw, Server, X } from "lucide-react";
import { useCallback, useEffect, useMemo, useState } from "react";

import {
  DEPLOY_LOCAL_SANDBOX_ID,
  DEPLOY_SPACE_MODULE_PARENT,
  LOCAL_DEPLOY_NODES,
} from "./constants.ts";
import { translateLocalProjects } from "./i18n.ts";
import { useLocalProjectsExplorerPort } from "./LocalProjectsExplorerPortContext.tsx";
import "./LocalProjectsPage.css";

export interface LocalProjectsPageProps {
  locale: DeploymentsLocale;
}

type BrowserTarget =
  | { kind: "root" }
  | { kind: "module"; name: string; logicalPath: string }
  | { kind: "node"; id: string; label: string };

export function LocalProjectsPage({ locale }: LocalProjectsPageProps) {
  const t = useCallback(
    (key: Parameters<typeof translateLocalProjects>[1], values?: Record<string, string | number>) =>
      translateLocalProjects(locale, key, values),
    [locale],
  );
  const port = useLocalProjectsExplorerPort();
  const [modules, setModules] = useState<readonly SandboxEntry[]>([]);
  const [modulesBusy, setModulesBusy] = useState(false);
  const [modulesError, setModulesError] = useState<string>();
  const [sandboxAvailable, setSandboxAvailable] = useState(true);
  const [browserTarget, setBrowserTarget] = useState<BrowserTarget | null>(null);

  const loadModules = useCallback(async () => {
    if (!port) return;
    setModulesBusy(true);
    setModulesError(undefined);
    try {
      const sandboxes = await port.listSandboxes({ page: 1, pageSize: 50 });
      const deploy = sandboxes.items.find((item) => item.id === DEPLOY_LOCAL_SANDBOX_ID);
      if (!deploy) {
        setSandboxAvailable(false);
        setModules([]);
        return;
      }
      setSandboxAvailable(true);
      let directories: SandboxEntry[] = [];
      try {
        const spaceChildren = await port.listChildren({
          sandboxId: deploy.id,
          parentPath: DEPLOY_SPACE_MODULE_PARENT,
          pageSize: 200,
        });
        directories = spaceChildren.items.filter((entry) => entry.kind === "directory");
      } catch {
        directories = [];
      }
      if (directories.length === 0) {
        const rootChildren = await port.listChildren({
          sandboxId: deploy.id,
          parentPath: "",
          pageSize: 200,
        });
        directories = rootChildren.items.filter((entry) => entry.kind === "directory");
      }
      setModules(directories);
    } catch {
      setModulesError(t("modules.error"));
      setModules([]);
    } finally {
      setModulesBusy(false);
    }
  }, [port, t]);

  useEffect(() => {
    void loadModules();
  }, [loadModules]);

  const browserHint = useMemo(() => {
    if (!browserTarget) return "";
    if (browserTarget.kind === "module") return t("browser.hint.module", { name: browserTarget.name });
    if (browserTarget.kind === "node") return t("browser.hint.node", { name: browserTarget.label });
    return t("browser.hint.root");
  }, [browserTarget, t]);

  const initialLogicalPath = useMemo(() => {
    if (!browserTarget) return "";
    if (browserTarget.kind === "module") return browserTarget.logicalPath;
    return "";
  }, [browserTarget]);

  const navigationKey = useMemo(() => {
    if (!browserTarget) return "closed";
    if (browserTarget.kind === "module") return `module:${browserTarget.logicalPath}`;
    if (browserTarget.kind === "node") return `node:${browserTarget.id}`;
    return "root";
  }, [browserTarget]);

  return (
    <section className="local-projects-page resource-page">
      <header className="page-header">
        <div>
          <span className="eyebrow">{t("page.eyebrow")}</span>
          <h1>{t("page.title")}</h1>
          <p>{t("page.description")}</p>
        </div>
        <button
          className="icon-button"
          type="button"
          disabled={modulesBusy || !port}
          title={t("modules.refresh")}
          onClick={() => void loadModules()}
        >
          <RefreshCw size={18} />
        </button>
      </header>

      {!port && (
        <div className="error-banner" role="alert">
          {t("browser.missingPort")}
        </div>
      )}
      {port && !sandboxAvailable && (
        <div className="error-banner" role="alert">
          {t("sandbox.missing")}
        </div>
      )}
      {modulesError && (
        <div className="error-banner" role="alert">
          {modulesError}
        </div>
      )}

      <div className="local-projects-grid">
        <section className="local-projects-panel" aria-labelledby="local-projects-modules">
          <header>
            <FolderTree size={16} />
            <h2 id="local-projects-modules">{t("section.modules")}</h2>
          </header>
          <ul className="local-projects-list">
            {modules.map((entry) => (
              <li key={entry.id}>
                <button
                  type="button"
                  className="local-projects-item"
                  onClick={() =>
                    setBrowserTarget({
                      kind: "module",
                      name: entry.name,
                      logicalPath: entry.logicalPath,
                    })
                  }
                >
                  <strong>{entry.name}</strong>
                  <span>{t("modules.open")}</span>
                </button>
              </li>
            ))}
          </ul>
          {!modulesBusy && modules.length === 0 && sandboxAvailable && (
            <div className="empty-state">{t("modules.empty")}</div>
          )}
        </section>

        <section className="local-projects-panel" aria-labelledby="local-projects-nodes">
          <header>
            <Server size={16} />
            <h2 id="local-projects-nodes">{t("section.nodes")}</h2>
          </header>
          <ul className="local-projects-list">
            {LOCAL_DEPLOY_NODES.map((node) => (
              <li key={node.id}>
                <button
                  type="button"
                  className="local-projects-item"
                  onClick={() =>
                    setBrowserTarget({
                      kind: "node",
                      id: node.id,
                      label: t(node.labelKey),
                    })
                  }
                >
                  <span>
                    <strong>{t(node.labelKey)}</strong>
                    <small>{t(node.descriptionKey)}</small>
                  </span>
                  <span>{t("nodes.open")}</span>
                </button>
              </li>
            ))}
          </ul>
        </section>
      </div>

      {browserTarget && port && (
        <section className="local-projects-browser" aria-labelledby="local-projects-browser">
          <header>
            <div>
              <h2 id="local-projects-browser">{t("section.browser")}</h2>
              <p>{browserHint}</p>
            </div>
            <button
              className="icon-button"
              type="button"
              title={t("browser.close")}
              onClick={() => setBrowserTarget(null)}
            >
              <X size={18} />
            </button>
          </header>
          <SandboxExplorerView
            key={navigationKey}
            mode="manage"
            port={port}
            preferredSandboxId={DEPLOY_LOCAL_SANDBOX_ID}
            initialLogicalPath={initialLogicalPath}
            navigationKey={navigationKey}
            className="local-projects-explorer"
          />
        </section>
      )}
    </section>
  );
}
