export interface AppDatabaseMigrationResponse {
  id: string;
  profileId: string;
  migrationVersion: string;
  migrationName: string;
  checksumSha256: string;
  scriptRef?: string;
  migrationStatus: string;
  appliedAt?: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
