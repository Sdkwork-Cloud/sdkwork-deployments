export interface RunnerHealthResponse {
  runnerNodeUuid: string;
  runnerVersion?: string;
  lastSeenAt: string;
  buildsCompleted: string;
  buildsFailed: string;
  activeBuilds: string;
}
