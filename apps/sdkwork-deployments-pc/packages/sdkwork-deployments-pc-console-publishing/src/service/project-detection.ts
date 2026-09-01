/**
 * sdkwork-specs project auto-detection (发布对话框 v2: 应用自动检测能力).
 *
 * Pure, UI-free detection over a host-provided directory inspection. The
 * dialog stays decoupled from the filesystem: a host port (BirdCoder desktop
 * via uiWorkspace.listDirectory, or any future bridge) hands over the child
 * directory names, and this module maps them onto the sdkwork-specs layout.
 *
 * Authorities (sdkwork-specs):
 * - `APPLICATION_SPEC.md` — `apps/` surface roots:
 *   `apps/sdkwork-<application-code>-{pc,h5,mini-program,android-mobile,ios-mobile,harmony-mobile}`.
 * - `APPLICATION_DEPLOY_LAYOUT_SPEC.md` §2 — deployable root markers:
 *   `specs/`, `etc/`, `deployments/` (+ `apps/` for client surfaces).
 * - `APP_MANIFEST_SPEC.md` — `sdkwork.app.config.json` + source-controlled
 *   `.sdkwork/` workspace on the same root.
 * - `ENVIRONMENT_SPEC.md` §2/§5.1 — canonical environments
 *   (`development|test|staging|demo|production`, aliases `dev|prod`) and the
 *   `<standalone|cloud>.<environment>` profile id grammar.
 */

/** Canonical publish environment (ENVIRONMENT_SPEC.md §2; no aliases). */
export type DeployEnvironmentId = "development" | "test" | "staging" | "demo" | "production";

/** Deployment mode half of the canonical profile id (ENVIRONMENT_SPEC.md §5.1). */
export type DeployDeploymentMode = "standalone" | "cloud";

/** Command/operator aliases normalized before canonical profile selection. */
export const DEPLOY_ENVIRONMENT_ALIASES: Readonly<Record<string, DeployEnvironmentId>> = {
  dev: "development",
  prod: "production",
};

/** Canonical environments in publish-target order. */
export const DEPLOY_ENVIRONMENT_IDS: readonly DeployEnvironmentId[] = [
  "development",
  "test",
  "staging",
  "demo",
  "production",
];

/** Deployment modes in dialog order. */
export const DEPLOY_DEPLOYMENT_MODES: readonly DeployDeploymentMode[] = ["standalone", "cloud"];

/** Canonical profile id `<mode>.<environment>` (ENVIRONMENT_SPEC.md §5.1). */
export function deployProfileId(mode: DeployDeploymentMode, environment: DeployEnvironmentId): string {
  return `${mode}.${environment}`;
}

/** Normalize a legacy alias; unknown values pass through unchanged. */
export function canonicalEnvironment(value: string): string {
  return DEPLOY_ENVIRONMENT_ALIASES[value.trim().toLowerCase()] ?? value.trim();
}

/**
 * App surface ids used by the dialog's type cards. `android`/`ios` are the
 * card-level ids; their `apps/` directory suffixes differ (see
 * {@link APP_SURFACE_DIRECTORY_SUFFIX}).
 */
export type AppSurfaceId =
  | "pc"
  | "h5"
  | "desktop"
  | "mini-program"
  | "android"
  | "ios"
  | "harmony"
  | "api"
  | "static";

/** `apps/` child directory suffix per surface (APPLICATION_SPEC.md §2). */
export const APP_SURFACE_DIRECTORY_SUFFIX: Readonly<Record<AppSurfaceId, string>> = {
  pc: "pc",
  h5: "h5",
  desktop: "desktop",
  "mini-program": "mini-program",
  android: "android-mobile",
  ios: "ios-mobile",
  harmony: "harmony-mobile",
  // Non-client surfaces have no dedicated apps/ root: they publish the repo root.
  api: "",
  static: "",
};

/** Surfaces that own an `apps/sdkwork-<code>-<suffix>/` root. */
const SURFACED: readonly AppSurfaceId[] = ["pc", "h5", "desktop", "mini-program", "android", "ios", "harmony"];

/** Root child directory markers checked for layout conformance. */
const ROOT_MARKERS: readonly string[] = ["apps", "deployments", "etc", "specs", ".sdkwork"];

/** Match `sdkwork-<code>-<suffix>` and capture the kebab application code. */
const SURFACE_DIRECTORY_PATTERN =
  /^sdkwork-(?<code>[a-z0-9][a-z0-9-]*?)-(?<suffix>pc|h5|desktop|mini-program|android-mobile|ios-mobile|harmony-mobile)$/;

/** Host inspection payload: names only, no file contents required. */
export interface DeployProjectInspection {
  /** Absolute inspected directory path. */
  readonly rootPath: string
  /** Child directory names of the inspected root. */
  readonly childDirectories: readonly string[]
  /** Child directory names of `<root>/apps`, when that directory exists. */
  readonly appsChildDirectories?: readonly string[] | undefined
  /**
   * v3: child directory names of each `apps/` surface root, keyed by the
   * surface directory name. Optional: hosts without a deeper listing simply
   * omit it and the build-output detection falls back to framework defaults.
   */
  readonly surfaceChildDirectories?: Readonly<Record<string, readonly string[]>> | undefined
}

/** One detected `apps/sdkwork-<code>-<suffix>/` surface root. */
export interface DeployDetectedSurface {
  /** Canonical dialog surface id. */
  readonly surface: AppSurfaceId
  /** Matched directory name under `apps/`. */
  readonly directory: string
  /** Absolute surface root (`<rootPath>/apps/<directory>`). */
  readonly path: string
  /** v3: child directory names of this surface root, when the host listed them. */
  readonly childDirectories?: readonly string[] | undefined
}

/** Layout conformance level reported to the dialog. */
export type DeployProjectConformance = "conformant" | "partial" | "unknown";

/** Detection result consumed by the directory step. */
export interface DeployProjectDetection {
  /** `sdkwork-<code>` application code derived from the matched surfaces. */
  readonly applicationCode?: string | undefined
  /** Detected surface roots, ordered by {@link SURFACED}. */
  readonly surfaces: readonly DeployDetectedSurface[]
  readonly conformance: DeployProjectConformance
  /** Spec markers present at the root. */
  readonly presentMarkers: readonly string[]
  /** Spec markers missing at the root. */
  readonly missingMarkers: readonly string[]
}

/** @returns the surface id for an `apps/` child name, or undefined. */
export function surfaceOfDirectoryName(name: string): { surface: AppSurfaceId; applicationCode: string } | undefined {
  const match = SURFACE_DIRECTORY_PATTERN.exec(name);
  if (!match?.groups) return undefined;
  const suffix = match.groups.suffix as string;
  const surface = (Object.entries(APP_SURFACE_DIRECTORY_SUFFIX) as readonly [AppSurfaceId, string][])
    .find(([, dirSuffix]) => dirSuffix === suffix)?.[0];
  if (surface === undefined) return undefined;
  return { surface, applicationCode: match.groups.code as string };
}

/**
 * Detect the sdkwork project shape behind an inspection.
 *
 * Conformance follows `APPLICATION_DEPLOY_LAYOUT_SPEC.md` §2: every marker
 * present is `conformant`, at least two is `partial`, otherwise the directory
 * is not recognized as a sdkwork deployable root. `sdkwork.app.config.json`
 * is a file and intentionally not required here — directory-only listings
 * (the current host bridge) cannot observe it, and the manifest check stays a
 * backend concern at publish time.
 */
export function detectSdkworkProject(inspection: DeployProjectInspection): DeployProjectDetection {
  const children = new Set(inspection.childDirectories);
  const presentMarkers = ROOT_MARKERS.filter((marker) => children.has(marker));
  const missingMarkers = ROOT_MARKERS.filter((marker) => !children.has(marker));

  const surfaces: DeployDetectedSurface[] = [];
  const applicationCodes = new Set<string>();
  for (const name of inspection.appsChildDirectories ?? []) {
    const detected = surfaceOfDirectoryName(name);
    if (detected === undefined) continue;
    applicationCodes.add(detected.applicationCode);
    surfaces.push({
      surface: detected.surface,
      directory: name,
      path: joinPath(inspection.rootPath, "apps", name),
      childDirectories: inspection.surfaceChildDirectories?.[name],
    });
  }
  surfaces.sort(
    (left, right) => SURFACED.indexOf(left.surface) - SURFACED.indexOf(right.surface),
  );

  const conformance: DeployProjectConformance = presentMarkers.length === ROOT_MARKERS.length
    ? "conformant"
    : presentMarkers.length >= 2
      ? "partial"
      : "unknown";

  return {
    applicationCode: applicationCodes.size === 1 ? applicationCodes.values().next().value : undefined,
    surfaces,
    conformance,
    presentMarkers,
    missingMarkers,
  };
}

/**
 * Resolve the publish source directory for a selected surface: the detected
 * `apps/` surface root when present, otherwise the inspected root.
 */
export function resolveSourceDirectory(
  detection: DeployProjectDetection,
  surface: AppSurfaceId | undefined,
  rootPath: string,
): string {
  if (surface === undefined) return rootPath;
  return detection.surfaces.find((candidate) => candidate.surface === surface)?.path ?? rootPath;
}

/**
 * Sdkwork repository root directory name: `sdkwork-<code>` without a surface
 * suffix (APPLICATION_SPEC.md §2 — surface directories always end in one of
 * the known suffixes, which the surface pattern above already strips first).
 */
const REPO_ROOT_DIRECTORY_PATTERN = /^sdkwork-[a-z0-9][a-z0-9-]*$/;

/**
 * v3.3: derive the spec-compliant surface root from the directory path alone
 * (no host listing required) — `E:\...\sdkwork-<code>` →
 * `E:\...\sdkwork-<code>\apps\sdkwork-<code>-<suffix>` per APPLICATION_SPEC.
 *
 * Rules:
 * - `api`/`static` surfaces publish the repo root itself → undefined.
 * - basename is already the requested surface root → undefined (nothing to do).
 * - basename is a *sibling* surface root under `apps/` (user switched app
 *   type while sitting in another surface root) → derive the sibling surface
 *   directory next to it.
 * - basename is a repo root `sdkwork-<code>` → `<dir>/apps/sdkwork-<code>-<suffix>`.
 * - anything else (not a sdkwork name) → undefined; never invents paths.
 *
 * The original path separator (Windows `\` vs POSIX `/`) is preserved.
 */
export function deriveSurfaceDirectory(
  directory: string,
  surface: AppSurfaceId,
): string | undefined {
  const suffix = APP_SURFACE_DIRECTORY_SUFFIX[surface];
  if (suffix === "") return undefined;
  const segments = directory.split(/[\\/]/).filter((segment) => segment !== "");
  const basename = segments[segments.length - 1];
  if (basename === undefined) return undefined;
  const separator = directory.includes("\\") ? "\\" : "/";

  const surfaceMatch = SURFACE_DIRECTORY_PATTERN.exec(basename);
  if (surfaceMatch?.groups !== undefined) {
    if (surfaceMatch.groups.suffix === suffix) return undefined;
    // 同级表面目录切换：仅当父目录是 apps/（规范布局）时推导兄弟表面根。
    if (segments[segments.length - 2] !== "apps") return undefined;
    return [...segments.slice(0, -1), `sdkwork-${surfaceMatch.groups.code}-${suffix}`].join(separator);
  }

  if (REPO_ROOT_DIRECTORY_PATTERN.test(basename) === false) return undefined;
  const applicationCode = basename.slice("sdkwork-".length);
  // POSIX/UNC 绝对路径保留前导分隔符（split+filter 会吃掉空首段）。
  const prefix = /^[\\/]/.test(directory) ? separator : "";
  return prefix + [...segments, "apps", `sdkwork-${applicationCode}-${suffix}`].join(separator);
}

/**
 * v3: directory names recognized as build-output roots when they appear as
 * direct children of the application root. Intentionally generic — the
 * framework-aware defaults live with the framework registry
 * (`deploy-app-publishing.ts`); this list only backs the "generic candidates"
 * chips shown next to the build-output field.
 */
export const KNOWN_BUILD_OUTPUT_DIRECTORY_NAMES: readonly string[] = [
  "dist", "build", "out", ".output", ".next", ".nuxt", "public", "unpackage",
  "target", "bin", "release",
];

/**
 * v3: does a relative build-output path exist under the surface root, judged
 * from the host's directory listing? `"."` (publish the root itself) always
 * exists; nested paths are judged by their first segment.
 */
export function buildOutputExists(
  buildOutputPath: string,
  childDirectories: readonly string[] | undefined,
): boolean | undefined {
  const trimmed = buildOutputPath.trim().replace(/^\.\//, "");
  if (trimmed === "" || trimmed === ".") return true;
  if (trimmed.startsWith("/") || trimmed.startsWith("../") || /^[a-zA-Z]:/.test(trimmed)) return false;
  if (childDirectories === undefined) return undefined;
  const firstSegment = trimmed.split(/[\\/]/)[0] ?? "";
  return childDirectories.includes(firstSegment);
}

/**
 * v3: generic build-output candidates observed in the surface root's child
 * directories (intersected with {@link KNOWN_BUILD_OUTPUT_DIRECTORY_NAMES}).
 */
export function detectBuildOutputCandidates(
  childDirectories: readonly string[] | undefined,
): readonly string[] {
  if (childDirectories === undefined) return [];
  return KNOWN_BUILD_OUTPUT_DIRECTORY_NAMES.filter((name) => childDirectories.includes(name));
}

/** POSIX/Windows-agnostic join for display paths (no filesystem access). */
export function joinPath(...segments: readonly string[]): string {
  return segments
    .filter((segment) => segment !== "")
    .map((segment, index) => (index === 0 ? segment.replace(/[\\/]+$/, "") : segment.replace(/^[\\/]+|[\\/]+$/g, "")))
    .join("/");
}
