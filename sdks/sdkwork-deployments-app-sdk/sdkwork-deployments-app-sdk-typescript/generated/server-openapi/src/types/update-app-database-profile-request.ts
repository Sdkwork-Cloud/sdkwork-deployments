export interface UpdateAppDatabaseProfileRequest {
  schemaVersion?: string;
  baselineVersion?: string;
  migrationStrategy?: 'VERSIONED' | 'REPEATABLE';
  profileStatus?: 'DRAFT' | 'READY' | 'ACTIVE' | 'SUPERSEDED' | 'ARCHIVED';
}
