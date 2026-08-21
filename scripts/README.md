# SDKWork Shared Deployment Scripts

Multi-module, cross-platform Docker deployment toolkit for SDKWork applications.
Provides high-cohesion, low-coupling deployment scripts reusable across all
SDKWork modules (sdkwork-api-cloud-gateway, sdkwork-webserver, etc.).

## Architecture

```
scripts/
├── lib/
│   └── common.sh              # Shared mapping: port/domain/database/env
├── generate-env.sh            # Generate per-environment .env files
├── deploy.sh                  # Deploy environments (Bash/Linux/WSL)
├── deploy.ps1                 # Deploy environments (PowerShell/Windows)
├── install-nginx-sites.sh     # Install nginx reverse-proxy sites
├── provision-databases.sh     # Provision external PostgreSQL databases
├── verify-deployment.sh       # Verify deployment health
├── setup-wsl-deployment.sh    # End-to-end WSL deployment orchestrator
├── bind-windows-hosts.ps1     # Bind Windows hosts file entries
└── README.md                  # This file
```

## Design Principles

- **Single Source of Truth**: `lib/common.sh` defines the port/domain/database
  mapping. All scripts source this file, ensuring consistency.
  
- **Module-Agnostic**: Scripts auto-detect the current module or accept
  `--module` parameter. Any SDKWork module can reuse these scripts.
  
- **Cross-Platform**: Bash scripts target Linux/macOS/WSL; PowerShell scripts
  target Windows. Same configuration, same behavior.
  
- **Idempotent**: Safe to run multiple times. Existing resources are detected
  and preserved unless `--force` is specified.

## Canonical Image & Compose

- The deployed image is **`sdkwork-api-deployments-standalone-gateway`**
  (spec: `sdkwork-specs/NAMING_SPEC` `sdkwork-api-<app>-standalone-gateway`).
  `generate-env.sh` writes `GATEWAY_IMAGE` to this name; `setup-wsl-deployment.sh`
  builds and tags it `:local`; the Dockerfile, Kubernetes manifest, and
  `docker/README.md` all use the same name — no `:local` vs `:latest` drift.
- Compose templates ship at **`deployments/docker/docker-compose.yml`** and
  **`docker-compose.external.yml`**. `deploy.sh` / `deploy.ps1` resolve them
  automatically (override with `--compose-dir`). The container listens on **3900**
  (spec); host ports 3910–3913 map to it per environment.

## Port & Domain Matrix

| Environment | Host Port | Database | Domains |
| --- | --- | --- | --- |
| development | 3910 | `sdkwork_ai_dev` | `api-dev.{sdkwork,birdcoder,dtupay}.com` |
| test | 3911 | `sdkwork_ai_test` | `api-test.{sdkwork,birdcoder,dtupay}.com` |
| staging | 3912 | `sdkwork_ai_staging` | `api-staging.{sdkwork,birdcoder,dtupay}.com` |
| production | 3913 | `sdkwork_ai_prod` | `api.{sdkwork,birdcoder,dtupay}.com` |

## Quick Start (WSL Ubuntu)

```bash
cd /path/to/sdkwork-api-cloud-gateway

# Full automated deployment (generates envs, provisions DB, builds, deploys, nginx, verify)
../sdkwork-deployments/scripts/setup-wsl-deployment.sh

# Or step by step:
../sdkwork-deployments/scripts/generate-env.sh
../sdkwork-deployments/scripts/provision-databases.sh
pnpm build:container
../sdkwork-deployments/scripts/deploy.sh all
sudo ../sdkwork-deployments/scripts/install-nginx-sites.sh all
../sdkwork-deployments/scripts/verify-deployment.sh
```

On Windows (PowerShell, as Administrator):

```powershell
..\sdkwork-deployments\scripts\bind-windows-hosts.ps1
```

## Usage by Module

### sdkwork-api-cloud-gateway

```bash
cd sdkwork-api-cloud-gateway
../sdkwork-deployments/scripts/setup-wsl-deployment.sh \
  --module sdkwork-api-cloud-gateway \
  --container-port 3900
```

### sdkwork-webserver (example)

```bash
cd sdkwork-webserver
../sdkwork-deployments/scripts/setup-wsl-deployment.sh \
  --module sdkwork-webserver \
  --container-port 8080
```

## Script Reference

### generate-env.sh

Generates production-ready `.env` files from templates.

```bash
../sdkwork-deployments/scripts/generate-env.sh \
  --module sdkwork-api-cloud-gateway \
  --container-port 3900 \
  --output-dir ./docker/env \
  --environments development,test,staging,production \
  --secrets auto \
  --force
```

Options:
- `--secrets auto`: Generate random secrets (simple password for dev)
- `--secrets random`: Generate random secrets for all environments
- `--secrets skip`: Leave as `<CHANGE_ME>` placeholders
- `--postgres-host`, `--postgres-port`: External PostgreSQL connection
- `--redis-host`, `--redis-port`: External Redis connection

### deploy.sh

Deploys one or all environments via Docker Compose.

```bash
../sdkwork-deployments/scripts/deploy.sh development
../sdkwork-deployments/scripts/deploy.sh all --validate
../sdkwork-deployments/scripts/deploy.sh production --down
```

### install-nginx-sites.sh

Installs nginx reverse-proxy sites for port 80 access.

```bash
sudo ../sdkwork-deployments/scripts/install-nginx-sites.sh all
sudo ../sdkwork-deployments/scripts/install-nginx-sites.sh development production
```

### provision-databases.sh

Creates databases, schemas, and roles on external PostgreSQL.

```bash
export SDKWORK_DATABASE_ADMIN_PASSWORD=...
../sdkwork-deployments/scripts/provision-databases.sh
../sdkwork-deployments/scripts/provision-databases.sh development test
```

### verify-deployment.sh

Checks container health, endpoint availability, and domain isolation.

```bash
../sdkwork-deployments/scripts/verify-deployment.sh all
../sdkwork-deployments/scripts/verify-deployment.sh development production
```

### setup-wsl-deployment.sh

End-to-end orchestrator that calls all other scripts in sequence.

```bash
../sdkwork-deployments/scripts/setup-wsl-deployment.sh \
  --skip-build \          # Skip container image build
  --skip-provision \      # Skip database provisioning
  --skip-nginx \          # Skip nginx installation
  --skip-verify \         # Skip post-deploy verification
  --force                 # Overwrite existing files
```

## External Dependencies

These scripts deploy in **external dependency mode**: PostgreSQL and Redis
run outside the Docker Compose stack (Ubuntu host-native or remote managed).

Required on the host:
- PostgreSQL 14+ with `vector` extension (pgvector), listening on an
  interface reachable from Docker (not just 127.0.0.1)
- Redis 7+ bound to 0.0.0.0 (not just 127.0.0.1)
- nginx for port 80 reverse-proxy

## Security Notes

- `.env` files are generated with `0600` permissions (owner read/write only)
- Secrets are auto-generated with `openssl rand` (cryptographically random)
- Production environments require `sslmode=require` for PostgreSQL
- TLS termination for production at nginx (443) with HTTP->HTTPS redirect
