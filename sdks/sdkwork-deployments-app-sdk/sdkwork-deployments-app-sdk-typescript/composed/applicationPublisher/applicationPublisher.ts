import { ApplicationPublishError, toApplicationPublishError } from './errors';
import {
  createdApplicationPublishSiteEvidence,
  findExactApplicationPublishSite,
  retrieveApplicationPublishSite,
} from './siteResolver';
import type {
  ApplicationPublishArtifact,
  ApplicationPublishProgress,
  ApplicationPublishProgressEvidence,
  ApplicationPublishRequest,
  ApplicationPublishResult,
  ApplicationPublishSiteEvidence,
  ApplicationPublishStage,
  DeployApplicationPublisher,
  DeployApplicationPublisherOptions,
} from './types';

const DEFAULT_UPLOAD_SCENE = 'deployment-package';
const DEFAULT_UPLOAD_SOURCE = '@sdkwork/deployments-app-sdk';
const SHA256_HEX_PATTERN = /^[a-fA-F0-9]{64}$/;

export function createDeployApplicationPublisher(
  options: DeployApplicationPublisherOptions,
): DeployApplicationPublisher {
  const createIdempotencyKey =
    options.createIdempotencyKey ?? createRuntimeIdempotencyKey;

  return {
    async publish(request): Promise<ApplicationPublishResult> {
      let currentStage: ApplicationPublishStage = 'resolveSite';
      const evidence: ApplicationPublishProgressEvidence = {};
      const emit = (progress: ApplicationPublishProgress): void => {
        try {
          request.onProgress?.(progress);
        } catch {
          // Progress observers cannot interrupt an operation after remote side effects begin.
        }
      };
      const startStage = (stage: ApplicationPublishStage): void => {
        currentStage = stage;
        emit({ kind: 'stage', stage, status: 'started', evidence: { ...evidence } });
      };
      const completeStage = (stage: ApplicationPublishStage): void => {
        emit({ kind: 'stage', stage, status: 'completed', evidence: { ...evidence } });
      };

      try {
        const normalizedArtifact = validateRequest(request);
        throwIfAborted(request.signal, currentStage);

        startStage('resolveSite');
        let siteEvidence: ApplicationPublishSiteEvidence;
        if (request.site.kind === 'existing') {
          siteEvidence = await retrieveApplicationPublishSite(
            options.deployClient,
            request.site.siteId.trim(),
            request.signal,
          );
        } else {
          const existing = await findExactApplicationPublishSite(
            options.deployClient,
            request.site,
            request.signal,
          );
          if (existing) {
            siteEvidence = existing;
          } else {
            completeStage('resolveSite');
            startStage('createSite');
            const created = await options.deployClient.site.create(
              {
                name: request.site.name.trim(),
                slug: normalizedOptionalText(request.site.slug),
                description: normalizedOptionalText(request.site.description),
                siteType: request.site.siteType,
                runtimeConfig: request.site.runtimeConfig,
              },
              {
                idempotencyKey: resolveIdempotencyKey(
                  request.idempotencyKeys?.site,
                  createIdempotencyKey,
                  'createSite',
                ),
              },
              { signal: request.signal, timeout: undefined },
            );
            siteEvidence = createdApplicationPublishSiteEvidence(created);
            evidence.siteId = siteEvidence.id;
            completeStage('createSite');
          }
        }
        evidence.siteId = siteEvidence.id;
        if (currentStage === 'resolveSite') {
          completeStage('resolveSite');
        }

        throwIfAborted(request.signal, 'uploadArchive');
        startStage('uploadArchive');
        const upload = await options.driveClient.uploader.uploadArchive({
          file: normalizedArtifact.file,
          taskId: normalizedOptionalText(normalizedArtifact.taskId),
          appResourceType: 'deploy.artifact',
          appResourceId: siteEvidence.id,
          scene: normalizedOptionalText(normalizedArtifact.scene) ?? DEFAULT_UPLOAD_SCENE,
          source: normalizedOptionalText(normalizedArtifact.source) ?? DEFAULT_UPLOAD_SOURCE,
          originalFileName: normalizedArtifact.fileName,
          contentType: normalizedArtifact.contentType,
          checksumSha256Hex: normalizedArtifact.checksumSha256,
          chunkSizeBytes: normalizedArtifact.chunkSizeBytes,
          signal: request.signal,
          onProgress: (progress) => {
            evidence.uploadItemId = progress.uploadItemId;
            evidence.uploadSessionId = progress.uploadSessionId;
            emit({
              kind: 'upload',
              stage: 'uploadArchive',
              status: progress.status,
              uploadedBytes: progress.uploadedBytes,
              totalBytes: progress.totalBytes,
              uploadedPartsCount: progress.uploadedPartsCount,
              totalParts: progress.totalParts,
              partNo: progress.partNo,
              evidence: { ...evidence },
            });
          },
        });
        const uploadEvidence = requireUploadEvidence(upload);
        evidence.uploadItemId = uploadEvidence.uploadItemId;
        evidence.uploadSessionId = uploadEvidence.uploadSessionId;
        completeStage('uploadArchive');

        throwIfAborted(request.signal, 'registerArtifact');
        startStage('registerArtifact');
        const artifactIdempotencyKey = resolveIdempotencyKey(
          request.idempotencyKeys?.artifact,
          createIdempotencyKey,
          'registerArtifact',
        );
        const artifact = await options.deployClient.artifact.create(
          {
            siteId: siteEvidence.id,
            packageType: normalizedArtifact.packageType,
            fileName: normalizedArtifact.fileName,
            contentType: normalizedArtifact.contentType,
            contentLength: String(normalizedArtifact.file.size),
            checksumSha256: normalizedArtifact.checksumSha256,
            driveUploadSessionId: uploadEvidence.uploadSessionId,
            driveUploadItemId: uploadEvidence.uploadItemId,
            driveSpaceId: uploadEvidence.driveSpaceId,
            driveNodeId: uploadEvidence.driveNodeId,
            idempotencyKey: artifactIdempotencyKey,
          },
          { idempotencyKey: artifactIdempotencyKey },
          { signal: request.signal, timeout: undefined },
        );
        const artifactId = requireResponseId(
          artifact.id,
          'ARTIFACT_RESPONSE_MISSING_ID',
          'registerArtifact',
          'Artifact',
        );
        evidence.artifactId = artifactId;
        completeStage('registerArtifact');

        throwIfAborted(request.signal, 'createRelease');
        startStage('createRelease');
        const releaseIdempotencyKey = resolveIdempotencyKey(
          request.idempotencyKeys?.release,
          createIdempotencyKey,
          'createRelease',
        );
        const release = await options.deployClient.release.sites.releases.create(
          siteEvidence.id,
          {
            artifactId,
            versionTag: normalizedOptionalText(request.release?.versionTag),
            idempotencyKey: releaseIdempotencyKey,
          },
          { idempotencyKey: releaseIdempotencyKey },
          { signal: request.signal, timeout: undefined },
        );
        const releaseId = requireResponseId(
          release.id,
          'RELEASE_RESPONSE_MISSING_ID',
          'createRelease',
          'Release',
        );
        evidence.releaseId = releaseId;
        completeStage('createRelease');

        let deploymentEvidence: ApplicationPublishResult['deployment'];
        if (request.deployment) {
          throwIfAborted(request.signal, 'createDeployment');
          startStage('createDeployment');
          const deploymentIdempotencyKey = resolveIdempotencyKey(
            request.idempotencyKeys?.deployment,
            createIdempotencyKey,
            'createDeployment',
          );
          const deployment =
            await options.deployClient.deployment.sites.deployments.create(
              siteEvidence.id,
              {
                ...request.deployment,
                releaseId,
                idempotencyKey: deploymentIdempotencyKey,
              },
              { idempotencyKey: deploymentIdempotencyKey },
              { signal: request.signal, timeout: undefined },
            );
          const deploymentId = requireResponseId(
            deployment.id,
            'DEPLOYMENT_RESPONSE_MISSING_ID',
            'createDeployment',
            'Deployment',
          );
          evidence.deploymentId = deploymentId;
          deploymentEvidence = { id: deploymentId, value: deployment };
          completeStage('createDeployment');
        }

        startStage('complete');
        const result: ApplicationPublishResult = {
          site: siteEvidence,
          upload: uploadEvidence,
          artifact: { id: artifactId, value: artifact },
          release: { id: releaseId, value: release },
          deployment: deploymentEvidence,
        };
        completeStage('complete');
        return result;
      } catch (cause) {
        const error = toApplicationPublishError(
          cause,
          currentStage,
          request.signal?.aborted === true,
        );
        emit({
          kind: 'failure',
          stage: error.stage,
          status: 'failed',
          error: { code: error.code, message: error.message },
          evidence: { ...evidence },
        });
        throw error;
      }
    },
  };
}

function validateRequest(request: ApplicationPublishRequest): ApplicationPublishArtifact {
  if (request.site.kind === 'existing') {
    requireText(request.site.siteId, 'site.siteId');
  } else {
    requireText(request.site.name, 'site.name');
    if (
      !Number.isInteger(request.site.siteType) ||
      request.site.siteType < 1 ||
      request.site.siteType > 6
    ) {
      throw invalidRequest('site.siteType must be a supported Site type.');
    }
  }

  const artifact = request.artifact;
  if (!artifact.file || !Number.isFinite(artifact.file.size) || artifact.file.size <= 0) {
    throw invalidRequest('artifact.file must contain a non-empty package.');
  }
  if (!Number.isInteger(artifact.packageType) || artifact.packageType <= 0) {
    throw invalidRequest('artifact.packageType must be a positive integer.');
  }
  const fileName = requireText(artifact.fileName, 'artifact.fileName');
  const contentType = requireText(artifact.contentType, 'artifact.contentType');
  const checksumSha256 = requireText(
    artifact.checksumSha256,
    'artifact.checksumSha256',
  );
  if (!SHA256_HEX_PATTERN.test(checksumSha256)) {
    throw invalidRequest('artifact.checksumSha256 must be a 64-character SHA-256 hex digest.');
  }
  if (
    artifact.chunkSizeBytes !== undefined &&
    (!Number.isInteger(artifact.chunkSizeBytes) || artifact.chunkSizeBytes <= 0)
  ) {
    throw invalidRequest('artifact.chunkSizeBytes must be a positive integer when provided.');
  }
  if (
    request.deployment &&
    (!Number.isInteger(request.deployment.deployType) ||
      request.deployment.deployType < 1 ||
      request.deployment.deployType > 4)
  ) {
    throw invalidRequest('deployment.deployType must be between 1 and 4.');
  }

  return {
    ...artifact,
    fileName,
    contentType,
    checksumSha256: checksumSha256.toLowerCase(),
  };
}

function requireUploadEvidence(
  value: Awaited<
    ReturnType<DeployApplicationPublisherOptions['driveClient']['uploader']['uploadArchive']>
  >,
): ApplicationPublishResult['upload'] {
  const uploadItemId = normalizedOptionalText(value.uploadItem?.id);
  const uploadSessionId = normalizedOptionalText(value.uploadSession?.id);
  const driveSpaceId = normalizedOptionalText(value.uploadItem?.spaceId);
  const driveNodeId = normalizedOptionalText(value.uploadItem?.nodeId);
  if (!uploadItemId || !uploadSessionId || !driveSpaceId || !driveNodeId) {
    throw new ApplicationPublishError(
      'UPLOAD_RESPONSE_INCOMPLETE',
      'uploadArchive',
      'Drive upload response did not include stable item, session, space, and node references.',
    );
  }
  return {
    uploadItemId,
    uploadSessionId,
    driveSpaceId,
    driveNodeId,
    value,
  };
}

function requireResponseId(
  value: string | undefined,
  code:
    | 'ARTIFACT_RESPONSE_MISSING_ID'
    | 'RELEASE_RESPONSE_MISSING_ID'
    | 'DEPLOYMENT_RESPONSE_MISSING_ID',
  stage: 'registerArtifact' | 'createRelease' | 'createDeployment',
  resource: string,
): string {
  const id = normalizedOptionalText(value);
  if (!id) {
    throw new ApplicationPublishError(
      code,
      stage,
      `Deploy ${resource} response did not include an id.`,
    );
  }
  return id;
}

function resolveIdempotencyKey(
  value: string | undefined,
  createIdempotencyKey: () => string,
  stage: 'createSite' | 'registerArtifact' | 'createRelease' | 'createDeployment',
): string {
  if (value !== undefined) {
    return requireText(value, 'idempotency key');
  }
  const generated = normalizedOptionalText(createIdempotencyKey());
  if (!generated) {
    throw new ApplicationPublishError(
      'IDEMPOTENCY_KEY_UNAVAILABLE',
      stage,
      'The idempotency key factory returned an empty value.',
    );
  }
  return generated;
}

function createRuntimeIdempotencyKey(): string {
  if (typeof globalThis.crypto?.randomUUID !== 'function') {
    throw new ApplicationPublishError(
      'IDEMPOTENCY_KEY_UNAVAILABLE',
      'registerArtifact',
      'crypto.randomUUID is required to publish an application.',
    );
  }
  return globalThis.crypto.randomUUID();
}

function throwIfAborted(
  signal: AbortSignal | undefined,
  stage: ApplicationPublishStage,
): void {
  if (signal?.aborted) {
    throw new ApplicationPublishError(
      'ABORTED',
      stage,
      'Application publishing was cancelled.',
      signal.reason,
    );
  }
}

function requireText(value: string, field: string): string {
  const normalized = value.trim();
  if (!normalized) {
    throw invalidRequest(`${field} is required.`);
  }
  return normalized;
}

function normalizedOptionalText(value: string | undefined): string | undefined {
  const normalized = value?.trim();
  return normalized || undefined;
}

function invalidRequest(message: string): ApplicationPublishError {
  return new ApplicationPublishError(
    'INVALID_REQUEST',
    'resolveSite',
    message,
  );
}
