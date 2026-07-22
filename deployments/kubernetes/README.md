# SDKWork Deploy Kubernetes Manifests

Apply order:

1. Create secrets `sdkwork-deploy-database`, `sdkwork-deploy-iam-database`,
   `sdkwork-deploy-web-internal`, `sdkwork-deploy-drive-internal`, and
   `sdkwork-deploy-knowledgebase-internal`. Each Internal API secret must contain the
   `ingress-token` key.
2. `kubectl apply -f migration-job.yaml` and wait for completion.
3. `kubectl apply -f deployment.yaml`.
4. `kubectl apply -f runtime-assignment-worker.yaml`.
5. `kubectl apply -f service.yaml`.

Gateway health endpoints:

- `GET /healthz` - gateway process liveness.
- `GET /readyz` - gateway database readiness.

The runtime-assignment worker has no HTTP surface. Kubernetes treats a running process as ready;
the worker exits on invalid production configuration and uses an expiring database lease so an
abandoned assignment can be reclaimed by another replica.

The gateway projects the three Internal API credentials into one read-only volume. Drive and
Knowledgebase tokens are used only for provider eligibility checks; the Web token is used for
immutable runtime-assignment publication. Kubernetes secret projection supports rotation without
embedding credentials in environment variables or images.

Production pods obtain collision-free Snowflake node ids through the shared database lease
registry. Do not set a static `SDKWORK_DEPLOY_SNOWFLAKE_NODE_ID` in production. The Pod UID is
injected as `SDKWORK_NODE_INSTANCE_ID` to give each worker and gateway process a unique identity.

API surfaces:

- App: `/app/v3/api/*`
- Backend: `/backend/v3/api/*`
