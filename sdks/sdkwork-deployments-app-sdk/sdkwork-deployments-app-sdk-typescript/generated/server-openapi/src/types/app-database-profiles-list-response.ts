import type { AppDatabaseProfileResponse } from './app-database-profile-response';
import type { PageInfo } from './page-info';

export interface AppDatabaseProfilesListResponse {
  code: 0;
  data: unknown & { items: AppDatabaseProfileResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
