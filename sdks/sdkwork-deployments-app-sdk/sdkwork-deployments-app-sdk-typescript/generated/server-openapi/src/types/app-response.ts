import type { AppKind } from './app-kind';
import type { AppStatus } from './app-status';

export interface AppResponse {
  id: string;
  name: string;
  slug: string;
  appKind: AppKind;
  appStatus: AppStatus;
  description?: string;
  siteId?: string;
  defaultEnvironment: string;
  /** Echo of deploy_app.metadata (category / media / version / releaseNotes). */
  metadata?: Record<string, unknown>;
  platformTargetCount?: string;
  latestReleaseTag?: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
