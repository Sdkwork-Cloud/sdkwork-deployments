import { describe, expect, it, vi } from 'vitest';
import type {
  DriveUploaderBlobLike,
  DriveUploaderRequest,
  DriveUploaderUploadResult,
} from '@sdkwork/drive-app-sdk';
import {
  createDeployApplicationPublisher,
  type ApplicationPublishDeployment,
  type ApplicationPublishProgress,
  type ApplicationPublishRequest,
  type ApplicationPublisherDeployClient,
  type ApplicationPublisherDriveClient,
} from '../composed/applicationPublisher';

const CHECKSUM_SHA256 = 'a'.repeat(64);

function packageFile(): DriveUploaderBlobLike {
  const bytes = new Uint8Array([1, 2, 3, 4]);
  return {
    name: 'application.zip',
    type: 'application/zip',
    size: bytes.byteLength,
    slice(start = 0, end = bytes.byteLength, contentType = 'application/zip') {
      return new Blob([bytes.slice(start, end)], { type: contentType });
    },
    async readRange(offsetBytes, lengthBytes) {
      return bytes.slice(offsetBytes, offsetBytes + lengthBytes).buffer;
    },
  };
}

function publishRequest(
  overrides: Omit<Partial<ApplicationPublishRequest>, 'deployment'> & {
    deployment?: ApplicationPublishDeployment | undefined;
  } = {},
): ApplicationPublishRequest {
  const { deployment, ...rest } = overrides;
  return {
    site: {
      kind: 'resolveOrCreate',
      name: 'BirdCoder',
      slug: 'birdcoder',
      siteType: 1,
    },
    artifact: {
      file: packageFile(),
      packageType: 1,
      fileName: 'application.zip',
      contentType: 'application/zip',
      checksumSha256: CHECKSUM_SHA256,
      source: 'sdkwork-birdcoder-pc',
    },
    release: { versionTag: '1.2.3' },
    ...(deployment !== undefined
      ? { deployment }
      : 'deployment' in overrides
        ? {}
        : { deployment: { deployType: 1, environment: 'production' } }),
    ...rest,
  };
}

interface MockClientOptions {
  siteList?: (keyword: string | undefined) => Promise<unknown>;
  siteRetrieve?: () => Promise<unknown>;
  siteCreate?: () => Promise<unknown>;
  upload?: (request: DriveUploaderRequest) => Promise<unknown>;
  artifactCreate?: () => Promise<unknown>;
  releaseCreate?: () => Promise<unknown>;
  deploymentCreate?: () => Promise<unknown>;
}

function createMockClients(options: MockClientOptions = {}) {
  const calls: string[] = [];
  const siteList = vi.fn(async (params: { keyword?: string }) => {
    calls.push(`site.list:${params.keyword ?? ''}`);
    return (
      (await options.siteList?.(params.keyword)) ?? {
        items: [{ id: 'site-1', name: 'BirdCoder', slug: 'birdcoder' }],
        pageInfo: { mode: 'offset', page: 1, pageSize: 50, hasMore: false },
      }
    );
  });
  const siteRetrieve = vi.fn(async () => {
    calls.push('site.retrieve');
    return (await options.siteRetrieve?.()) ?? { id: 'site-1', name: 'BirdCoder' };
  });
  const siteCreate = vi.fn(async () => {
    calls.push('site.create');
    return (await options.siteCreate?.()) ?? { id: 'site-1', name: 'BirdCoder' };
  });
  const uploadArchive = vi.fn(async (request: DriveUploaderRequest) => {
    calls.push('drive.uploadArchive');
    request.onProgress?.({
      taskId: 'task-1',
      uploadItemId: 'upload-item-1',
      uploadSessionId: 'upload-session-1',
      nodeId: 'node-1',
      uploadedBytes: request.file.size,
      totalBytes: request.file.size,
      uploadedPartsCount: 1,
      totalParts: 1,
      status: 'completed',
    });
    return (
      (await options.upload?.(request)) ?? {
        uploadItem: {
          id: 'upload-item-1',
          spaceId: 'space-1',
          nodeId: 'node-1',
        },
        uploadSession: { id: 'upload-session-1' },
        parts: [],
      }
    );
  });
  const artifactCreate = vi.fn(async () => {
    calls.push('artifact.create');
    return (await options.artifactCreate?.()) ?? { id: 'artifact-1' };
  });
  const releaseCreate = vi.fn(async () => {
    calls.push('release.create');
    return (await options.releaseCreate?.()) ?? { id: 'release-1' };
  });
  const deploymentCreate = vi.fn(async () => {
    calls.push('deployment.create');
    return (await options.deploymentCreate?.()) ?? { id: 'deployment-1' };
  });

  const deployClient = {
    site: {
      list: siteList,
      retrieve: siteRetrieve,
      create: siteCreate,
    },
    artifact: { create: artifactCreate },
    release: { sites: { releases: { create: releaseCreate } } },
    deployment: { sites: { deployments: { create: deploymentCreate } } },
  } as unknown as ApplicationPublisherDeployClient;
  const driveClient = {
    uploader: { uploadArchive },
  } as unknown as ApplicationPublisherDriveClient;

  return {
    calls,
    deployClient,
    driveClient,
    mocks: {
      siteList,
      siteRetrieve,
      siteCreate,
      uploadArchive,
      artifactCreate,
      releaseCreate,
      deploymentCreate,
    },
  };
}

function publisher(
  clients: ReturnType<typeof createMockClients>,
) {
  let idempotencyIndex = 0;
  return createDeployApplicationPublisher({
    deployClient: clients.deployClient,
    driveClient: clients.driveClient,
    createIdempotencyKey: () => `idempotency-${++idempotencyIndex}`,
  });
}

describe('createDeployApplicationPublisher', () => {
  it('publishes in the frozen site, upload, artifact, release, deployment order', async () => {
    const clients = createMockClients();
    const progress: ApplicationPublishProgress[] = [];

    const result = await publisher(clients).publish(
      publishRequest({ onProgress: (event) => progress.push(event) }),
    );

    expect(clients.calls).toEqual([
      'site.list:birdcoder',
      'drive.uploadArchive',
      'artifact.create',
      'release.create',
      'deployment.create',
    ]);
    expect(result).toMatchObject({
      site: { id: 'site-1', resolution: 'existingBySlug' },
      upload: {
        uploadItemId: 'upload-item-1',
        uploadSessionId: 'upload-session-1',
        driveSpaceId: 'space-1',
        driveNodeId: 'node-1',
      },
      artifact: { id: 'artifact-1' },
      release: { id: 'release-1' },
      deployment: { id: 'deployment-1' },
    });
    expect(clients.mocks.uploadArchive).toHaveBeenCalledWith(
      expect.objectContaining({
        appResourceType: 'deploy.artifact',
        appResourceId: 'site-1',
        checksumSha256Hex: CHECKSUM_SHA256,
        source: 'sdkwork-birdcoder-pc',
      }),
    );
    expect(clients.mocks.artifactCreate).toHaveBeenCalledWith(
      expect.objectContaining({
        siteId: 'site-1',
        driveUploadSessionId: 'upload-session-1',
        driveUploadItemId: 'upload-item-1',
        idempotencyKey: 'idempotency-1',
      }),
      { idempotencyKey: 'idempotency-1' },
      { signal: undefined, timeout: undefined },
    );
    expect(progress).toContainEqual(
      expect.objectContaining({
        kind: 'upload',
        stage: 'uploadArchive',
        status: 'completed',
        uploadedBytes: 4,
        totalBytes: 4,
      }),
    );
  });

  it('falls back from an exact slug lookup to an exact name lookup', async () => {
    const clients = createMockClients({
      siteList: async (keyword) => ({
        items:
          keyword === 'BirdCoder'
            ? [{ id: 'site-by-name', name: 'BirdCoder', slug: 'legacy-slug' }]
            : [{ id: 'unrelated', name: 'Other', slug: 'other' }],
        pageInfo: { mode: 'offset', page: 1, pageSize: 50, hasMore: false },
      }),
    });

    const result = await publisher(clients).publish(
      publishRequest({ deployment: undefined }),
    );

    expect(clients.calls.slice(0, 2)).toEqual([
      'site.list:birdcoder',
      'site.list:BirdCoder',
    ]);
    expect(result.site).toMatchObject({
      id: 'site-by-name',
      resolution: 'existingByName',
    });
    expect(result.deployment).toBeUndefined();
    expect(clients.mocks.deploymentCreate).not.toHaveBeenCalled();
  });

  it('creates a Site only after both exact lookups return no match', async () => {
    const clients = createMockClients({
      siteList: async () => ({
        items: [],
        pageInfo: { mode: 'offset', page: 1, pageSize: 50, hasMore: false },
      }),
      siteCreate: async () => ({ id: 'created-site' }),
    });

    const result = await publisher(clients).publish(publishRequest());

    expect(clients.calls.slice(0, 4)).toEqual([
      'site.list:birdcoder',
      'site.list:BirdCoder',
      'site.create',
      'drive.uploadArchive',
    ]);
    expect(result.site).toMatchObject({ id: 'created-site', resolution: 'created' });
    expect(clients.mocks.siteCreate).toHaveBeenCalledWith(
      expect.objectContaining({ name: 'BirdCoder', slug: 'birdcoder' }),
      { idempotencyKey: 'idempotency-1' },
      { signal: undefined, timeout: undefined },
    );
  });

  it('rejects an incomplete exact lookup instead of creating a duplicate Site', async () => {
    const clients = createMockClients({
      siteList: async () => ({
        items: [],
        pageInfo: { mode: 'offset', page: 1, pageSize: 50, hasMore: true },
      }),
    });

    await expect(publisher(clients).publish(publishRequest())).rejects.toMatchObject({
      code: 'SITE_RESOLUTION_AMBIGUOUS',
      stage: 'resolveSite',
    });
    expect(clients.mocks.siteCreate).not.toHaveBeenCalled();
    expect(clients.mocks.uploadArchive).not.toHaveBeenCalled();
  });

  it('fails closed when a resolved Site response omits its id', async () => {
    const clients = createMockClients({ siteRetrieve: async () => ({ name: 'BirdCoder' }) });

    await expect(
      publisher(clients).publish(
        publishRequest({ site: { kind: 'existing', siteId: 'site-1' } }),
      ),
    ).rejects.toMatchObject({
      code: 'SITE_RESPONSE_MISSING_ID',
      stage: 'resolveSite',
    });
    expect(clients.mocks.uploadArchive).not.toHaveBeenCalled();
  });

  it.each([
    {
      label: 'Artifact',
      options: { artifactCreate: async () => ({}) },
      code: 'ARTIFACT_RESPONSE_MISSING_ID',
      stage: 'registerArtifact',
    },
    {
      label: 'Release',
      options: { releaseCreate: async () => ({}) },
      code: 'RELEASE_RESPONSE_MISSING_ID',
      stage: 'createRelease',
    },
    {
      label: 'Deployment',
      options: { deploymentCreate: async () => ({}) },
      code: 'DEPLOYMENT_RESPONSE_MISSING_ID',
      stage: 'createDeployment',
    },
  ])('fails closed when the $label response omits its id', async ({ options, code, stage }) => {
    const clients = createMockClients(options);

    await expect(publisher(clients).publish(publishRequest())).rejects.toMatchObject({
      code,
      stage,
    });
  });
});

// Keep the mock boundary honest without reproducing dependency-owned DTOs.
void ({} as DriveUploaderUploadResult);
