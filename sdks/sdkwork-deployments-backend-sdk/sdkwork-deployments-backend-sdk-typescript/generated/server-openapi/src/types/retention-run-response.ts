export interface RetentionRunResponse {
  dryRun: boolean;
  packagesRetired: string;
  releasesRetired: string;
  buildLogsPurged: string;
  packageRetentionDays: string;
  releaseRetentionDays: string;
  buildLogRetentionDays: string;
}
