/**
 * Console-facing apps page: lists tenant deploy_app records and opens the
 * CreateDeployAppDialog from a "Publish" command.
 *
 * Clients arrive as props (no console-core context dependency), so the same
 * page can be embedded by any host that can construct the two generated
 * clients — the deployments console shell and the BirdCoder plugin alike.
 */
import { useEffect, useMemo, useState } from "react";
import type { AppResponse, SdkworkDeployAppClient } from "@sdkwork/deployments-app-sdk";
import type { SdkworkDriveAppClient } from "@sdkwork/drive-app-sdk";
import type { DeploymentsLocale } from "@sdkwork/deployments-pc-commons";
import { createDeployAppPublishingService } from "../service/deploy-app-publishing.ts";
import { CreateDeployAppDialog } from "./CreateDeployAppDialog.tsx";
import "./create-deploy-app.module.css";

export interface PublishingAppsPageProps {
  readonly deployClient: SdkworkDeployAppClient
  readonly driveClient: SdkworkDriveAppClient
  readonly locale: DeploymentsLocale
  /** Host directory-picker port (optional; falls back to manual path input). */
  readonly pickDirectory?: (current: string | undefined) => Promise<string | undefined>
}

export function PublishingAppsPage({ deployClient, driveClient, locale, pickDirectory }: PublishingAppsPageProps) {
  const service = useMemo(
    () => createDeployAppPublishingService({ deployClient, driveClient }),
    [deployClient, driveClient],
  )
  const [apps, setApps] = useState<AppResponse[]>([])
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string>()
  const [createOpen, setCreateOpen] = useState(false)
  const [refresh, setRefresh] = useState(0)

  useEffect(() => {
    let active = true
    setBusy(true)
    setError(undefined)
    void service.listApps({ page: 1, pageSize: 50 }).then((result) => {
      if (active) setApps(result.items)
    }).catch((cause) => {
      if (active) setError(cause instanceof Error ? cause.message : String(cause))
    }).finally(() => {
      if (active) setBusy(false)
    })
    return () => { active = false }
  }, [refresh, service])

  return (
    <section className="resource-page publishing-apps-page">
      <header className="page-header">
        <div>
          <span className="eyebrow">apps</span>
          <h1>Apps</h1>
          <p>Create and publish deploy_app applications</p>
        </div>
        <div className="actions">
          <button type="button" className="command-button" disabled={busy} onClick={() => { setRefresh((value) => value + 1) }}>
            Refresh
          </button>
          <button type="button" className="command-button" onClick={() => { setCreateOpen(true) }}>
            + Publish application
          </button>
        </div>
      </header>
      {error && <div className="error-banner" role="alert">{error}</div>}
      <div className="table-frame" aria-busy={busy}>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Slug</th>
              <th>Kind</th>
              <th>Status</th>
              <th>Platform targets</th>
              <th>Version</th>
              <th>Updated</th>
            </tr>
          </thead>
          <tbody>
            {apps.map((app) => (
              <tr key={app.id}>
                <td><strong>{app.name}</strong></td>
                <td>{app.slug}</td>
                <td>{app.appKind}</td>
                <td><span className={`status-badge status-${app.appStatus.toLowerCase()}`}>{app.appStatus}</span></td>
                <td>{app.platformTargetCount ?? "-"}</td>
                <td>{app.latestReleaseTag ?? "-"}</td>
                <td>{new Date(app.updatedAt).toLocaleString(locale)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {!busy && apps.length === 0 && <div className="empty-state">No applications yet. Publish the first one.</div>}
      </div>
      {createOpen && (
        <CreateDeployAppDialog
          deployClient={deployClient}
          driveClient={driveClient}
          locale={locale}
          pickDirectory={pickDirectory}
          onClose={() => { setCreateOpen(false) }}
          onPublished={() => {
            setCreateOpen(false)
            setRefresh((value) => value + 1)
          }}
        />
      )}
    </section>
  )
}
