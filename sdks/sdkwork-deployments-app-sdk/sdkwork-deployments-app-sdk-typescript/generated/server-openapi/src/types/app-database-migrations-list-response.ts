import type { AppDatabaseMigrationResponse } from './app-database-migration-response';
import type { PageInfo } from './page-info';

export interface AppDatabaseMigrationsListResponse {
  code: 0;
  data: unknown & { items: AppDatabaseMigrationResponse[]; pageInfo: PageInfo; };
  /** Server-owned request correlation id. */
  traceId: string;
}
