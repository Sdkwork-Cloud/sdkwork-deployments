import type { AppStatus } from './app-status';

export interface UpdateAppRequest {
  name?: string;
  description?: string;
  appStatus?: AppStatus;
  defaultEnvironment?: string;
  /** Free-form JSONB merged into deploy_app.metadata (shallow per-key merge). */
  metadata?: Record<string, unknown>;
}
