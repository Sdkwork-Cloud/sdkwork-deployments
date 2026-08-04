import type { PackageFormat } from './package-format';

export interface RegisterPackageRequest {
  platformTargetId: string;
  buildId: string;
  packageFormat: PackageFormat;
  semanticVersion: string;
  packageSizeBytes: string;
  checksumSha256: string;
  manifestSha256: string;
  driveNodeId: string;
  driveSpaceId?: string;
  signingIdentityId?: string;
  minPlatformVersion?: string;
  architectures?: string[];
  bundleIdentity?: Record<string, unknown>;
  validationReport?: Record<string, unknown>;
  idempotencyKey?: string;
}
