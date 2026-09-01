/**
 * App Store preview (screenshot) guidelines, as documented by App Store Connect.
 *
 * Screenshots are uploaded per device size. Every device size allows at most
 * `MAX_SCREENSHOTS_PER_TARGET` images; dimensions are validated against the
 * nominal pixel size with a small aspect tolerance (Apple recommends exact
 * pixels, but JPG re-encoding routinely shifts a row or two, so we accept
 * 0.5% deviation and normalize by cropping the overflow at upload time).
 */
import type { AppKind } from "@sdkwork/deployments-app-sdk";

/** One supported preview size target. */
export interface PreviewSizeTarget {
  /** Stable key persisted in deploy_app.metadata.media.screenshots. */
  readonly key: string
  /** Label key in the publishing locale catalog. */
  readonly labelKey: string
  /** Nominal portrait pixel size. */
  readonly width: number
  readonly height: number
  /** Which device family this target documents. */
  readonly device: "iphone" | "ipad" | "android" | "mac"
  /** Per-target screenshot count cap (App Store limit). */
  readonly max: number
}

/** Apple documentation reference list of portrait preview sizes. */
export const APP_STORE_PREVIEW_TARGETS: readonly PreviewSizeTarget[] = [
  { key: "iphone-67", labelKey: "screenshotTarget", width: 1290, height: 2796, device: "iphone", max: 10 },
  { key: "iphone-65", labelKey: "screenshotTarget", width: 1242, height: 2688, device: "iphone", max: 10 },
  { key: "iphone-61", labelKey: "screenshotTarget", width: 1179, height: 2556, device: "iphone", max: 10 },
  { key: "iphone-55", labelKey: "screenshotTarget", width: 1242, height: 2208, device: "iphone", max: 10 },
  { key: "ipad-pro-129", labelKey: "screenshotTarget", width: 2048, height: 2732, device: "ipad", max: 10 },
  { key: "ipad-11", labelKey: "screenshotTarget", width: 1668, height: 2388, device: "ipad", max: 10 },
  { key: "ipad-105", labelKey: "screenshotTarget", width: 1668, height: 2224, device: "ipad", max: 10 },
  { key: "ipad-102", labelKey: "screenshotTarget", width: 1620, height: 2160, device: "ipad", max: 10 },
  { key: "android-portrait", labelKey: "screenshotTarget", width: 1080, height: 2340, device: "android", max: 10 },
  { key: "mac", labelKey: "screenshotTarget", width: 2880, height: 1800, device: "mac", max: 10 },
] as const;

/** Total screenshot cap across all targets (guards runaway uploads). */
export const MAX_SCREENSHOTS_TOTAL = 30;

/** Aspect deviation accepted before the size check fails. */
export const PREVIEW_ASPECT_TOLERANCE = 0.005;

/** Default targets offered per app kind. */
export function previewTargetsForAppKind(appKind: AppKind): readonly PreviewSizeTarget[] {
  switch (appKind) {
    case "IOS_APP":
      return APP_STORE_PREVIEW_TARGETS.filter((target) => target.device === "iphone" || target.device === "ipad");
    case "ANDROID_APP":
      return APP_STORE_PREVIEW_TARGETS.filter((target) => target.device === "android");
    case "HARMONYOS_APP":
      return APP_STORE_PREVIEW_TARGETS.filter((target) => target.device === "iphone");
    default:
      return APP_STORE_PREVIEW_TARGETS.filter((target) => target.device === "iphone");
  }
}

/** Aspect-ratio-tolerant size validation result. */
export type PreviewValidationResult =
  | { readonly ok: true }
  | { readonly ok: false; readonly reason: "size" | "count" | "type" };

/** Validate a screenshot against one target; count is checked by the caller. */
export function validatePreviewSize(
  width: number,
  height: number,
  target: PreviewSizeTarget,
): PreviewValidationResult {
  const nominalAspect = target.width / target.height
  const actualAspect = width / height
  if (Math.abs(actualAspect - nominalAspect) / nominalAspect > PREVIEW_ASPECT_TOLERANCE) {
    return { ok: false, reason: "size" }
  }
  return { ok: true }
}

/** Accepted image content types for icon / cover / screenshots. */
export const MEDIA_ACCEPTED_TYPES: readonly string[] = ["image/png", "image/jpeg", "image/webp"];

/** Icon requirements: square bitmap, recommended 1024x1024. */
export const APP_ICON_SPEC = {
  minEdge: 512,
  recommendedEdge: 1024,
  mustBeSquare: true,
  maxBytes: 8 * 1024 * 1024,
} as const;

/** Cover requirements: landscape banner. */
export const COVER_SPEC = {
  minWidth: 1200,
  minHeight: 400,
  maxBytes: 12 * 1024 * 1024,
} as const;
