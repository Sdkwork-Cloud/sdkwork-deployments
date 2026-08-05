import type { AppDatabaseProfileResponse } from './app-database-profile-response';

export interface AppDatabaseProfilesUpdateResponse {
  code: 0;
  data: unknown & { item: AppDatabaseProfileResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
