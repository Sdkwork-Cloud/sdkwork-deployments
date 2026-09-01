import type { DeploymentsLocale, DeploymentsResourcePageProps } from "@sdkwork/deployments-pc-commons";
import {
  type CertificateResponse,
  type DomainHostnameResponse,
  type DomainVerifyResponse,
  type DomainZoneResponse,
  type PageInfo,
  useDeploymentsDeliveryService,
} from "@sdkwork/deployments-pc-console-core";
import {
  ArrowLeft,
  BadgeCheck,
  ChevronLeft,
  ChevronRight,
  CirclePause,
  CirclePlay,
  Clipboard,
  ExternalLink,
  FileKey2,
  Globe2,
  Pencil,
  Plus,
  RefreshCw,
  RotateCw,
  Search,
  ShieldCheck,
  Trash2,
  X,
} from "lucide-react";
import {
  useEffect,
  useState,
  type FormEvent,
  type ReactNode,
} from "react";
import { Link, Navigate, Route, Routes, useParams, useSearchParams } from "react-router-dom";

import { deliveryText, type DeliveryMessageKey } from "./i18n.ts";

type Translator = (key: DeliveryMessageKey, values?: Record<string, string | number>) => string;
type ZoneDialog =
  | { kind: "create" }
  | { kind: "edit"; zone: DomainZoneResponse }
  | { kind: "status"; zone: DomainZoneResponse }
  | { kind: "delete"; zone: DomainZoneResponse };

export function DomainManagementPage({ locale }: DeploymentsResourcePageProps) {
  return <Routes>
    <Route index element={<DomainZoneList locale={locale} />} />
    <Route path=":zoneId" element={<DomainHostnameList locale={locale} />} />
    <Route path="*" element={<Navigate to="/console/domains" replace />} />
  </Routes>;
}

function DomainZoneList({ locale }: { locale: DeploymentsLocale }) {
  const service = useDeploymentsDeliveryService();
  const t = translator(locale);
  const [zones, setZones] = useState<DomainZoneResponse[]>([]);
  const [pageInfo, setPageInfo] = useState<PageInfo>({ mode: "offset", page: 1, pageSize: 20, hasMore: false });
  const [page, setPage] = useState(1);
  const [searchDraft, setSearchDraft] = useState("");
  const [keyword, setKeyword] = useState("");
  const [status, setStatus] = useState<"ALL" | "ACTIVE" | "PAUSED">("ALL");
  const [refreshVersion, setRefreshVersion] = useState(0);
  const [dialog, setDialog] = useState<ZoneDialog>();
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();

  useEffect(() => {
    let active = true;
    setBusy(true);
    setError(undefined);
    void service.listDomainZones({
      page,
      pageSize: 20,
      keyword: keyword || undefined,
      status: status === "ALL" ? undefined : status,
    }).then((result) => {
      if (!active) return;
      setZones(result.items);
      setPageInfo(result.pageInfo);
    }).catch((cause) => {
      if (active) setError(errorText(cause));
    }).finally(() => {
      if (active) setBusy(false);
    });
    return () => { active = false; };
  }, [keyword, page, refreshVersion, service, status]);

  const reload = () => setRefreshVersion((value) => value + 1);
  const closeAndReload = () => { setDialog(undefined); reload(); };

  return <section className="resource-page domain-page">
    <div className="resource-commandbar">
      <div className="resource-identity"><h1>{t("domainsTitle")}</h1></div>
      <div className="resource-query">
        <form className="search-box" onSubmit={(event) => { event.preventDefault(); setPage(1); setKeyword(searchDraft.trim()); }}>
          <Search size={16} /><input aria-label={t("search")} value={searchDraft} onChange={(event) => setSearchDraft(event.target.value)} placeholder={t("search")} />
        </form>
        <div className="segmented-control" aria-label={t("status")}>
          {(["ALL", "ACTIVE", "PAUSED"] as const).map((value) => <button key={value} type="button" aria-pressed={status === value} onClick={() => { setPage(1); setStatus(value); }}>{value === "ALL" ? t("all") : value === "ACTIVE" ? t("active") : t("paused")}</button>)}
        </div>
      </div>
      <div className="actions">
        <button className="icon-button" type="button" disabled={busy} title={t("refresh")} onClick={reload}><RefreshCw size={17} /></button>
        <button className="command-button" type="button" onClick={() => setDialog({ kind: "create" })}><Plus size={16} />{t("defineRoot")}</button>
      </div>
    </div>
    {error && <ErrorBanner message={error} t={t} />}
    <div className="table-frame domain-table-frame" aria-busy={busy}>
      <table className="domain-table"><thead><tr>
        <th>{t("rootDomain")}</th><th>{t("status")}</th><th>{t("hostnames")}</th><th>{t("certificates")}</th><th>{t("appBindings")}</th><th>{t("updated")}</th><th className="operations-column">{t("operations")}</th>
      </tr></thead><tbody>{zones.map((zone) => {
        // hostnameCount includes the apex hostname row every zone owns, so
        // only counts above 1 represent user-added subdomains that block
        // zone deletion.
        const deleteBlocked = Number(zone.hostnameCount) > 1 || Number(zone.certificateCount) > 0 || Number(zone.bindingCount) > 0;
        return <tr key={zone.id}>
          <td><Link className="primary-cell-link" to={zone.id}><Globe2 size={17} /><span><strong>{zone.apexHostname}</strong><small>{zone.displayName || zone.dnsProvider || "-"}</small></span></Link></td>
          <td><StatusBadge value={zone.status} t={t} /></td>
          <td><strong>{zone.hostnameCount}</strong><small className="cell-subtitle">{t("verifiedSummary", { verified: zone.verifiedHostnameCount, total: zone.hostnameCount })}</small></td>
          <td>{zone.certificateCount}</td><td>{zone.bindingCount}</td><td>{formatDate(zone.updatedAt, locale)}</td>
          <td><div className="row-actions">
            <Link className="table-action" to={zone.id} title={t("open")} aria-label={`${t("open")} ${zone.apexHostname}`}><ExternalLink size={16} /></Link>
            <button className="table-action" type="button" title={t("edit")} aria-label={`${t("edit")} ${zone.apexHostname}`} onClick={() => setDialog({ kind: "edit", zone })}><Pencil size={16} /></button>
            <button className="table-action" type="button" title={zone.status === "ACTIVE" ? t("pause") : t("resume")} aria-label={`${zone.status === "ACTIVE" ? t("pause") : t("resume")} ${zone.apexHostname}`} onClick={() => setDialog({ kind: "status", zone })}>{zone.status === "ACTIVE" ? <CirclePause size={16} /> : <CirclePlay size={16} />}</button>
            <button className="table-action danger-action" type="button" disabled={deleteBlocked} title={deleteBlocked ? t("deleteBlocked") : t("delete")} aria-label={`${t("delete")} ${zone.apexHostname}`} onClick={() => setDialog({ kind: "delete", zone })}><Trash2 size={16} /></button>
          </div></td>
        </tr>;
      })}</tbody></table>
      {!busy && zones.length === 0 && <div className="empty-state"><Globe2 size={24} />{t("noRootDomains")}</div>}
    </div>
    <Pagination page={page} pageInfo={pageInfo} busy={busy} setPage={setPage} t={t} />
    {dialog?.kind === "create" && <ZoneFormDialog t={t} close={() => setDialog(undefined)} submit={async (body) => { await service.createDomainZone(toDomainZoneRequestBody(body)); closeAndReload(); }} />}
    {dialog?.kind === "edit" && <ZoneFormDialog t={t} zone={dialog.zone} close={() => setDialog(undefined)} submit={async (body) => { await service.updateDomainZone(dialog.zone.id, toDomainZoneRequestBody(body)); closeAndReload(); }} />}
    {dialog?.kind === "status" && <ConfirmDialog
      title={dialog.zone.status === "ACTIVE" ? t("pauseZoneTitle") : t("resumeZoneTitle")}
      message={dialog.zone.status === "ACTIVE" ? t("pauseZoneConfirm") : t("resumeZoneConfirm")}
      dangerous={dialog.zone.status === "ACTIVE"}
      t={t}
      close={() => setDialog(undefined)}
      submit={async () => { await service.updateDomainZone(dialog.zone.id, { status: dialog.zone.status === "ACTIVE" ? "PAUSED" : "ACTIVE" }); closeAndReload(); }}
    />}
    {dialog?.kind === "delete" && <ConfirmDialog title={t("deleteZoneTitle")} message={t("deleteZoneConfirm")} dangerous t={t} close={() => setDialog(undefined)} submit={async () => { await service.deleteDomainZone(dialog.zone.id); closeAndReload(); }} />}
  </section>;
}

/** Zone form payload: every optional member is a genuine "left blank" state. */
export interface DomainZoneFormBody {
  apexHostname: string;
  displayName?: string | undefined;
  dnsProvider?: string | undefined;
  providerZoneRef?: string | undefined;
}

/**
 * Map the form payload onto the generated create/update request.
 *
 * Generated request types declare their optionals as `field?: string`, so an
 * explicit `undefined` is rejected under `exactOptionalPropertyTypes`; hand
 * editing generated output is forbidden, so blank fields are omitted here. The
 * wire treats an absent key and an explicit `undefined` identically.
 */
export function toDomainZoneRequestBody(body: DomainZoneFormBody) {
  return {
    apexHostname: body.apexHostname,
    ...(body.displayName === undefined ? {} : { displayName: body.displayName }),
    ...(body.dnsProvider === undefined ? {} : { dnsProvider: body.dnsProvider }),
    ...(body.providerZoneRef === undefined ? {} : { providerZoneRef: body.providerZoneRef }),
  };
}

function ZoneFormDialog({ close, submit, t, zone }: {
  close(): void;
  submit(body: DomainZoneFormBody): Promise<void>;
  t: Translator;
  zone?: DomainZoneResponse | undefined;
}) {
  const [apexHostname, setApexHostname] = useState(zone?.apexHostname ?? "");
  const [displayName, setDisplayName] = useState(zone?.displayName ?? "");
  const [dnsProvider, setDnsProvider] = useState(zone?.dnsProvider ?? "");
  const [providerZoneRef, setProviderZoneRef] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (!apexHostname.trim()) return;
    setBusy(true); setError(undefined);
    try {
      await submit({
        apexHostname: apexHostname.trim().toLowerCase(),
        displayName: optionalText(displayName),
        dnsProvider: optionalText(dnsProvider),
        providerZoneRef: optionalText(providerZoneRef),
      });
    } catch (cause) { setError(errorText(cause)); setBusy(false); }
  }
  return <Modal close={close} closeLabel={t("close")} title={zone ? t("editRootTitle") : t("createRootTitle")}>
    <form onSubmit={(event) => void onSubmit(event)}>
      <div className="form-grid">
        <label><span>{t("apexHostname")}</span><input autoFocus={!zone} required disabled={Boolean(zone)} value={apexHostname} onChange={(event) => setApexHostname(event.target.value)} placeholder="example.com" autoComplete="off" /><small className="form-hint">{t("apexHint")}</small></label>
        <label><span>{t("displayName")}</span><input value={displayName} onChange={(event) => setDisplayName(event.target.value)} autoComplete="off" /></label>
        <label><span>{t("dnsProvider")}</span><input value={dnsProvider} onChange={(event) => setDnsProvider(event.target.value)} placeholder="Aliyun DNS" autoComplete="off" /></label>
        <label><span>{t("providerZoneRef")}</span><input value={providerZoneRef} onChange={(event) => setProviderZoneRef(event.target.value)} autoComplete="off" /></label>
      </div>
      {error && <ErrorBanner message={error} t={t} />}
      <DialogFooter busy={busy} close={close} submitLabel={zone ? t("save") : t("create")} t={t} />
    </form>
  </Modal>;
}

function DomainHostnameList({ locale }: { locale: DeploymentsLocale }) {
  const { zoneId = "" } = useParams();
  const service = useDeploymentsDeliveryService();
  const t = translator(locale);
  const [zone, setZone] = useState<DomainZoneResponse>();
  const [hostnames, setHostnames] = useState<DomainHostnameResponse[]>([]);
  const [pageInfo, setPageInfo] = useState<PageInfo>({ mode: "offset", page: 1, pageSize: 20, hasMore: false });
  const [page, setPage] = useState(1);
  const [refreshVersion, setRefreshVersion] = useState(0);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [createOpen, setCreateOpen] = useState(false);
  const [editTarget, setEditTarget] = useState<DomainHostnameResponse>();
  const [deleteTarget, setDeleteTarget] = useState<DomainHostnameResponse>();
  const [verification, setVerification] = useState<DomainVerifyResponse>();

  useEffect(() => {
    let active = true;
    setBusy(true); setError(undefined);
    void Promise.all([
      service.retrieveDomainZone(zoneId),
      service.listDomainHostnames(zoneId, { page, pageSize: 20 }),
    ]).then(([zoneResult, hostnameResult]) => {
      if (!active) return;
      setZone(zoneResult);
      setHostnames(hostnameResult.items);
      setPageInfo(hostnameResult.pageInfo);
    }).catch((cause) => { if (active) setError(errorText(cause)); }).finally(() => { if (active) setBusy(false); });
    return () => { active = false; };
  }, [page, refreshVersion, service, zoneId]);

  const reload = () => setRefreshVersion((value) => value + 1);
  if (!zone && busy) return <section className="resource-page"><div className="empty-state">{t("loading")}</div></section>;

  return <section className="resource-page domain-page">
    <Link className="back-link" to="/console/domains"><ArrowLeft size={16} />{t("backDomains")}</Link>
    <div className="resource-commandbar">
      <div className="resource-identity"><h1>{zone?.apexHostname ?? "-"}</h1></div>
      <div className="actions"><button className="icon-button" type="button" disabled={busy} title={t("refresh")} onClick={reload}><RefreshCw size={17} /></button><button className="command-button" type="button" onClick={() => setCreateOpen(true)}><Plus size={16} />{t("addHostname")}</button></div>
    </div>
    {zone && <div className="metric-strip">
      <Metric label={t("verification")} value={t("verifiedSummary", { verified: zone.verifiedHostnameCount, total: zone.hostnameCount })} />
      <Metric label={t("certificates")} value={zone.certificateCount} />
      <Metric label={t("appBindings")} value={zone.bindingCount} />
      <Metric label={t("status")} value={zone.status === "ACTIVE" ? t("active") : t("paused")} />
    </div>}
    {error && <ErrorBanner message={error} t={t} />}
    <div className="table-frame domain-table-frame" aria-busy={busy}><table className="domain-table"><thead><tr>
      <th>{t("hostname")}</th><th>{t("type")}</th><th>{t("verification")}</th><th>{t("certificateCoverage")}</th><th>{t("appBindings")}</th><th>{t("updated")}</th><th className="operations-column">{t("operations")}</th>
    </tr></thead><tbody>{hostnames.map((hostname) => {
      // The apex hostname row is owned by the zone itself and can only be
      // removed together with the whole zone; hostnames with active
      // certificate coverage or application bindings keep their name and
      // cannot be renamed or deleted independently.
      const isApex = hostname.hostname === zone?.apexHostname;
      const related = Number(hostname.certificateCount) > 0 || Number(hostname.bindingCount) > 0;
      const renameBlocked = related || isApex;
      return <tr key={hostname.id}>
        <td><span className="hostname-cell"><Globe2 size={16} /><strong>{hostname.hostname}</strong></span></td>
        <td>{hostname.hostnameType === "WILDCARD" ? t("wildcard") : t("exact")}</td>
        <td><StatusBadge value={hostname.verificationStatus} t={t} /></td>
        <td>{hostname.certificateCount}</td><td>{hostname.bindingCount}</td><td>{formatDate(hostname.updatedAt, locale)}</td>
        <td><div className="row-actions">
          <button className="table-action" type="button" disabled={hostname.verificationStatus === "VERIFIED"} title={t("verify")} aria-label={`${t("verify")} ${hostname.hostname}`} onClick={() => { setBusy(true); void service.verifyDomainHostname(zoneId, hostname.id).then((result) => { setVerification(result); reload(); }).catch((cause) => setError(errorText(cause))).finally(() => setBusy(false)); }}><ShieldCheck size={16} /></button>
          {hostname.verificationStatus === "VERIFIED" ? <Link className="table-action" to={`/console/certificates?zoneId=${encodeURIComponent(zoneId)}&domainId=${encodeURIComponent(hostname.id)}&hostname=${encodeURIComponent(hostname.hostname)}`} title={t("requestCertificate")} aria-label={`${t("requestCertificate")} ${hostname.hostname}`}><FileKey2 size={16} /></Link> : <button className="table-action" type="button" disabled title={t("requestCertificate")} aria-label={`${t("requestCertificate")} ${hostname.hostname}`}><FileKey2 size={16} /></button>}
          <button className="table-action" type="button" disabled={renameBlocked} title={isApex ? t("apexEditBlocked") : related ? t("renameBlocked") : t("editHostname")} aria-label={`${t("editHostname")} ${hostname.hostname}`} onClick={() => setEditTarget(hostname)}><Pencil size={16} /></button>
          <button className="table-action danger-action" type="button" disabled={related || isApex} title={isApex ? t("apexDeleteBlocked") : related ? t("hostnameBlocked") : t("delete")} aria-label={`${t("delete")} ${hostname.hostname}`} onClick={() => setDeleteTarget(hostname)}><Trash2 size={16} /></button>
        </div></td>
      </tr>;
    })}</tbody></table>{!busy && hostnames.length === 0 && <div className="empty-state"><Globe2 size={24} />{t("noHostnames")}</div>}</div>
    <Pagination page={page} pageInfo={pageInfo} busy={busy} setPage={setPage} t={t} />
    {createOpen && <HostnameFormDialog t={t} close={() => setCreateOpen(false)} submit={async (relativeName) => { await service.createDomainHostname(zoneId, { relativeName }); setCreateOpen(false); reload(); }} />}
    {editTarget && <HostnameFormDialog hostname={editTarget} t={t} close={() => setEditTarget(undefined)} submit={async (relativeName) => { await service.updateDomainHostname(zoneId, editTarget.id, { relativeName }); setEditTarget(undefined); reload(); }} />}
    {deleteTarget && <ConfirmDialog title={t("deleteHostnameTitle")} message={t("deleteHostnameConfirm")} dangerous t={t} close={() => setDeleteTarget(undefined)} submit={async () => { await service.deleteDomainHostname(zoneId, deleteTarget.id); setDeleteTarget(undefined); reload(); }} />}
    {verification && <VerificationDialog result={verification} t={t} close={() => setVerification(undefined)} />}
  </section>;
}

function HostnameFormDialog({ close, hostname, submit, t }: { close(): void; hostname?: DomainHostnameResponse; submit(relativeName: string): Promise<void>; t: Translator }) {
  const [relativeName, setRelativeName] = useState(hostname?.relativeName ?? "");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (!relativeName.trim()) return;
    setBusy(true); setError(undefined);
    try { await submit(relativeName.trim().toLowerCase()); } catch (cause) { setError(errorText(cause)); setBusy(false); }
  }
  return <Modal close={close} closeLabel={t("close")} title={hostname ? t("editHostnameTitle") : t("addHostnameTitle")}><form onSubmit={(event) => void onSubmit(event)}>
    <div className="form-grid single-column"><label><span>{t("relativeName")}</span><input autoFocus required value={relativeName} onChange={(event) => setRelativeName(event.target.value)} placeholder="@ / www / api.eu / *" autoComplete="off" /><small className="form-hint">{hostname ? t("renameHint") : t("relativeNameHint")}</small></label></div>
    {error && <ErrorBanner message={error} t={t} />}<DialogFooter busy={busy} close={close} submitLabel={hostname ? t("save") : t("create")} t={t} />
  </form></Modal>;
}

function VerificationDialog({ close, result, t }: { close(): void; result: DomainVerifyResponse; t: Translator }) {
  const [copied, setCopied] = useState<string>();
  async function copy(name: string, value: string) {
    try { await navigator.clipboard.writeText(value); setCopied(name); } catch { setCopied(undefined); }
  }
  return <Modal close={close} closeLabel={t("close")} title={t("verificationTitle")}>
    <div className={result.verified ? "verification-success" : "verification-pending"}><BadgeCheck size={19} />{result.verified ? t("verificationComplete") : t("verificationInstructions")}</div>
    {result.recordName && <CopyField label={t("recordName")} value={result.recordName} copied={copied === "name"} copy={() => void copy("name", result.recordName!)} t={t} />}
    {result.token && <CopyField label={t("recordValue")} value={result.token} copied={copied === "token"} copy={() => void copy("token", result.token!)} t={t} />}
    {result.expiresAt && <div className="verification-expiry"><span>{t("expiresAt")}</span><strong>{result.expiresAt}</strong></div>}
    <footer className="dialog-footer"><button className="secondary-button" type="button" onClick={close}>{t("close")}</button></footer>
  </Modal>;
}

function CopyField({ copied, copy, label, t, value }: { copied: boolean; copy(): void; label: string; t: Translator; value: string }) {
  return <div className="copy-field"><span>{label}</span><code>{value}</code><button className="table-action" type="button" title={copied ? t("copied") : t("copy")} onClick={copy}><Clipboard size={16} /></button></div>;
}

export function CertificateManagementPage({ locale }: DeploymentsResourcePageProps) {
  const service = useDeploymentsDeliveryService();
  const t = translator(locale);
  const [searchParams, setSearchParams] = useSearchParams();
  const initialDomainId = searchParams.get("domainId") ?? undefined;
  const initialHostname = searchParams.get("hostname") ?? initialDomainId;
  const initialZoneId = searchParams.get("zoneId") ?? undefined;
  const [certificates, setCertificates] = useState<CertificateResponse[]>([]);
  const [pageInfo, setPageInfo] = useState<PageInfo>({ mode: "offset", page: 1, pageSize: 20, hasMore: false });
  const [page, setPage] = useState(1);
  const [refreshVersion, setRefreshVersion] = useState(0);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const [createOpen, setCreateOpen] = useState(Boolean(initialDomainId));
  const [renewTarget, setRenewTarget] = useState<CertificateResponse>();
  const [revokeTarget, setRevokeTarget] = useState<CertificateResponse>();

  useEffect(() => {
    if (initialDomainId) setCreateOpen(true);
  }, [initialDomainId]);

  useEffect(() => {
    let active = true;
    setBusy(true); setError(undefined);
    void service.listCertificates({ page, pageSize: 20 }).then((result) => {
      if (!active) return;
      setCertificates(result.items); setPageInfo(result.pageInfo);
    }).catch((cause) => { if (active) setError(errorText(cause)); }).finally(() => { if (active) setBusy(false); });
    return () => { active = false; };
  }, [page, refreshVersion, service]);

  const reload = () => setRefreshVersion((value) => value + 1);
  const closeCreate = () => { setCreateOpen(false); setSearchParams({}, { replace: true }); };
  return <section className="resource-page domain-page">
    <div className="resource-commandbar">
      <div className="resource-identity"><h1>{t("certificatesTitle")}</h1></div>
      <div className="actions"><button className="icon-button" type="button" disabled={busy} title={t("refresh")} onClick={reload}><RefreshCw size={17} /></button><button className="command-button" type="button" onClick={() => setCreateOpen(true)}><Plus size={16} />{t("requestCertificate")}</button></div>
    </div>
    {error && <ErrorBanner message={error} t={t} />}
    <div className="table-frame domain-table-frame certificate-table-frame" aria-busy={busy}><table className="domain-table"><thead><tr>
      <th>{t("certificates")}</th><th>{t("identifiers")}</th><th>{t("status")}</th><th>{t("keyAlgorithm")}</th><th>{t("caProfile")}</th><th>{t("expiration")}</th><th>{t("renewal")}</th><th className="operations-column">{t("operations")}</th>
    </tr></thead><tbody>{certificates.map((certificate) => <tr key={certificate.id}>
      <td><span className="certificate-name"><FileKey2 size={17} /><strong>{certificate.certName}</strong></span></td>
      <td><div className="identifier-list">{certificate.identifiers.map((identifier) => <span key={identifier}>{identifier}</span>)}</div></td>
      <td><StatusBadge value={certificate.status} t={t} /></td><td>{certificate.preferredKeyAlgorithm}</td><td>{certificate.caProfile}</td><td>{certificate.notAfter ? formatDate(certificate.notAfter, locale) : "-"}</td><td>{certificate.renewalStatus}</td>
      <td><div className="row-actions"><button className="table-action" type="button" disabled={certificate.certificateSource !== "MANAGED" || certificate.status === "REVOKED"} title={t("renew")} aria-label={`${t("renew")} ${certificate.certName}`} onClick={() => setRenewTarget(certificate)}><RotateCw size={16} /></button><button className="table-action danger-action" type="button" disabled={certificate.status === "REVOKED"} title={t("revoke")} aria-label={`${t("revoke")} ${certificate.certName}`} onClick={() => setRevokeTarget(certificate)}><Trash2 size={16} /></button></div></td>
    </tr>)}</tbody></table>{!busy && certificates.length === 0 && <div className="empty-state"><FileKey2 size={24} />{t("noCertificates")}</div>}</div>
    <Pagination page={page} pageInfo={pageInfo} busy={busy} setPage={setPage} t={t} />
    {createOpen && <CertificateFormDialog initialDomain={initialDomainId ? { id: initialDomainId, hostname: initialHostname ?? initialDomainId, zoneId: initialZoneId } : undefined} t={t} close={closeCreate} submit={async (body) => { await service.createCertificate(body); closeCreate(); reload(); }} />}
    {renewTarget && <ConfirmDialog title={t("renew")} message={renewTarget.certName} t={t} close={() => setRenewTarget(undefined)} submit={async () => { await service.renewCertificate(renewTarget.id); setRenewTarget(undefined); reload(); }} />}
    {revokeTarget && <ConfirmDialog title={t("revokeCertificateTitle")} message={t("revokeCertificateConfirm")} dangerous t={t} close={() => setRevokeTarget(undefined)} submit={async () => { await service.deleteCertificate(revokeTarget.id); setRevokeTarget(undefined); reload(); }} />}
  </section>;
}

function CertificateFormDialog({ close, initialDomain, submit, t }: {
  close(): void;
  initialDomain?: { id: string; hostname: string; zoneId?: string | undefined } | undefined;
  submit(body: { certName: string; domainIds: string[]; caProfile: "LETS_ENCRYPT_STAGING" | "LETS_ENCRYPT_PRODUCTION"; preferredKeyAlgorithm: "RSA" | "ECDSA" }): Promise<void>;
  t: Translator;
}) {
  const service = useDeploymentsDeliveryService();
  const [certName, setCertName] = useState(initialDomain ? `${initialDomain.hostname} TLS` : "");
  const [caProfile, setCaProfile] = useState<"LETS_ENCRYPT_STAGING" | "LETS_ENCRYPT_PRODUCTION">("LETS_ENCRYPT_PRODUCTION");
  const [algorithm, setAlgorithm] = useState<"RSA" | "ECDSA">("RSA");
  const [zoneSearchDraft, setZoneSearchDraft] = useState("");
  const [zoneSearch, setZoneSearch] = useState("");
  const [zones, setZones] = useState<DomainZoneResponse[]>([]);
  const [zoneId, setZoneId] = useState(initialDomain?.zoneId ?? "");
  const [hostnames, setHostnames] = useState<DomainHostnameResponse[]>([]);
  const [hostnamePage, setHostnamePage] = useState(1);
  const [hostnamePageInfo, setHostnamePageInfo] = useState<PageInfo>({ mode: "offset", page: 1, pageSize: 50, hasMore: false });
  const [selected, setSelected] = useState<Map<string, string>>(() => new Map(initialDomain ? [[initialDomain.id, initialDomain.hostname]] : []));
  const [busy, setBusy] = useState(false);
  const [loadingOptions, setLoadingOptions] = useState(false);
  const [error, setError] = useState<string>();

  useEffect(() => {
    let active = true;
    setLoadingOptions(true);
    void service.listDomainZones({ page: 1, pageSize: 50, status: "ACTIVE", keyword: zoneSearch.trim() || undefined }).then((result) => {
      if (!active) return;
      setZones(result.items);
      if (!zoneId && result.items[0]) setZoneId(result.items[0].id);
    }).catch((cause) => { if (active) setError(errorText(cause)); }).finally(() => { if (active) setLoadingOptions(false); });
    return () => { active = false; };
  }, [service, zoneSearch]);

  useEffect(() => {
    if (!zoneId) { setHostnames([]); return; }
    let active = true;
    setLoadingOptions(true);
    void service.listDomainHostnames(zoneId, { page: hostnamePage, pageSize: 50 }).then((result) => {
      if (!active) return;
      setHostnames(result.items); setHostnamePageInfo(result.pageInfo);
    }).catch((cause) => { if (active) setError(errorText(cause)); }).finally(() => { if (active) setLoadingOptions(false); });
    return () => { active = false; };
  }, [hostnamePage, service, zoneId]);

  const toggle = (hostname: DomainHostnameResponse) => setSelected((current) => {
    const next = new Map(current);
    if (next.has(hostname.id)) next.delete(hostname.id); else next.set(hostname.id, hostname.hostname);
    return next;
  });
  async function onSubmit(event: FormEvent) {
    event.preventDefault();
    if (!certName.trim() || selected.size === 0) return;
    setBusy(true); setError(undefined);
    try { await submit({ certName: certName.trim(), domainIds: [...selected.keys()], caProfile, preferredKeyAlgorithm: algorithm }); }
    catch (cause) { setError(errorText(cause)); setBusy(false); }
  }
  return <Modal close={close} closeLabel={t("close")} title={t("requestCertificateTitle")} wide><form onSubmit={(event) => void onSubmit(event)}>
    <div className="form-grid certificate-form-grid">
      <label><span>{t("certificates")}</span><input autoFocus required value={certName} onChange={(event) => setCertName(event.target.value)} autoComplete="off" /></label>
      <label><span>{t("caProfile")}</span><select value={caProfile} onChange={(event) => setCaProfile(event.target.value as typeof caProfile)}><option value="LETS_ENCRYPT_PRODUCTION">{t("production")}</option><option value="LETS_ENCRYPT_STAGING">{t("staging")}</option></select></label>
    </div>
    <fieldset className="form-fieldset"><legend>{t("keyAlgorithm")}</legend><div className="segmented-control algorithm-control">{(["RSA", "ECDSA"] as const).map((value) => <button key={value} type="button" aria-pressed={algorithm === value} onClick={() => setAlgorithm(value)}>{value === "RSA" ? t("rsa") : t("ecdsa")}</button>)}</div></fieldset>
    <fieldset className="form-fieldset domain-selector"><legend>{t("domainSelection")}</legend>
      <div className="search-box selector-search"><Search size={16} /><input value={zoneSearchDraft} onChange={(event) => setZoneSearchDraft(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") { event.preventDefault(); setZoneSearch(zoneSearchDraft.trim()); } }} placeholder={t("searchRoot")} aria-label={t("searchRoot")} /><button className="table-action" type="button" title={t("searchRoot")} onClick={() => setZoneSearch(zoneSearchDraft.trim())}><Search size={15} /></button></div>
      <label className="selector-zone"><span>{t("rootDomainSelect")}</span><select value={zoneId} onChange={(event) => { setZoneId(event.target.value); setHostnamePage(1); }}>{zones.map((zone) => <option key={zone.id} value={zone.id}>{zone.apexHostname}</option>)}</select></label>
      <div className="hostname-selector-list" aria-busy={loadingOptions}>{hostnames.map((hostname) => <label key={hostname.id} className={hostname.verificationStatus !== "VERIFIED" ? "disabled" : ""}><input type="checkbox" disabled={hostname.verificationStatus !== "VERIFIED"} checked={selected.has(hostname.id)} onChange={() => toggle(hostname)} /><span><strong>{hostname.hostname}</strong><small>{hostname.verificationStatus === "VERIFIED" ? t("verified") : t("pending")}</small></span></label>)}{!loadingOptions && hostnames.length === 0 && <div className="selector-empty">{t("noHostnames")}</div>}</div>
      <div className="selector-pagination"><button className="icon-button" type="button" disabled={hostnamePage <= 1 || loadingOptions} title={t("previous")} onClick={() => setHostnamePage((value) => Math.max(1, value - 1))}><ChevronLeft size={17} /></button><span>{t("page", { page: hostnamePage })}</span><button className="icon-button" type="button" disabled={!hostnamePageInfo.hasMore || loadingOptions} title={t("next")} onClick={() => setHostnamePage((value) => value + 1)}><ChevronRight size={17} /></button></div>
      <div className="selected-hostnames"><span>{t("selectedDomains")} ({selected.size})</span><div>{[...selected].map(([id, hostname]) => <span key={id}>{hostname}<button type="button" title={t("delete")} onClick={() => setSelected((current) => { const next = new Map(current); next.delete(id); return next; })}><X size={13} /></button></span>)}</div></div>
    </fieldset>
    {error && <ErrorBanner message={error} t={t} />}<DialogFooter busy={busy} disabled={selected.size === 0} close={close} submitLabel={t("requestCertificate")} t={t} />
  </form></Modal>;
}

function ConfirmDialog({ close, dangerous = false, message, submit, t, title }: { close(): void; dangerous?: boolean; message: string; submit(): Promise<void>; t: Translator; title: string }) {
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  async function run() {
    setBusy(true); setError(undefined);
    try { await submit(); } catch (cause) { setError(errorText(cause)); setBusy(false); }
  }
  return <Modal close={close} closeLabel={t("close")} title={title}><p className={dangerous ? "confirmation-message dangerous-confirmation" : "confirmation-message"}>{message}</p>{error && <ErrorBanner message={error} t={t} />}<footer className="dialog-footer"><button className="secondary-button" type="button" onClick={close}>{t("cancel")}</button><button className={dangerous ? "danger-button" : "command-button"} type="button" disabled={busy} onClick={() => void run()}>{t("confirm")}</button></footer></Modal>;
}

function Modal({ children, close, closeLabel, title, wide = false }: { children: ReactNode; close(): void; closeLabel: string; title: string; wide?: boolean }) {
  return <div className="dialog-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) close(); }}><div className={`dialog delivery-dialog${wide ? " delivery-dialog-wide" : ""}`} role="dialog" aria-modal="true" aria-labelledby="delivery-dialog-title"><header><h2 id="delivery-dialog-title">{title}</h2><button className="icon-button" type="button" title={closeLabel} onClick={close}><X size={18} /></button></header>{children}</div></div>;
}

function DialogFooter({ busy, close, disabled = false, submitLabel, t }: { busy: boolean; close(): void; disabled?: boolean; submitLabel: string; t: Translator }) {
  return <footer className="dialog-footer"><button className="secondary-button" type="button" onClick={close}>{t("cancel")}</button><button className="command-button" type="submit" disabled={busy || disabled}>{submitLabel}</button></footer>;
}

function Pagination({ busy, page, pageInfo, setPage, t }: { busy: boolean; page: number; pageInfo: PageInfo; setPage(value: number | ((current: number) => number)): void; t: Translator }) {
  const summary = pageInfo.totalItems ? t("total", { total: pageInfo.totalItems }) : t("page", { page: pageInfo.page ?? page });
  return <footer className="pagination"><span>{summary}</span><button className="icon-button" type="button" disabled={page <= 1 || busy} title={t("previous")} onClick={() => setPage((value) => Math.max(1, value - 1))}><ChevronLeft size={18} /></button><button className="icon-button" type="button" disabled={!pageInfo.hasMore || busy} title={t("next")} onClick={() => setPage((value) => value + 1)}><ChevronRight size={18} /></button></footer>;
}

function Metric({ label, value }: { label: string; value: string }) {
  return <div className="domain-metric"><span>{label}</span><strong>{value}</strong></div>;
}

function StatusBadge({ t, value }: { t: Translator; value: string }) {
  const labels: Partial<Record<string, DeliveryMessageKey>> = { ACTIVE: "active", PAUSED: "paused", PENDING: "pending", VERIFIED: "verified", FAILED: "failed", EXPIRED: "expired", ISSUING: "issuing" };
  return <span className={`status-badge status-${value.toLowerCase()}`}>{labels[value] ? t(labels[value]!) : value}</span>;
}

function ErrorBanner({ message, t }: { message?: string; t: Translator }) {
  return <div className="error-banner" role="alert">{message || t("error")}</div>;
}

function errorText(cause: unknown): string | undefined {
  return cause instanceof Error && cause.message ? cause.message : undefined;
}

function translator(locale: DeploymentsLocale): Translator {
  return (key, values) => deliveryText(locale, key, values);
}

function formatDate(value: string, locale: DeploymentsLocale): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : new Intl.DateTimeFormat(locale, { dateStyle: "medium", timeStyle: "short" }).format(date);
}

function optionalText(value: string): string | undefined {
  return value.trim() || undefined;
}
