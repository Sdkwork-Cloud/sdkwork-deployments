import type { AppKind } from './app-kind';

export interface CreateAppRequest {
  name: string;
  slug?: string;
  appKind: AppKind;
  description?: string;
  siteId?: string;
  defaultEnvironment?: string;
  /** Free-form JSONB persisted into deploy_app.metadata (category, media, version, releaseNotes). */
  metadata?: Record<string, unknown>;
  idempotencyKey?: string;
}
