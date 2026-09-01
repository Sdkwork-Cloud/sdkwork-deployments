/**
 * Unit tests for the App Store preview (screenshot) guidelines: aspect
 * tolerance validation, per-kind target selection, and content-type list.
 */
import { describe, expect, it } from "vitest";
import {
  APP_STORE_PREVIEW_TARGETS,
  MAX_SCREENSHOTS_TOTAL,
  MEDIA_ACCEPTED_TYPES,
  PREVIEW_ASPECT_TOLERANCE,
  previewTargetsForAppKind,
  validatePreviewSize,
} from "../src/service/app-store-preview-spec.ts";

describe("APP_STORE_PREVIEW_TARGETS", () => {
  it("caps every target at 10 screenshots and the total at 30", () => {
    for (const target of APP_STORE_PREVIEW_TARGETS) {
      expect(target.max).toBeLessThanOrEqual(10);
    }
    expect(MAX_SCREENSHOTS_TOTAL).toBe(30);
  });
});

describe("validatePreviewSize", () => {
  const iphone67 = APP_STORE_PREVIEW_TARGETS.find((target) => target.key === "iphone-67")!;

  it("accepts the exact nominal size", () => {
    expect(validatePreviewSize(1290, 2796, iphone67)).toEqual({ ok: true });
  });

  it("accepts sizes within the aspect tolerance", () => {
    const shifted = Math.round(1290 * (1 - PREVIEW_ASPECT_TOLERANCE / 2));
    expect(validatePreviewSize(shifted, 2796, iphone67)).toEqual({ ok: true });
  });

  it("rejects clearly wrong aspect ratios", () => {
    expect(validatePreviewSize(2796, 1290, iphone67)).toEqual({ ok: false, reason: "size" });
  });
});

describe("previewTargetsForAppKind", () => {
  it("offers phone and tablet targets for iOS apps", () => {
    const targets = previewTargetsForAppKind("IOS_APP");
    expect(targets.length).toBeGreaterThan(0);
    expect(targets.every((target) => target.device === "iphone" || target.device === "ipad")).toBe(true);
  });

  it("falls back to phone targets for web kinds", () => {
    const targets = previewTargetsForAppKind("STATIC_WEB");
    expect(targets.length).toBeGreaterThan(0);
    expect(targets.every((target) => target.device === "iphone")).toBe(true);
  });
});

describe("MEDIA_ACCEPTED_TYPES", () => {
  it("accepts png, jpeg and webp", () => {
    expect(MEDIA_ACCEPTED_TYPES).toEqual(["image/png", "image/jpeg", "image/webp"]);
  });
});
