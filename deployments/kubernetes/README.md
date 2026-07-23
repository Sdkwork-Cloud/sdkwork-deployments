# SDKWork Deploy Kubernetes Manifests

Apply order:

1. Create secrets `sdkwork-deploy-database`, `sdkwork-deploy-iam-database`,
   `sdkwork-deploy-web-internal`, `sdkwork-deploy-drive-internal`, and
   `sdkwork-deploy-knowledgebase-internal`. Each Internal API secret must contain the
   `ingress-token` key. Install/configure Secrets Store CSI and create the
   `sdkwork-deploy-website-provider-events` `SecretProviderClass` before starting the worker.
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
immutable runtime-assignment publication. The worker projects only the Web and Drive Internal
tokens it needs. Its separate CSI volume exposes per-Web-Node Drive derivation secrets using the
hashed filename contract documented in `etc/README.md`; gateway pods never receive those node
secrets. Secret projection supports rotation without embedding credentials in environment
variables or images. A rotated node secret changes the worker cache fingerprint and causes channel
replacement on the next bounded worker cycle.

The referenced `SecretProviderClass` is platform-owned and intentionally absent from this
application repository because its provider, vault object identifiers, and workload identity are
environment-specific secrets. It must project each assigned Node secret read-only as
`drive-website-node-<lowercase-sha256(nodeUuid UTF-8)>.derivation-secret`, with file mode no broader
than `0440`; startup fails closed when a required file is absent or invalid. The same secret bytes
must be mounted only into the matching Web Node's provider-event configuration. Gateway Pods must
never mount this class.

The worker registers each Drive callback as
`https://web-provider-events.sdkwork.com/nodes/{nodeUuid}/provider-events/drive-website-events`.
The internal HTTPS/mTLS ingress must preserve the complete path and send it to the Node-specific
Web provider-event Service; it must not rewrite to the unqualified Knowledgebase route or
load-balance across a tenant fleet. Deploy renews the Drive channel but is not in the ordinary
content-event data path.

Production pods obtain collision-free Snowflake node ids through the shared database lease
registry. Do not set a static `SDKWORK_DEPLOY_SNOWFLAKE_NODE_ID` in production. The Pod UID is
injected as `SDKWORK_NODE_INSTANCE_ID` to give each worker and gateway process a unique identity.

API surfaces:

- App: `/app/v3/api/*`
- Backend: `/backend/v3/api/*`
