# Application Publisher Flow

Use the public composed facade rather than generated transport internals:

```ts
import { createDeployApplicationPublisher } from "@sdkwork/deployments-app-sdk/application-publisher";

const publisher = createDeployApplicationPublisher({
  deployClient,
  driveClient,
});

const result = await publisher.publish({
  site: {
    kind: "resolveOrCreate",
    name: applicationName,
    slug: applicationSlug,
    siteType,
  },
  artifact: {
    file: packageFile,
    packageType,
    fileName: archiveName,
    contentType: "application/zip",
    checksumSha256,
    source: applicationKey,
  },
  release: {
    versionTag,
  },
  deployment: {
    deployType,
    environment,
    versionTag,
    commitHash,
    sourceRef,
  },
  onProgress(progress) {
    reportProgress(progress);
  },
});
```

Obtain `deployClient` and `driveClient` from the target application's approved bootstrap or injected SDK provider. `packageFile` must implement the Drive uploader blob contract and represent the final non-empty archive. Compute `checksumSha256` from those exact bytes before calling `publish`.

The result contains stable evidence:

```ts
const {
  site,
  upload,
  artifact,
  release,
  deployment,
} = result;
```

When a deployment was requested, retrieve it through the generated facade:

```ts
const current = await deployClient.deployment.sites.deployments.retrieve(
  site.id,
  deployment.id,
);
```

Do not map numeric deployment statuses to names unless the active API contract defines that mapping. Preserve the raw status and timestamps in completion evidence.

Rollback is a separate confirmed action:

```ts
const rollback = await deployClient.deployment.sites.deployments.rollback(
  site.id,
  deployment.id,
);
```

Never call rollback speculatively or use it as a deployment verification step.
