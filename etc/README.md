# Source Configuration

`sdkwork.deployment.config.json` is the source-controlled deployment profile index for SDKWork
Deploy. It selects one topology profile from `topology/`; the selected profile owns the lifecycle
environment, deployment profile, public and internal service origins, Drive mode, live content
provider mode, and Web Internal SDK endpoint. Profiles that run the runtime-assignment worker also
own its bounded batch, polling, lease, Drive WebsiteRoot event-callback, and event-channel renewal
settings. Gateway runtime templates live under `gateways/`.

## Supported Profiles

| Profile | Purpose |
| --- | --- |
| `standalone.development` | Default local gateway with in-memory Drive/content providers and a local Web Server. |
| `cloud.development` | Remote development provider services; no loopback cloud fallbacks. |
| `standalone.production` | Production template for a single-host deployment. |
| `cloud.production` | Production template for the hosted SDKWork deployment. |

The topology contract is `specs/topology.spec.json`. Source profiles are materialized by
`@sdkwork/app-topology`; installed runtime configuration follows `RUNTIME_DIRECTORY_SPEC.md` and is
not written back into this directory.

## Local Overrides And Secrets

Tracked profiles contain no credential values. Cloud development reads Drive Internal,
Knowledgebase Internal, and Web Internal ingress tokens from the ignored `.runtime/secrets/`
directory. Production reads the same three credentials from the mounted
`/run/secrets/sdkwork/` directory. Rotate each file atomically; provider adapters read the relevant
file for every validation or publication attempt, so a process restart is not required.

The runtime-assignment worker additionally reads one Drive WebsiteRoot derivation secret for each
assigned Web Node from `SDKWORK_DEPLOY_WEBSITE_PROVIDER_EVENT_SECRET_DIRECTORY`. The filename is
`drive-website-node-<sha256(nodeUuid UTF-8)>.derivation-secret`; the digest is lowercase hexadecimal
and the file contains 32 to 1024 bytes. The same bytes are mounted only into that Web Node and are
referenced by its provider-event config. The worker caches only renewal time and the secret's
SHA-256 fingerprint. Atomic secret rotation changes the fingerprint and forces immediate channel
replacement; plaintext channel tokens are neither persisted nor logged.

For every Drive WebsiteRoot referenced by an active runtime set, the worker registers or renews the
owner channel through the generated Drive Internal SDK before publishing the assignment to Web.
The callback is exactly
`<SDKWORK_DEPLOY_WEBSITE_PROVIDER_EVENT_CALLBACK_BASE_URL>/nodes/{nodeUuid}/provider-events/drive-website-events`.
The HTTPS ingress must preserve this path and route it to the provider-event Service for that exact
Web Node. Deploy owns registration and renewal only; Drive delivers ordinary content events
directly to Web Server, and Deploy neither relays nor acknowledges them.

Files under `.runtime/`, `etc/**/*.local.*`, and `etc/secrets/` remain untracked. Production secret
material comes from the deployment platform or a protected operating-system secret file.
`SDKWORK_NODE_INSTANCE_ID` is runtime instance identity and is never hardcoded in a source profile;
the process supervisor or orchestrator must inject a unique value for each production process.

## Verification

```powershell
node ../sdkwork-specs/tools/check-source-config-standard.mjs --root .
pnpm topology:validate
pnpm check
```
