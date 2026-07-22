# SDKWork Deploy Docker Image

The image contains two process entrypoints:

- `sdkwork-api-deployments-standalone-gateway` serves the public application APIs.
- `sdkwork-deploy-runtime-assignment-worker` claims durable runtime assignments and publishes them
  to Web Server through the generated Web Internal SDK.

Run one process per container. The default command starts the gateway; override the command with
`sdkwork-deploy-runtime-assignment-worker` for worker replicas. Both process types use the same
immutable image digest and database schema.

Build from the SDKWork workspace root so Cargo sibling dependencies are available:

```bash
docker build -f sdkwork-deployments/deployments/docker/Dockerfile -t sdkwork-api-deployments-standalone-gateway:latest .
```

Run with SQLite (development only):

```bash
docker run --rm -p 3900:8080 \
  -e SDKWORK_DEPLOY_DATABASE_ENGINE=sqlite \
  -e SDKWORK_DEPLOY_DATABASE_URL=sqlite:///app/data/deploy.db \
  -e SDKWORK_DEPLOY_DATABASE_AUTO_MIGRATE=true \
  sdkwork-api-deployments-standalone-gateway:latest
```

Production deployments must use PostgreSQL and IAM database credentials via secrets. They must
also set `SDKWORK_DEPLOY_USE_MEMORY_DRIVE=false`, `SDKWORK_DEPLOY_USE_MEMORY_CONTENT_PROVIDER=false`,
`SDKWORK_DRIVE_FACADE_URL`, the Drive/Knowledgebase Internal API URLs, and
`SDKWORK_DEPLOY_WEB_INTERNAL_API_URL`. Mount each Internal API ingress token as a read-only file at
the path declared by its corresponding `*_INGRESS_TOKEN_FILE` key. Production Snowflake ids use
the shared database lease allocator; static node ids are rejected.

The worker does not serve HTTP and does not need Drive, IAM, CORS, or listener configuration. It
requires the Deploy database URL, Web Internal URL and ingress-token file, a unique
`SDKWORK_NODE_INSTANCE_ID`, and the bounded runtime-assignment batch, polling, and lease settings
from the selected topology profile. Kubernetes injects the Pod UID as the process identity.
