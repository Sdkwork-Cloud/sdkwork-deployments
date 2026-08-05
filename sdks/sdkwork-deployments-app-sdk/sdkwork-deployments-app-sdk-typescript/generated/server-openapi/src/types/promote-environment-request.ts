export interface PromoteEnvironmentRequest {
  releaseId: string;
  fromEnvironmentId?: string;
  note?: string;
}
