export interface AppDatabaseProfileResponse {
  id: string;
  appId: string;
  profileKey: string;
  dbEngine: string;
  catalogName: string;
  schemaVersion?: string;
  baselineVersion?: string;
  migrationStrategy: string;
  profileStatus: string;
  migrationCount: string;
  createdAt: string;
  updatedAt: string;
  version: string;
}
