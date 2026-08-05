export interface CreateAppDatabaseMigrationRequest {
  migrationVersion: string;
  migrationName: string;
  checksumSha256: string;
  scriptRef?: string;
}
