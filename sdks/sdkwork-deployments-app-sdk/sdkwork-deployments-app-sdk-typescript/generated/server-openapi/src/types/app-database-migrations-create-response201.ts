import type { AppDatabaseMigrationResponse } from './app-database-migration-response';

export interface AppDatabaseMigrationsCreateResponse201 {
  code: 0;
  data: unknown & { item: AppDatabaseMigrationResponse; };
  /** Server-owned request correlation id. */
  traceId: string;
}
