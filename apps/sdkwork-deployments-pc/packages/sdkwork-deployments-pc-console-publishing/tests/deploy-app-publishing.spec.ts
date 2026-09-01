/**
 * Unit tests for the create-deploy-app publishing service: slug derivation,
 * semver validation, metadata assembly (JSONB shape), and Drive upload
 * result mapping. Pure functions only — no clients are exercised here.
 */
import { describe, expect, it } from "vitest";
import {
  createDeployAppPublishingService,
  detectFrameworkId,
  DEPLOY_APP_TYPE_OPTIONS,
  deriveAppSlug,
  frameworksOfCard,
  isValidSemver,
  resolveDeployAppType,
  toDeployAppMediaRef,
  type CreateDeployAppInput,
  type DeployAppTypeOption,
} from "../src/service/deploy-app-publishing.ts";

const flutterIos = DEPLOY_APP_TYPE_OPTIONS.find((option) => option.id === "flutter-ios");
const staticWeb = DEPLOY_APP_TYPE_OPTIONS.find((option) => option.id === "static-web");

describe("deriveAppSlug", () => {
  it("derives a lowercase dashed ascii slug from a display name", () => {
    expect(deriveAppSlug("My Store App")).toBe("my-store-app");
  });

  it("drops non-ascii characters and collapses separators", () => {
    expect(deriveAppSlug("商城 App__v1")).toBe("app-v1");
  });

  it("trims leading/trailing separators", () => {
    expect(deriveAppSlug("- App -")).toBe("app");
  });
});

describe("isValidSemver", () => {
  it("accepts plain and prerelease semvers", () => {
    expect(isValidSemver("1.0.0")).toBe(true);
    expect(isValidSemver("1.2.3-beta.1")).toBe(true);
    expect(isValidSemver("0.0.1+build.7")).toBe(true);
  });

  it("rejects malformed versions", () => {
    expect(isValidSemver("1.0")).toBe(false);
    expect(isValidSemver("v1.0.0")).toBe(false);
    expect(isValidSemver("1.0.0.0")).toBe(false);
    expect(isValidSemver("")).toBe(false);
  });
});

describe("createDeployAppPublishingService metadata assembly", () => {
  it("assembles deploy_app.metadata JSONB with the dialog fields", () => {
    const input: CreateDeployAppInput = {
      sourceDirectory: "/workspace/my-app",
      type: flutterIos as DeployAppTypeOption,
      version: "1.0.0",
      description: "A test app",
      releaseNotes: "First release",
      category: {
        id: "dev-tools",
        path: [{ id: "developer", label: "开发者" }, { id: "dev-tools", label: "开发工具" }],
      },
      media: {
        icon: { driveNodeId: "n1", driveSpaceId: "s1", uploadItemId: "i1", uploadSessionId: "u1", fileName: "icon.png", contentType: "image/png" },
        cover: undefined,
        screenshots: {},
      },
    };
    // The service builds metadata through the same helper used by createApp;
    // instantiate it with a minimal client seam that never runs.
    const service = createDeployAppPublishingService({
      deployClient: undefined as never,
      driveClient: undefined as never,
    });
    const metadata = service.buildMetadata(input);
    expect(metadata.sourceDirectory).toBe("/workspace/my-app");
    expect(metadata.version).toBe("1.0.0");
    expect(metadata.releaseNotes).toBe("First release");
    expect(metadata.category).toEqual({
      id: "dev-tools",
      path: [{ id: "developer", label: "开发者" }, { id: "dev-tools", label: "开发工具" }],
    });
    expect(metadata.media).toEqual(input.media);
  });

  it("drops undefined metadata keys so JSONB stays tidy", () => {
    const service = createDeployAppPublishingService({
      deployClient: undefined as never,
      driveClient: undefined as never,
    });
    const metadata = service.buildMetadata({
      sourceDirectory: "/d",
      type: staticWeb as DeployAppTypeOption,
      version: "0.1.0",
    });
    expect(metadata).not.toHaveProperty("releaseNotes");
    expect(metadata).not.toHaveProperty("category");
    expect(metadata).not.toHaveProperty("media");
    expect(metadata).not.toHaveProperty("framework");
    expect(metadata).not.toHaveProperty("buildOutputPath");
    expect(Object.keys(metadata).sort()).toEqual(["sourceDirectory", "version"]);
  });

  it("writes the v3 framework and build-output path into metadata", () => {
    const service = createDeployAppPublishingService({
      deployClient: undefined as never,
      driveClient: undefined as never,
    });
    const metadata = service.buildMetadata({
      sourceDirectory: "/workspace/apps/sdkwork-shop-h5",
      type: flutterIos as DeployAppTypeOption,
      version: "1.0.0",
      framework: "flutter",
      buildOutputPath: "build/ios/iphoneos/",
    });
    expect(metadata.framework).toBe("flutter");
    // Trailing separators are trimmed so the stored path stays canonical.
    expect(metadata.buildOutputPath).toBe("build/ios/iphoneos");
  });
});

describe("toDeployAppMediaRef", () => {
  it("maps the Drive upload result onto the persisted media reference", () => {
    const ref = toDeployAppMediaRef(
      {
        uploadSession: { id: "session-1" },
        uploadItem: { id: "item-1", spaceId: "space-1", nodeId: "node-1" },
      } as never,
      { fileName: "cover.png", contentType: "image/png", width: 1200, height: 400 },
    );
    expect(ref).toEqual({
      driveNodeId: "node-1",
      driveSpaceId: "space-1",
      uploadItemId: "item-1",
      uploadSessionId: "session-1",
      fileName: "cover.png",
      contentType: "image/png",
      width: 1200,
      height: 400,
    });
  });
});

describe("DEPLOY_APP_TYPE_OPTIONS", () => {
  it("covers the requested publish targets with platform and tech stack", () => {
    const byId = new Map(DEPLOY_APP_TYPE_OPTIONS.map((option) => [option.id, option]));
    expect(byId.get("static-web")).toMatchObject({ appKind: "STATIC_WEB", platform: "WEB" });
    expect(byId.get("wechat-mini-program")).toMatchObject({ appKind: "WECHAT_MINIPROGRAM", platform: "WECHAT" });
    expect(byId.get("flutter-ios")).toMatchObject({ appKind: "IOS_APP", platform: "IOS", techStack: "FLUTTER" });
    expect(byId.get("flutter-android")).toMatchObject({ appKind: "ANDROID_APP", platform: "ANDROID", techStack: "FLUTTER" });
    expect(byId.get("native-ios")).toMatchObject({ appKind: "IOS_APP", platform: "IOS", techStack: "NATIVE" });
    expect(byId.get("native-android")).toMatchObject({ appKind: "ANDROID_APP", platform: "ANDROID", techStack: "NATIVE" });
    expect(byId.get("harmonyos")).toMatchObject({ appKind: "HARMONYOS_APP", platform: "HARMONYOS" });
    expect(byId.get("api-service")).toMatchObject({ appKind: "API_SERVICE", platform: "API" });
  });

  it("adds the v2 h5 / pc-web / desktop targets with sdkwork surfaces", () => {
    const byId = new Map(DEPLOY_APP_TYPE_OPTIONS.map((option) => [option.id, option]));
    expect(byId.get("h5")).toMatchObject({ appKind: "SPA_WEB", platform: "WEB", surface: "h5" });
    expect(byId.get("pc-web")).toMatchObject({ appKind: "SPA_WEB", platform: "WEB", surface: "pc" });
    // Rust 契约已定义 DESKTOP_APP；生成 TS SDK 滞后，由 DeployAppKind 本地扩宽。
    expect(byId.get("desktop")).toMatchObject({ appKind: "DESKTOP_APP", surface: "desktop" });
  });

  it("adds the v3 framework-resolution rows over the full TechStack union", () => {
    const byId = new Map(DEPLOY_APP_TYPE_OPTIONS.map((option) => [option.id, option]));
    expect(byId.get("react-native-android")).toMatchObject({ appKind: "ANDROID_APP", platform: "ANDROID", techStack: "OTHER" });
    expect(byId.get("uniapp-android")).toMatchObject({ appKind: "ANDROID_APP", platform: "ANDROID", techStack: "UNI_APP" });
    expect(byId.get("uniapp-h5")).toMatchObject({ appKind: "SPA_WEB", platform: "WEB", techStack: "UNI_APP" });
    expect(byId.get("api-service-rust")).toMatchObject({ appKind: "API_SERVICE", platform: "API", techStack: "RUST" });
    expect(byId.get("api-service-node")).toMatchObject({ techStack: "NODE" });
    expect(byId.get("api-service-go")).toMatchObject({ techStack: "GO" });
    expect(byId.get("api-service-java")).toMatchObject({ techStack: "JAVA" });
    expect(byId.get("desktop-tauri")).toMatchObject({ appKind: "DESKTOP_APP", surface: "desktop" });
    expect(byId.get("uniapp-wechat-mini-program")).toMatchObject({ appKind: "WECHAT_MINIPROGRAM", techStack: "UNI_APP" });
  });
});

describe("resolveDeployAppType", () => {
  it("resolves framework-selected cards onto the concrete option rows", () => {
    expect(resolveDeployAppType("mini-program", "wechat-native")?.id).toBe("wechat-mini-program");
    expect(resolveDeployAppType("mini-program", "douyin-native")?.id).toBe("douyin-mini-program");
    expect(resolveDeployAppType("mini-program", "uniapp")?.id).toBe("uniapp-wechat-mini-program");
    expect(resolveDeployAppType("android", "flutter")?.id).toBe("flutter-android");
    expect(resolveDeployAppType("android", "react-native")?.id).toBe("react-native-android");
    expect(resolveDeployAppType("ios", "swift")?.id).toBe("native-ios");
    expect(resolveDeployAppType("ios", "uniapp")?.id).toBe("uniapp-ios");
  });

  it("falls back to the default framework and maps unsurfaced cards directly", () => {
    expect(resolveDeployAppType("android")?.id).toBe("native-android");
    expect(resolveDeployAppType("ios")?.id).toBe("native-ios");
    expect(resolveDeployAppType("h5")?.surface).toBe("h5");
    expect(resolveDeployAppType("desktop")?.id).toBe("desktop-electron");
    expect(resolveDeployAppType("desktop")?.appKind).toBe("DESKTOP_APP");
    expect(resolveDeployAppType("static-web")).toBeDefined();
    expect(resolveDeployAppType(undefined)).toBeUndefined();
    expect(resolveDeployAppType("unknown-card")).toBeUndefined();
    expect(resolveDeployAppType("android", "unknown-framework")).toBeUndefined();
  });
});

describe("frameworksOfCard (v3 framework registry)", () => {
  it("exposes industry-standard frameworks per card with build-output defaults", () => {
    expect(frameworksOfCard("h5").map((framework) => framework.id)).toEqual([
      "react", "vue", "next", "nuxt", "uniapp", "capacitor",
    ]);
    expect(frameworksOfCard("android").map((framework) => framework.id)).toEqual([
      "kotlin", "java", "flutter", "react-native", "uniapp",
    ]);
    expect(frameworksOfCard("ios").map((framework) => framework.id)).toEqual([
      "swift", "objc", "flutter", "react-native", "uniapp",
    ]);
    expect(frameworksOfCard("desktop").map((framework) => framework.id)).toEqual([
      "electron", "tauri", "qt", "flutter",
    ]);
    const nuxtH5 = frameworksOfCard("h5").find((framework) => framework.id === "nuxt");
    expect(nuxtH5?.buildOutputPath).toBe(".output/public");
  });

  it("returns an empty list for unknown cards", () => {
    expect(frameworksOfCard(undefined)).toEqual([]);
    expect(frameworksOfCard("unknown-card")).toEqual([]);
  });
});

describe("detectFrameworkId (v3.2 directory-signal detection)", () => {
  it("detects frameworks from marker directories in the surface listing", () => {
    // Flutter Android 工程：.dart_tool 标记命中 flutter。
    expect(detectFrameworkId(frameworksOfCard("android"), [".dart_tool", "android", "ios", "lib"])).toBe("flutter");
    // uni-app 产物目录：unpackage 标记命中 uniapp。
    expect(detectFrameworkId(frameworksOfCard("h5"), ["src", "unpackage"])).toBe("uniapp");
    // Nuxt：.nuxt 标记命中 nuxt（而非默认 react）。
    expect(detectFrameworkId(frameworksOfCard("h5"), [".nuxt", "app", "public"])).toBe("nuxt");
    // React Native：android + ios 双标记命中 react-native（无 .dart_tool 时）。
    expect(detectFrameworkId(frameworksOfCard("android"), ["android", "ios", "src"])).toBe("react-native");
    // Tauri 桌面：src-tauri 标记命中 tauri。
    expect(detectFrameworkId(frameworksOfCard("desktop"), ["src", "src-tauri"])).toBe("tauri");
  });

  it("prefers the earlier registry entry when multiple frameworks match", () => {
    // 注册表顺序 kotlin → java → flutter → react-native → uniapp：
    // .dart_tool 与 unpackage 同时存在时按优先级取 flutter。
    expect(detectFrameworkId(frameworksOfCard("android"), [".dart_tool", "unpackage"])).toBe("flutter");
  });

  it("returns undefined when no marker set is fully present", () => {
    // 无任何标记目录。
    expect(detectFrameworkId(frameworksOfCard("android"), ["src", "lib"])).toBeUndefined();
    // 空列举与缺失输入。
    expect(detectFrameworkId(frameworksOfCard("android"), [])).toBeUndefined();
    expect(detectFrameworkId(frameworksOfCard("android"), undefined)).toBeUndefined();
    // 未知卡片无注册表。
    expect(detectFrameworkId(frameworksOfCard("unknown-card"), ["unpackage"])).toBeUndefined();
  });
});
