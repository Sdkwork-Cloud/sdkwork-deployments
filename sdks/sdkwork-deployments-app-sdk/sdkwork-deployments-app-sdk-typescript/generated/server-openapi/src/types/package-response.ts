import type { PackageFormat } from './package-format';
import type { PackageStatus } from './package-status';

export interface PackageResponse {
  id: string;
  appId: string;
  platformTargetId: string;
  buildId: string;
  packageFormat: PackageFormat;
  semanticVersion: string;
  packageSizeBytes: string;
  checksumSha256: string;
  manifestSha256: string;
  driveNodeId?: string;
  signingIdentityId?: string;
  minPlatformVersion?: string;
  architectures?: string[];
  packageStatus: PackageStatus;
  createdAt: string;
  updatedAt: string;
  version: string;
}
