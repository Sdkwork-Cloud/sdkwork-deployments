# Source Configuration

`sdkwork.deployment.config.json` is the source-controlled deployment profile index for SDKWork
Deploy. It selects one topology profile from `topology/`; the selected profile owns the lifecycle
environment, deployment profile, public and internal service origins, Drive mode, live content
provider mode, and Web Internal SDK endpoint. Profiles that run the runtime-assignment worker also
own its bounded batch, polling, and lease settings. Gateway runtime templates live under
`gateways/`.

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
