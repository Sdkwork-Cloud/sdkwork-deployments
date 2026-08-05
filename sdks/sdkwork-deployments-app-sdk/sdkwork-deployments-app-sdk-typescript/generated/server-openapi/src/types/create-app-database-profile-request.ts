export interface CreateAppDatabaseProfileRequest {
  profileKey: string;
  dbEngine: 'POSTGRES' | 'MYSQL' | 'SQLITE';
  catalogName: string;
  schemaVersion?: string;
  baselineVersion?: string;
  migrationStrategy?: 'VERSIONED' | 'REPEATABLE';
}
