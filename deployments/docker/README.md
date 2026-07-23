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

The worker does not serve HTTP and does not need the Drive App SDK or IAM/CORS/listener
configuration. It requires the Deploy database URL, Web and Drive Internal URLs and ingress-token
files, the HTTPS provider-event callback base, the protected per-Web-Node derivation-secret
directory, a unique `SDKWORK_NODE_INSTANCE_ID`, and the bounded runtime-assignment batch, polling,
lease, expiration, and renew-before settings from the selected topology profile. Kubernetes
injects the Pod UID as the process identity.

The callback base is an HTTPS origin or path prefix. The worker appends
`/nodes/{nodeUuid}/provider-events/drive-website-events`, registers the result through the generated
Drive Internal SDK, and renews it before expiry. The ingress must route the resulting path to the
matching Web Node without a fleet-wide load balancer or path rewrite. Deploy is the channel
registration/renewal controller; it does not receive or acknowledge ordinary Drive content events.
