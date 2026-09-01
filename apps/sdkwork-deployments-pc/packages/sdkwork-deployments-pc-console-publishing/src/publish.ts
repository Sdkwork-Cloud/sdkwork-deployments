/**
 * Publishing capability surface: the reusable create-deploy-app components and
 * services. `src/index.ts` re-exports this barrel alongside the module
 * registration, so hosts can import the whole capability from the package root
 * (console shell + BirdCoder plugin) without reaching into package internals.
 */
export { CreateDeployAppDialog } from "./components/CreateDeployAppDialog.tsx";
export type { CreateDeployAppDialogProps, DeployAppPublishResult } from "./components/CreateDeployAppDialog.tsx";
export { PublishingAppsPage } from "./components/PublishingAppsPage.tsx";
export type { PublishingAppsPageProps } from "./components/PublishingAppsPage.tsx";
export { CategoryCascadeSelect } from "./components/CategoryCascadeSelect.tsx";
export { DeployAppTypeSelect } from "./components/DeployAppTypeSelect.tsx";
export { DeployAppMediaFields, screenshotTargets } from "./components/DeployAppMediaFields.tsx";
export type { DeployAppMediaFiles, DeployAppMediaFieldsProps } from "./components/DeployAppMediaFields.tsx";
export {
  createDeployAppPublishingService,
  deriveAppSlug,
  isValidSemver,
  toDeployAppMediaRef,
  DEPLOY_APP_TYPE_OPTIONS,
} from "./service/deploy-app-publishing.ts";
export type {
  CreateDeployAppInput,
  DeployAppCategorySelection,
  DeployAppMediaGroup,
  DeployAppMediaRef,
  DeployAppMediaUpload,
  DeployAppPublishingService,
  DeployAppPublishingServiceOptions,
  DeployAppTypeOption,
} from "./service/deploy-app-publishing.ts";
export {
  DEPLOY_APP_CATEGORY_TREE,
  categoriesForAppKind,
  findCategoryNode,
  categoryPathTo,
} from "./service/app-categories.ts";
export type { DeployAppCategoryNode } from "./service/app-categories.ts";
export {
  APP_ICON_SPEC,
  APP_STORE_PREVIEW_TARGETS,
  COVER_SPEC,
  MAX_SCREENSHOTS_TOTAL,
  MEDIA_ACCEPTED_TYPES,
  PREVIEW_ASPECT_TOLERANCE,
  previewTargetsForAppKind,
  validatePreviewSize,
} from "./service/app-store-preview-spec.ts";
export type { PreviewSizeTarget, PreviewValidationResult } from "./service/app-store-preview-spec.ts";
export { publishingText, publishingTranslator } from "./i18n.ts";
export type { PublishingMessageKey, PublishingTranslator } from "./i18n.ts";
