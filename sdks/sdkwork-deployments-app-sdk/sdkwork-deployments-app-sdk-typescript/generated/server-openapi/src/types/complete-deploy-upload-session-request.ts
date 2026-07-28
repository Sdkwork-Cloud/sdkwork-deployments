import type { CompletedUploadPartInput } from './completed-upload-part-input';

export interface CompleteDeployUploadSessionRequest {
  checksumSha256Hex: string;
  contentLength?: string;
  contentType?: string;
  parts?: CompletedUploadPartInput[];
}
