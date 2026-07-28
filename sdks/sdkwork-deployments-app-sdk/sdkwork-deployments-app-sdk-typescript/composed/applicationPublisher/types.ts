import type {
  CreateDeploymentRequest,
  CreateSiteRequest,
  DeploymentResponse,
  ArtifactResponse,
  ReleaseResponse,
  SiteResponse,
} from '../../generated/server-openapi/src/types';
import type { SdkworkDeployAppClient } from '../../generated/server-openapi/src/sdk';
import type {
  DriveUploaderBlobLike,
  DriveUploaderProgress,
  DriveUploaderUploadResult,
  SdkworkDriveAppClient,
} from '@sdkwork/drive-app-sdk';

export type ApplicationPublishStage =
  | 'resolveSite'
  | 'createSite'
  | 'uploadArchive'
  | 'registerArtifact'
  | 'createRelease'
  | 'createDeployment'
  | 'complete';

export type ApplicationPublishErrorCode =
  | 'INVALID_REQUEST'
  | 'SITE_RESOLUTION_AMBIGUOUS'
  | 'SITE_RESPONSE_MISSING_ID'
  | 'SITE_ID_MISMATCH'
  | 'UPLOAD_RESPONSE_INCOMPLETE'
  | 'ARTIFACT_RESPONSE_MISSING_ID'
  | 'RELEASE_RESPONSE_MISSING_ID'
  | 'DEPLOYMENT_RESPONSE_MISSING_ID'
  | 'IDEMPOTENCY_KEY_UNAVAILABLE'
  | 'ABORTED'
  | 'STAGE_FAILED';

export interface ExistingApplicationPublishSite {
  kind: 'existing';
  siteId: string;
}

export interface ResolveOrCreateApplicationPublishSite extends CreateSiteRequest {
  kind: 'resolveOrCreate';
}

export type ApplicationPublishSite =
  | ExistingApplicationPublishSite
  | ResolveOrCreateApplicationPublishSite;

export interface ApplicationPublishArtifact {
  file: DriveUploaderBlobLike;
  packageType: number;
  fileName: string;
  contentType: string;
  checksumSha256: string;
  taskId?: string;
  chunkSizeBytes?: number;
  scene?: string;
  source?: string;
}

export interface ApplicationPublishRelease {
  versionTag?: string;
}

export type ApplicationPublishDeployment = Omit<
  CreateDeploymentRequest,
  'releaseId' | 'idempotencyKey'
>;

export interface ApplicationPublishIdempotencyKeys {
  artifact?: string;
  release?: string;
  deployment?: string;
}

export interface ApplicationPublishRequest {
  site: ApplicationPublishSite;
  artifact: ApplicationPublishArtifact;
  release?: ApplicationPublishRelease;
  deployment?: ApplicationPublishDeployment;
  idempotencyKeys?: ApplicationPublishIdempotencyKeys;
  signal?: AbortSignal;
  onProgress?: ApplicationPublishProgressCallback;
}

export type ApplicationPublishSiteResolution =
  | 'existingById'
  | 'existingBySlug'
  | 'existingByName'
  | 'created';

export interface ApplicationPublishSiteEvidence {
  id: string;
  resolution: ApplicationPublishSiteResolution;
  value: SiteResponse;
}

export interface ApplicationPublishUploadEvidence {
  uploadItemId: string;
  uploadSessionId: string;
  driveSpaceId: string;
  driveNodeId: string;
  value: DriveUploaderUploadResult;
}

export interface ApplicationPublishArtifactEvidence {
  id: string;
  value: ArtifactResponse;
}

export interface ApplicationPublishReleaseEvidence {
  id: string;
  value: ReleaseResponse;
}

export interface ApplicationPublishDeploymentEvidence {
  id: string;
  value: DeploymentResponse;
}

export interface ApplicationPublishResult {
  site: ApplicationPublishSiteEvidence;
  upload: ApplicationPublishUploadEvidence;
  artifact: ApplicationPublishArtifactEvidence;
  release: ApplicationPublishReleaseEvidence;
  deployment?: ApplicationPublishDeploymentEvidence;
}

export interface ApplicationPublishProgressEvidence {
  siteId?: string;
  uploadItemId?: string;
  uploadSessionId?: string;
  artifactId?: string;
  releaseId?: string;
  deploymentId?: string;
}

export interface ApplicationPublishStageProgress {
  kind: 'stage';
  stage: ApplicationPublishStage;
  status: 'started' | 'completed';
  evidence: ApplicationPublishProgressEvidence;
}

export interface ApplicationPublishUploadProgress {
  kind: 'upload';
  stage: 'uploadArchive';
  status: DriveUploaderProgress['status'];
  uploadedBytes: number;
  totalBytes: number;
  uploadedPartsCount: number;
  totalParts: number;
  partNo?: number;
  evidence: ApplicationPublishProgressEvidence;
}

export interface ApplicationPublishFailureProgress {
  kind: 'failure';
  stage: ApplicationPublishStage;
  status: 'failed';
  error: {
    code: ApplicationPublishErrorCode;
    message: string;
  };
  evidence: ApplicationPublishProgressEvidence;
}

export type ApplicationPublishProgress =
  | ApplicationPublishStageProgress
  | ApplicationPublishUploadProgress
  | ApplicationPublishFailureProgress;

export type ApplicationPublishProgressCallback = (
  progress: ApplicationPublishProgress,
) => void;

export interface ApplicationPublisherDeployClient {
  readonly site: Pick<SdkworkDeployAppClient['site'], 'create' | 'list' | 'retrieve'>;
  readonly artifact: Pick<SdkworkDeployAppClient['artifact'], 'create'>;
  readonly release: {
    readonly sites: {
      readonly releases: Pick<
        SdkworkDeployAppClient['release']['sites']['releases'],
        'create'
      >;
    };
  };
  readonly deployment: {
    readonly sites: {
      readonly deployments: Pick<
        SdkworkDeployAppClient['deployment']['sites']['deployments'],
        'create'
      >;
    };
  };
}

export interface ApplicationPublisherDriveClient {
  readonly uploader: Pick<SdkworkDriveAppClient['uploader'], 'uploadArchive'>;
}

export interface DeployApplicationPublisherOptions {
  deployClient: ApplicationPublisherDeployClient;
  driveClient: ApplicationPublisherDriveClient;
  createIdempotencyKey?: () => string;
}

export interface DeployApplicationPublisher {
  publish(request: ApplicationPublishRequest): Promise<ApplicationPublishResult>;
}
