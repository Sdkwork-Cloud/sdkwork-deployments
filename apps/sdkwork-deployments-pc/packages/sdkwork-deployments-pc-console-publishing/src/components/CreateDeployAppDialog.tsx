/**
 * CreateDeployAppDialog — 创建/发布 deploy_app 应用对话框。
 *
 * 需求覆盖：
 *   1. 目录选择（可更换；支持关联已存在 deploy_app 或创建新应用并填写名称）
 *   2. 应用类型（静态资源 / 小程序 / Flutter iOS / Flutter 安卓 / 原生 / 鸿蒙 / API）
 *   3. 多级分类级联
 *   4. 应用 icon 上传
 *   5. 封面图上传
 *   6. 截图与预览图（App Store 预览图规格：尺寸校验 + 每类 ≤10 张）
 *   7. 版本号（语义化校验）
 *   8. 应用描述
 *   9. release notes
 *
 * 持久化严格走 sdkwork-deployments 现有表结构：
 *   deploy_app（name/slug/app_kind/description/metadata）、
 *   deploy_app_platform_target、deploy_app.metadata(JSONB: category/media/version/releaseNotes)。
 *
 * 组件为纯 props 输入（两个生成式 client + locale + 目录选择端口），不依赖
 * console context，deployments 控制台与 BirdCoder 插件均可复用（高内聚低耦合）。
 */
import { useMemo, useRef, useState, type FormEvent, type ReactNode } from "react";
import type { AppKind, AppResponse, SdkworkDeployAppClient } from "@sdkwork/deployments-app-sdk";
import type { SdkworkDriveAppClient } from "@sdkwork/drive-app-sdk";
import type { DeploymentsLocale } from "@sdkwork/deployments-pc-commons";
import { publishingTranslator, type PublishingTranslator } from "../i18n.ts";
import {
  createDeployAppPublishingService,
  deriveAppSlug,
  isValidSemver,
  toDeployAppMediaRef,
  type CreateDeployAppInput,
  type DeployAppMediaGroup,
  type DeployAppPublishingService,
  type DeployAppTypeOption,
} from "../service/deploy-app-publishing.ts";
import { CategoryCascadeSelect } from "./CategoryCascadeSelect.tsx";
import {
  DeployAppMediaFields,
  type DeployAppMediaFiles,
} from "./DeployAppMediaFields.tsx";
import { DeployAppTypeSelect } from "./DeployAppTypeSelect.tsx";
import css from "./create-deploy-app.module.css";

export interface CreateDeployAppDialogProps {
  readonly deployClient: SdkworkDeployAppClient
  readonly driveClient: SdkworkDriveAppClient
  readonly locale: DeploymentsLocale
  /** 需求 1: 初始目录。 */
  readonly initialDirectory?: string
  /** 需求 1: 目录更换端口（宿主提供：浏览器目录选择 / 桌面原生选择器）。 */
  readonly pickDirectory?: (current: string | undefined) => Promise<string | undefined>
  /** 主题（驱动组件内建 CSS 变量切换；缺省浅色）。 */
  readonly theme?: "light" | "dark"
  readonly onClose: () => void
  readonly onPublished?: (result: { app: AppResponse; media: DeployAppMediaGroup }) => void
}

export interface DeployAppPublishResult {
  app: AppResponse
  media: DeployAppMediaGroup
}

const STEP_COUNT = 4

export function CreateDeployAppDialog({
  deployClient,
  driveClient,
  locale,
  initialDirectory,
  pickDirectory,
  theme = "light",
  onClose,
  onPublished,
}: CreateDeployAppDialogProps) {
  const t = useMemo(() => publishingTranslator(locale), [locale])
  const service = useMemo(
    () => createDeployAppPublishingService({ deployClient, driveClient }),
    [deployClient, driveClient],
  )

  const [step, setStep] = useState(1)
  const [directory, setDirectory] = useState<string | undefined>(initialDirectory)
  const [mode, setMode] = useState<"associate" | "create">("create")
  const [apps, setApps] = useState<AppResponse[]>([])
  const [appsSearch, setAppsSearch] = useState("")
  const [appsLoading, setAppsLoading] = useState(false)
  const [associateId, setAssociateId] = useState<string>()
  const [name, setName] = useState("")
  const [slug, setSlug] = useState("")
  const [type, setType] = useState<DeployAppTypeOption>()
  const [category, setCategory] = useState<CreateDeployAppInput["category"]>()
  const [media, setMedia] = useState<DeployAppMediaFiles>({ screenshots: {} })
  const [version, setVersion] = useState("1.0.0")
  const [description, setDescription] = useState("")
  const [releaseNotes, setReleaseNotes] = useState("")
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string>()
  const [uploadingLabel, setUploadingLabel] = useState<string>()
  const pickedAppsRef = useRef(false)

  const loadApps = async (keyword: string) => {
    setAppsLoading(true)
    setError(undefined)
    try {
      const result = await service.listApps({ page: 1, pageSize: 50, keyword: keyword.trim() || undefined })
      setApps(result.items)
    } catch (cause) {
      setError(errorText(cause, t))
    } finally {
      setAppsLoading(false)
    }
  }

  const searchApps = (event: FormEvent) => {
    event.preventDefault()
    void loadApps(appsSearch)
  }

  const changeDirectory = async () => {
    if (!pickDirectory) return
    try {
      const next = await pickDirectory(directory)
      if (next) setDirectory(next)
    } catch (cause) {
      setError(errorText(cause, t))
    }
  }

  const canNext = (): boolean => {
    if (step === 1) {
      if (mode === "associate") return Boolean(directory && associateId)
      return Boolean(directory && name.trim())
    }
    if (step === 2) return Boolean(type)
    if (step === 3) return true
    return isValidSemver(version)
  }

  const next = () => {
    if (!canNext()) {
      setError(t("publishRequiredFields"))
      return
    }
    setError(undefined)
    setStep((current) => Math.min(STEP_COUNT, current + 1))
  }

  const previous = () => {
    setError(undefined)
    setStep((current) => Math.max(1, current - 1))
  }

  const uploadMedia = async (appId: string, files: DeployAppMediaFiles): Promise<DeployAppMediaGroup> => {
    const upload = async (kind: "icon" | "cover" | "screenshot", file: File, targetKey?: string) => {
      setUploadingLabel(t("mediaUploading", { name: file.name }))
      const ref = await service.uploadMedia({
        kind,
        file,
        fileName: file.name,
        contentType: file.type || "application/octet-stream",
        targetKey,
      }, appId)
      return ref
    }

    const group: { icon?: DeployAppMediaGroup["icon"]; cover?: DeployAppMediaGroup["cover"]; screenshots: DeployAppMediaGroup["screenshots"] } = { screenshots: {} }
    if (files.icon) {
      group.icon = await upload("icon", files.icon)
    }
    if (files.cover) {
      group.cover = await upload("cover", files.cover)
    }
    for (const [targetKey, fileList] of Object.entries(files.screenshots)) {
      group.screenshots[targetKey] = []
      for (const file of fileList) {
        const ref = await upload("screenshot", file, targetKey)
        group.screenshots[targetKey] = [...group.screenshots[targetKey], ref]
      }
    }
    return group
  }

  const submit = async () => {
    if (!directory || !type || !isValidSemver(version)) {
      setError(t("publishRequiredFields"))
      return
    }
    setBusy(true)
    setError(undefined)
    setUploadingLabel(undefined)
    try {
      const base: CreateDeployAppInput = {
        sourceDirectory: directory,
        associateAppId: mode === "associate" ? associateId : undefined,
        name: mode === "create" ? name.trim() : undefined,
        slug: mode === "create" ? slug.trim() || deriveAppSlug(name) : undefined,
        type,
        category,
        version,
        description,
        releaseNotes,
      }

      // 1) 创建（或关联更新）deploy_app。
      const app = await service.createApp(base)

      // 2) 上传媒体（图标/封面/截图 → Drive），app 存在后以 appId 为资源锚点。
      let mediaGroup: DeployAppMediaGroup = { screenshots: {} }
      if (media.icon || media.cover || Object.keys(media.screenshots).length > 0) {
        mediaGroup = await uploadMedia(app.id, media)
        // 3) 回写 metadata.media。
        await deployClient.app.update(app.id, {
          metadata: { ...service.buildMetadata(base), media: mediaGroup },
        })
      }

      onPublished?.({ app, media: mediaGroup })
      setBusy(false)
    } catch (cause) {
      setError(errorText(cause, t))
      setBusy(false)
    }
  }

  return (
    <div className={css.publishDialog} data-theme={theme} role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget && !busy) onClose() }}>
      <div className={css.dialog} role="dialog" aria-modal="true" aria-label={t("publishApp")}>
        <header className={css.header}>
          <div className={css.headerText}>
            <h2>{t("publishApp")}</h2>
            <p>{t("publishAppDescription")}</p>
          </div>
          <button type="button" className={css.closeButton} title={t("close")} aria-label={t("close")} onClick={onClose}>
            ×
          </button>
        </header>

        <div className={css.steps} aria-label={t("step", { step: String(step), total: String(STEP_COUNT) })}>
          {Array.from({ length: STEP_COUNT }, (_, index) => (
            <span key={index} className={css.stepDot} data-active={index < step} />
          ))}
        </div>

        <div className={css.body}>
          {step === 1 && <StepApplication
            directory={directory}
            mode={mode}
            apps={apps}
            appsLoading={appsLoading}
            appsSearch={appsSearch}
            associateId={associateId}
            name={name}
            slug={slug}
            pickedRef={pickedAppsRef}
            t={t}
            onDirectoryChange={setDirectory}
            onChangeDirectory={() => { void changeDirectory() }}
            onModeChange={setMode}
            onAppsSearchChange={setAppsSearch}
            onSearchApps={searchApps}
            onLoadApps={(keyword) => { void loadApps(keyword) }}
            onAssociateChange={setAssociateId}
            onNameChange={setName}
            onSlugChange={setSlug}
          />}

          {step === 2 && (
            <div className={css.field}>
              <span className={css.fieldLabel}>{t("appType")}</span>
              <span className={css.fieldHint}>{t("appTypeHint")}</span>
              <DeployAppTypeSelect value={type} onChange={setType} t={t} />
              <span className={css.fieldLabel}>{t("category")}</span>
              <span className={css.fieldHint}>{t("categoryHint")}</span>
              <CategoryCascadeSelect appKind={type?.appKind as AppKind | undefined} value={category} onChange={setCategory} t={t} />
            </div>
          )}

          {step === 3 && <DeployAppMediaFields value={media} onChange={setMedia} t={t} />}

          {step === 4 && (
            <>
              <div className={css.field}>
                <span className={css.fieldLabel}>{t("version")}</span>
                <input
                  className={css.input}
                  value={version}
                  placeholder={t("versionPlaceholder")}
                  onChange={(event) => { setVersion(event.target.value) }}
                  aria-invalid={version.trim() !== "" && !isValidSemver(version)}
                />
                <span className={css.fieldHint}>{t("versionHint")}</span>
                {version.trim() !== "" && !isValidSemver(version) && (
                  <span className={css.errorBanner} role="alert">{t("versionError")}</span>
                )}
              </div>
              <div className={css.field}>
                <span className={css.fieldLabel}>{t("description")}</span>
                <textarea
                  className={css.textarea}
                  value={description}
                  placeholder={t("descriptionPlaceholder")}
                  onChange={(event) => { setDescription(event.target.value) }}
                />
              </div>
              <div className={css.field}>
                <span className={css.fieldLabel}>{t("releaseNotes")}</span>
                <textarea
                  className={css.textarea}
                  value={releaseNotes}
                  placeholder={t("releaseNotesPlaceholder")}
                  onChange={(event) => { setReleaseNotes(event.target.value) }}
                />
              </div>
            </>
          )}
        </div>

        <footer className={css.footer}>
          {error && <div className={css.errorBanner} role="alert">{error}</div>}
          {uploadingLabel && <div className={css.uploadingText}>{uploadingLabel}</div>}
          <div className={css.footerSpacer} />
          {step > 1 && (
            <button type="button" className={css.secondaryButton} disabled={busy} onClick={previous}>
              {t("previous")}
            </button>
          )}
          {step < STEP_COUNT
            ? (
              <button type="button" className={css.primaryButton} disabled={busy} onClick={next}>
                {t("next")}
              </button>
            )
            : (
              <button type="button" className={css.primaryButton} disabled={busy || !canNext()} onClick={() => { void submit() }}>
                {busy ? t("publishing") : t("publish")}
              </button>
            )}
        </footer>
      </div>
    </div>
  )
}

interface StepApplicationProps {
  directory: string | undefined
  mode: "associate" | "create"
  apps: readonly AppResponse[]
  appsLoading: boolean
  appsSearch: string
  associateId: string | undefined
  name: string
  slug: string
  pickedRef: { current: boolean }
  t: PublishingTranslator
  onDirectoryChange: (value: string) => void
  onChangeDirectory: () => void
  onModeChange: (mode: "associate" | "create") => void
  onAppsSearchChange: (value: string) => void
  onSearchApps: (event: FormEvent) => void
  onLoadApps: (keyword: string) => void
  onAssociateChange: (value: string) => void
  onNameChange: (value: string) => void
  onSlugChange: (value: string) => void
}

function StepApplication(props: StepApplicationProps) {
  const {
    directory, mode, apps, appsLoading, appsSearch, associateId, name, slug, pickedRef,
    t, onDirectoryChange, onChangeDirectory, onModeChange, onAppsSearchChange, onSearchApps,
    onLoadApps, onAssociateChange, onNameChange, onSlugChange,
  } = props

  return (
    <>
      <div className={css.field}>
        <span className={css.fieldLabel}>{t("sourceDirectory")}</span>
        <span className={css.fieldHint}>{t("sourceDirectoryHint")}</span>
        <div className={css.directoryRow}>
          <input
            className={css.input}
            value={directory ?? ""}
            data-empty={!directory}
            placeholder={t("noDirectory")}
            onChange={(event) => { onDirectoryChange(event.target.value) }}
          />
          <button type="button" className={css.secondaryButton} onClick={onChangeDirectory}>
            {t("changeDirectory")}
          </button>
        </div>
      </div>

      <div className={css.field}>
        <span className={css.fieldLabel}>{t("appAssociation")}</span>
        <div className={css.radioGroup}>
          <label className={css.radioRow} data-selected={mode === "create"}>
            <input
              type="radio"
              name="app-mode"
              checked={mode === "create"}
              onChange={() => { onModeChange("create") }}
            />
            <span className={css.radioLabel}>
              <strong>{t("createNew")}</strong>
              <small>{t("applicationNameHint")}</small>
            </span>
          </label>
          {mode === "create" && (
            <>
              <div className={css.field}>
                <input
                  className={css.input}
                  value={name}
                  placeholder={t("applicationNamePlaceholder")}
                  onChange={(event) => { onNameChange(event.target.value) }}
                />
              </div>
              <div className={css.field}>
                <input
                  className={css.input}
                  value={slug}
                  placeholder={t("appSlug")}
                  onChange={(event) => { onSlugChange(event.target.value) }}
                />
                <span className={css.fieldHint}>{t("appSlugHint")}</span>
              </div>
            </>
          )}

          <label className={css.radioRow} data-selected={mode === "associate"}>
            <input
              type="radio"
              name="app-mode"
              checked={mode === "associate"}
              onChange={() => {
                onModeChange("associate")
                if (!pickedRef.current) {
                  pickedRef.current = true
                  onLoadApps("")
                }
              }}
            />
            <span className={css.radioLabel}>
              <strong>{t("associateExisting")}</strong>
              <small>{t("associateHint")}</small>
            </span>
          </label>
          {mode === "associate" && (
            <>
              <form className={css.appSearch} onSubmit={onSearchApps}>
                <input
                  className={css.input}
                  value={appsSearch}
                  placeholder={t("searchApps")}
                  onChange={(event) => { onAppsSearchChange(event.target.value) }}
                />
                <button type="submit" className={css.secondaryButton}>{t("searchApps")}</button>
              </form>
              <div className={css.appList} aria-busy={appsLoading}>
                {appsLoading && <div className={css.appEmpty}>{t("appSearching")}</div>}
                {!appsLoading && apps.length === 0 && <div className={css.appEmpty}>{t("noApps")}</div>}
                {!appsLoading && apps.map((app) => (
                  <button
                    key={app.id}
                    type="button"
                    className={css.appRow}
                    data-selected={associateId === app.id}
                    onClick={() => { onAssociateChange(app.id) }}
                  >
                    <span className={css.appRowMeta}>
                      <strong>{app.name}</strong>
                      <small>{app.appKind}{app.slug ? ` · ${app.slug}` : ""}</small>
                    </span>
                  </button>
                ))}
              </div>
            </>
          )}
        </div>
      </div>
    </>
  )
}

function errorText(cause: unknown, t: PublishingTranslator): string {
  const message = cause instanceof Error && cause.message ? cause.message : String(cause)
  return t("publishFailed", { message })
}

export type { ReactNode };
