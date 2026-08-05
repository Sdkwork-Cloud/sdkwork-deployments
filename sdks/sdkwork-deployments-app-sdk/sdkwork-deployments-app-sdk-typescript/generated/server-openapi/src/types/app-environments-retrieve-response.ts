import type { AppEnvironmentResponse } from './app-environment-response';

export interface AppEnvironmentsRetrieveResponse {
  code: 0;
  data: unknown & { item: AppEnvironmentResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
