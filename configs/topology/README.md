# Topology Profiles

Safe checked-in topology profile templates for SDKWork Deploy.

Load a profile through `specs/topology.spec.json` and `pnpm topology:validate`.

Production profiles set Drive integration env:

- `SDKWORK_DEPLOY_USE_MEMORY_DRIVE=0`
- `SDKWORK_DRIVE_FACADE_URL` — Drive app-api base URL (for example `https://api.sdkwork.com`)
