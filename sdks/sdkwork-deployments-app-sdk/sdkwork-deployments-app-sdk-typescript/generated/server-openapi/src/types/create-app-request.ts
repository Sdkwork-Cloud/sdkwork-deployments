import type { AppKind } from './app-kind';

export interface CreateAppRequest {
  name: string;
  slug?: string;
  appKind: AppKind;
  description?: string;
  siteId?: string;
  defaultEnvironment?: string;
  idempotencyKey?: string;
}
