import type { AppKind } from './app-kind';

export interface CreateAppRequest {
  name: string;
  slug?: string;
  appKind: AppKind;
  description?: string;
  siteId?: string;
  defaultEnvironment?: string;
  /**
   * Free-form JSONB persisted verbatim into deploy_app.metadata.
   * The create-deploy-app dialog stores { category, media, version, releaseNotes } here.
   */
  metadata?: Record<string, unknown>;
  idempotencyKey?: string;
}
