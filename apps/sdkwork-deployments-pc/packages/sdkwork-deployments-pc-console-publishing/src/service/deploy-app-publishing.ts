/**
 * Deploy-app publishing service.
 *
 * One high-cohesion facade over the deploy app-api (`/app/v3/api/apps`) and the
 * Drive uploader. It maps the create-deploy-app dialog model onto the existing
 * deploy schema:
 *
 * - `deploy_app`                      <- apps.create (name / slug / app_kind / description / metadata)
 * - `deploy_app_platform_target`      <- apps.platformTargets.create
 * - `deploy_app.metadata` (JSONB)     <- category / media / version / releaseNotes
 * - Drive node refs (icon/cover/screenshots) <- drive uploader, referenced from metadata.media
 *
 * The service is UI-framework-free: callers (deployments console or the
 * birdcoder publish plugin) construct it with the two generated clients and a
 * Drive upload result type, so it stays reusable and decoupled.
 */
import type { SdkworkDeployAppClient } from "@sdkwork/deployments-app-sdk";
import type {
  AppKind,
  AppResponse,
  AppStatus,
  CreatePlatformTargetRequest,
  CreateAppRequest,
  PageInfo,
  Platform,
  TechStack,
} from "@sdkwork/deployments-app-sdk";
import type {
  DriveUploaderBlobLike,
  DriveUploaderUploadResult,
  SdkworkDriveAppClient,
} from "@sdkwork/drive-app-sdk";
import { uuid } from "@sdkwork/utils/id";

/** One selectable application type in the dialog. */
export interface DeployAppTypeOption {
  readonly id: string
  readonly appKind: AppKind
  readonly platform: Platform
  readonly techStack?: TechStack
  /** Locale key for the option label. */
  readonly labelKey: import("../i18n.ts").PublishingMessageKey
  /** Locale key for the helper text. */
  readonly hintKey: import("../i18n.ts").PublishingMessageKey
}

/** Dialog-level application types (需求 2: 静态资源/小程序/Flutter iOS/安卓等). */
export const DEPLOY_APP_TYPE_OPTIONS: readonly DeployAppTypeOption[] = [
  { id: "static-web", appKind: "STATIC_WEB", platform: "WEB", labelKey: "typeStaticWeb", hintKey: "appTypeHint" },
  { id: "spa-web", appKind: "SPA_WEB", platform: "WEB", labelKey: "typeSpaWeb", hintKey: "appTypeHint" },
  { id: "wechat-mini-program", appKind: "WECHAT_MINIPROGRAM", platform: "WECHAT", labelKey: "typeWechatMiniProgram", hintKey: "appTypeHint" },
  { id: "douyin-mini-program", appKind: "DOUYIN_MINIPROGRAM", platform: "DOUYIN", labelKey: "typeDouyinMiniProgram", hintKey: "appTypeHint" },
  { id: "flutter-ios", appKind: "IOS_APP", platform: "IOS", techStack: "FLUTTER", labelKey: "typeFlutterIos", hintKey: "appTypeHint" },
  { id: "flutter-android", appKind: "ANDROID_APP", platform: "ANDROID", techStack: "FLUTTER", labelKey: "typeFlutterAndroid", hintKey: "appTypeHint" },
  { id: "native-ios", appKind: "IOS_APP", platform: "IOS", techStack: "NATIVE", labelKey: "typeNativeIos", hintKey: "appTypeHint" },
  { id: "native-android", appKind: "ANDROID_APP", platform: "ANDROID", techStack: "NATIVE", labelKey: "typeNativeAndroid", hintKey: "appTypeHint" },
  { id: "harmonyos", appKind: "HARMONYOS_APP", platform: "HARMONYOS", labelKey: "typeHarmonyos", hintKey: "appTypeHint" },
  { id: "api-service", appKind: "API_SERVICE", platform: "API", labelKey: "typeApiService", hintKey: "appTypeHint" },
] as const;

/** Category selection stored in metadata. */
export interface DeployAppCategorySelection {
  readonly id: string
  readonly path: readonly { readonly id: string; readonly label: string }[]
}

/** Drive-backed media reference stored in metadata.media. */
export interface DeployAppMediaRef {
  readonly driveNodeId: string
  readonly driveSpaceId: string
  readonly uploadItemId: string
  readonly uploadSessionId: string
  readonly fileName: string
  readonly contentType: string
  readonly width?: number
  readonly height?: number
  readonly url?: string
}

/** Media group persisted to deploy_app.metadata.media. */
export interface DeployAppMediaGroup {
  readonly icon?: DeployAppMediaRef
  readonly cover?: DeployAppMediaRef
  readonly screenshots: Record<string, readonly DeployAppMediaRef[]>
}

/** Full create-deploy-app dialog model. */
export interface CreateDeployAppInput {
  /** 需求 1: 发布目录（sourceDirectory）。 */
  readonly sourceDirectory: string
  /** 需求 1: 关联已有应用；缺省时创建新应用。 */
  readonly associateAppId?: string
  /** 需求 1: 新应用名称（关联模式可空，由后端推导）。 */
  readonly name?: string
  readonly slug?: string
  /** 需求 2: 应用类型。 */
  readonly type: DeployAppTypeOption
  /** 需求 3: 多级分类。 */
  readonly category?: DeployAppCategorySelection
  /** 需求 4/5/6: 图标/封面/截图（Drive 上传后的引用）。 */
  readonly media?: DeployAppMediaGroup
  /** 需求 7: 版本号。 */
  readonly version: string
  /** 需求 8: 应用描述。 */
  readonly description?: string
  /** 需求 9: release notes。 */
  readonly releaseNotes?: string
  /** 可选：关联站点（静态/SPA 场景）。 */
  readonly siteId?: string
}

/** 需求 7: 语义化版本校验。 */
const SEMVER_PATTERN = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$/;

export function isValidSemver(version: string): boolean {
  return SEMVER_PATTERN.test(version.trim())
}

/** Derive a slug from a name: lowercase, ascii, dash-separated. */
export function deriveAppSlug(name: string): string {
  const normalized = name
    .trim()
    .toLowerCase()
    .replace(/[\s_]+/g, "-")
    .replace(/[^a-z0-9-]/g, "")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "")
  return normalized
}

/** One media upload request. */
export interface DeployAppMediaUpload {
  readonly kind: "icon" | "cover" | "screenshot"
  readonly file: DriveUploaderBlobLike
  readonly fileName: string
  readonly contentType: string
  /** Screenshot target key when kind === "screenshot". */
  readonly targetKey?: string
  readonly width?: number
  readonly height?: number
}

export interface DeployAppPublishingService {
  /** 需求 1: 可关联的已有应用列表。 */
  listApps(params?: { page?: number; pageSize?: number; keyword?: string }): Promise<{ items: AppResponse[]; pageInfo: PageInfo }>
  /** 需求 4/5/6: 上传媒体到 Drive，返回可持久化的引用。 */
  uploadMedia(input: DeployAppMediaUpload, appResourceId: string): Promise<DeployAppMediaRef>
  /** 需求 1/7/8/9: 创建（或更新元数据）应用并写平台目标。 */
  createApp(input: CreateDeployAppInput): Promise<AppResponse>
  /** 给已有应用追加平台目标。 */
  createPlatformTarget(appId: string, request: CreatePlatformTargetRequest): Promise<unknown>
  /** 组装 deploy_app.metadata JSONB。 */
  buildMetadata(input: CreateDeployAppInput): Record<string, unknown>
}

/** Service context: two generated clients + optional idempotency source. */
export interface DeployAppPublishingServiceOptions {
  readonly deployClient: SdkworkDeployAppClient
  readonly driveClient: SdkworkDriveAppClient
  readonly createIdempotencyKey?: () => string
}

/** Drive 上传结果 → 持久化引用（取 console-core artifacts 同一字段集）。 */
export function toDeployAppMediaRef(
  uploaded: DriveUploaderUploadResult,
  meta: Omit<DeployAppMediaRef, "driveNodeId" | "driveSpaceId" | "uploadItemId" | "uploadSessionId">,
): DeployAppMediaRef {
  return {
    driveNodeId: uploaded.uploadItem.nodeId,
    driveSpaceId: uploaded.uploadItem.spaceId,
    uploadItemId: uploaded.uploadItem.id,
    uploadSessionId: uploaded.uploadSession.id,
    fileName: meta.fileName,
    contentType: meta.contentType,
    width: meta.width,
    height: meta.height,
  }
}

export function createDeployAppPublishingService(
  options: DeployAppPublishingServiceOptions,
): DeployAppPublishingService {
  const createIdempotencyKey = options.createIdempotencyKey ?? (() => uuid())
  const { deployClient, driveClient } = options

  return {
    listApps(params) {
      return deployClient.app.list(params)
    },

    async uploadMedia(input, appResourceId) {
      const uploaded = await driveClient.uploader.uploadArchive({
        file: input.file,
        appResourceType: "deploy.app.media",
        appResourceId,
        scene: `deploy-app-${input.kind}`,
        source: "@sdkwork/deployments-pc-console-publishing",
        originalFileName: input.fileName,
        contentType: input.contentType,
      })
      return toDeployAppMediaRef(uploaded, {
        fileName: input.fileName,
        contentType: input.contentType,
        width: input.width,
        height: input.height,
      })
    },

    buildMetadata(input) {
      const metadata: Record<string, unknown> = {
        sourceDirectory: input.sourceDirectory,
        version: input.version.trim(),
        releaseNotes: input.releaseNotes?.trim() || undefined,
        category: input.category
          ? { id: input.category.id, path: input.category.path }
          : undefined,
        media: input.media ?? undefined,
      }
      // Drop explicit undefined so JSONB stays tidy.
      return Object.fromEntries(
        Object.entries(metadata).filter(([, value]) => value !== undefined),
      )
    },

    async createApp(input) {
      const idempotencyKey = createIdempotencyKey()

      // 需求 1: 关联已有应用 → 仅更新元数据（版本/分类/媒体/说明）。
      if (input.associateAppId) {
        const updated = await deployClient.app.update(input.associateAppId, {
          name: input.name?.trim() || undefined,
          description: input.description?.trim() || undefined,
          metadata: this.buildMetadata(input),
        })
        return updated
      }

      // 需求 1: 创建新应用。
      const name = input.name?.trim() ?? input.sourceDirectory.split(/[\\/]/).filter(Boolean).at(-1) ?? "Untitled app"
      const request: CreateAppRequest = {
        name,
        slug: input.slug?.trim() || deriveAppSlug(name) || undefined,
        appKind: input.type.appKind,
        description: input.description?.trim() || undefined,
        siteId: input.siteId,
        metadata: this.buildMetadata(input),
        idempotencyKey,
      }
      const created = await deployClient.app.create(request, { idempotencyKey })

      // 需求 2: 平台目标（deploy_app_platform_target）。
      await this.createPlatformTarget(created.id, {
        targetKey: input.type.id,
        platform: input.type.platform,
        techStack: input.type.techStack,
        idempotencyKey: createIdempotencyKey(),
      })
      return created
    },

    createPlatformTarget(appId, request) {
      return deployClient.app.platformTargets.create(appId, request, {
        idempotencyKey: request.idempotencyKey ?? createIdempotencyKey(),
      })
    },
  }
}

export type { AppStatus };
