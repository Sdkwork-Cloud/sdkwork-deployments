/**
 * Media asset fields (需求 4/5/6): app icon, cover image, and screenshots.
 *
 * Screenshots follow App Store preview guidelines: each device-size target
 * accepts up to `max` images and pixel dimensions are validated (aspect
 * tolerance). The component collects and validates `File`s only — uploading
 * happens once the app exists, from the parent dialog, so this stays a dumb
 * presentational unit.
 */
import { useEffect, useRef, useState, type ChangeEvent } from "react";
import type { PublishingTranslator } from "../i18n.ts";
import {
  APP_ICON_SPEC,
  APP_STORE_PREVIEW_TARGETS,
  COVER_SPEC,
  MAX_SCREENSHOTS_TOTAL,
  MEDIA_ACCEPTED_TYPES,
  validatePreviewSize,
  type PreviewSizeTarget,
} from "../service/app-store-preview-spec.ts";
import css from "./create-deploy-app.module.css";

export interface DeployAppMediaFiles {
  readonly icon?: File
  readonly cover?: File
  readonly screenshots: Record<string, readonly File[]>
}

export interface DeployAppMediaFieldsProps {
  readonly value: DeployAppMediaFiles
  readonly onChange: (next: DeployAppMediaFiles) => void
  readonly t: PublishingTranslator
}

interface ProbingImage {
  width: number
  height: number
}

/** Read intrinsic pixel size from a local file via the browser image decoder. */
function probeImage(file: File): Promise<ProbingImage | undefined> {
  return new Promise((resolve) => {
    const url = URL.createObjectURL(file)
    const image = new Image()
    image.onload = () => {
      URL.revokeObjectURL(url)
      resolve({ width: image.naturalWidth, height: image.naturalHeight })
    }
    image.onerror = () => {
      URL.revokeObjectURL(url)
      resolve(undefined)
    }
    image.src = url
  })
}

/** Screenshot target keys offered per selected app kind (caller narrows). */
export function screenshotTargets(): readonly PreviewSizeTarget[] {
  return APP_STORE_PREVIEW_TARGETS
}

export function DeployAppMediaFields({ value, onChange, t }: DeployAppMediaFieldsProps) {
  const [iconUrl, setIconUrl] = useState<string>()
  const [coverUrl, setCoverUrl] = useState<string>()
  const [error, setError] = useState<string>()
  const [screenshotUrls, setScreenshotUrls] = useState<Record<string, string[]>>({})
  const [selectedTarget, setSelectedTarget] = useState<string>(APP_STORE_PREVIEW_TARGETS[0].key)
  const iconInputRef = useRef<HTMLInputElement>(null)
  const coverInputRef = useRef<HTMLInputElement>(null)
  const screenshotInputRef = useRef<HTMLInputElement>(null)

  // Revoke object URLs on unmount.
  useEffect(() => {
    const urls = [iconUrl, coverUrl, ...Object.values(screenshotUrls).flat()].filter(Boolean) as string[]
    return () => { urls.forEach((url) => URL.revokeObjectURL(url)) }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  const accept = MEDIA_ACCEPTED_TYPES.join(",")
  const target = APP_STORE_PREVIEW_TARGETS.find((item) => item.key === selectedTarget) ?? APP_STORE_PREVIEW_TARGETS[0]
  const targetCount = value.screenshots[target.key]?.length ?? 0

  const setIcon = async (file: File) => {
    if (!MEDIA_ACCEPTED_TYPES.includes(file.type)) {
      setError(t("mediaTypeError", { types: "PNG / JPG / WEBP" }))
      return
    }
    const probe = await probeImage(file)
    if (probe && (APP_ICON_SPEC.mustBeSquare ? probe.width !== probe.height : probe.width < APP_ICON_SPEC.minEdge)) {
      setError(t("appIconError"))
      return
    }
    setError(undefined)
    setIconUrl((current) => { if (current) URL.revokeObjectURL(current); return URL.createObjectURL(file) })
    onChange({ ...value, icon: file })
  }

  const setCover = async (file: File) => {
    if (!MEDIA_ACCEPTED_TYPES.includes(file.type)) {
      setError(t("mediaTypeError", { types: "PNG / JPG / WEBP" }))
      return
    }
    const probe = await probeImage(file)
    if (probe && (probe.width < COVER_SPEC.minWidth || probe.height < COVER_SPEC.minHeight)) {
      setError(t("mediaSizeError", { name: file.name, width: String(COVER_SPEC.minWidth), height: String(COVER_SPEC.minHeight), tolerance: "min" }))
      return
    }
    setError(undefined)
    setCoverUrl((current) => { if (current) URL.revokeObjectURL(current); return URL.createObjectURL(file) })
    onChange({ ...value, cover: file })
  }

  const addScreenshots = async (files: FileList | null) => {
    if (!files || files.length === 0) return
    const next = [...files]
    for (const file of next) {
      if (!MEDIA_ACCEPTED_TYPES.includes(file.type)) {
        setError(t("mediaTypeError", { types: "PNG / JPG / WEBP" }))
        return
      }
      const probe = await probeImage(file)
      if (probe && !validatePreviewSize(probe.width, probe.height, target).ok) {
        setError(t("mediaSizeError", { name: file.name, width: String(target.width), height: String(target.height), tolerance: "0.5%" }))
        return
      }
    }
    const total = Object.values(value.screenshots).reduce((sum, list) => sum + list.length, 0)
    if (targetCount + next.length > target.max || total + next.length > MAX_SCREENSHOTS_TOTAL) {
      setError(t("screenshotsHint", { max: String(Math.min(target.max, MAX_SCREENSHOTS_TOTAL)) }))
      return
    }
    setError(undefined)
    const urls = next.map((file) => URL.createObjectURL(file))
    setScreenshotUrls((current) => ({ ...current, [target.key]: [...(current[target.key] ?? []), ...urls] }))
    onChange({
      ...value,
      screenshots: {
        ...value.screenshots,
        [target.key]: [...(value.screenshots[target.key] ?? []), ...next],
      },
    })
  }

  const removeScreenshot = (index: number) => {
    const current = value.screenshots[target.key] ?? []
    onChange({
      ...value,
      screenshots: {
        ...value.screenshots,
        [target.key]: current.filter((_, itemIndex) => itemIndex !== index),
      },
    })
    setScreenshotUrls((urls) => ({
      ...urls,
      [target.key]: (urls[target.key] ?? []).filter((_, itemIndex) => itemIndex !== index),
    }))
  }

  const onIconInput = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) void setIcon(file)
    event.target.value = ""
  }

  const onCoverInput = (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (file) void setCover(file)
    event.target.value = ""
  }

  const onScreenshotInput = (event: ChangeEvent<HTMLInputElement>) => {
    void addScreenshots(event.target.files)
    event.target.value = ""
  }

  return (
    <div className={css.mediaGrid}>
      <div className={css.mediaField}>
        <span className={css.fieldLabel}>{t("appIcon")}</span>
        <div className={css.mediaPreview}>
          {iconUrl
            ? <img src={iconUrl} alt="" />
            : <span className={css.mediaPreviewEmpty}>{t("appIconHint")}</span>}
        </div>
        <div className={css.mediaFileRow}>
          {value.icon && <span className={css.mediaFileName}>{value.icon.name}</span>}
          <button type="button" className={css.secondaryButton} onClick={() => { iconInputRef.current?.click() }}>
            {value.icon ? t("changeImage") : t("chooseImage")}
          </button>
          <input ref={iconInputRef} className={css.fileInputHidden} type="file" accept={accept} onChange={onIconInput} />
        </div>
      </div>

      <div className={css.mediaField}>
        <span className={css.fieldLabel}>{t("coverImage")}</span>
        <div className={css.mediaPreview}>
          {coverUrl
            ? <img src={coverUrl} alt="" />
            : <span className={css.mediaPreviewEmpty}>{t("coverImageHint")}</span>}
        </div>
        <div className={css.mediaFileRow}>
          {value.cover && <span className={css.mediaFileName}>{value.cover.name}</span>}
          <button type="button" className={css.secondaryButton} onClick={() => { coverInputRef.current?.click() }}>
            {value.cover ? t("changeImage") : t("chooseImage")}
          </button>
          <input ref={coverInputRef} className={css.fileInputHidden} type="file" accept={accept} onChange={onCoverInput} />
        </div>
      </div>

      <div className={css.screenshotZone}>
        <span className={css.fieldLabel}>{t("screenshots")}</span>
        <span className={css.fieldHint}>{t("screenshotsHint", { max: String(APP_STORE_PREVIEW_TARGETS[0].max) })}</span>
        <div className={css.screenshotTargets} role="tablist" aria-label={t("screenshotTarget")}>
          {APP_STORE_PREVIEW_TARGETS.map((item) => (
            <button
              key={item.key}
              type="button"
              role="tab"
              aria-selected={item.key === selectedTarget}
              data-selected={item.key === selectedTarget}
              className={css.targetChip}
              title={`${item.width}x${item.height}`}
              onClick={() => { setSelectedTarget(item.key) }}
            >
              {item.device === "iphone" ? "iPhone" : item.device === "ipad" ? "iPad" : item.device === "android" ? "Android" : "Mac"} {item.width}x{item.height}
            </button>
          ))}
        </div>
        <div className={css.mediaMeta}>
          {t("screenshotLimit", { count: String(targetCount), max: String(target.max) })}
        </div>
        <div className={css.screenshotGrid}>
          {(value.screenshots[target.key] ?? []).map((file, index) => (
            <div key={`${file.name}-${index}`} className={css.screenshotCard}>
              <img src={screenshotUrls[target.key]?.[index]} alt={file.name} />
              <button type="button" className={css.screenshotRemove} title={t("remove")} aria-label={`${t("remove")} ${file.name}`} onClick={() => { removeScreenshot(index) }}>
                ×
              </button>
            </div>
          ))}
          {targetCount < target.max && (
            <button type="button" className={css.addScreenshot} onClick={() => { screenshotInputRef.current?.click() }}>
              + {t("addScreenshot")}
            </button>
          )}
          <input ref={screenshotInputRef} className={css.fileInputHidden} type="file" accept={accept} multiple onChange={onScreenshotInput} />
        </div>
      </div>

      {error && <div className={css.errorBanner} role="alert">{error}</div>}
    </div>
  )
}
