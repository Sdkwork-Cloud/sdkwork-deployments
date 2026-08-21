#!/bin/bash
# End-to-end WSL Ubuntu multi-environment Docker deployment.
#
# This is the unified entry point that orchestrates the full deployment:
#   1. Generate environment files with proper secrets
#   2. Provision external PostgreSQL databases
#   3. Build container image
#   4. Deploy all environments (external PostgreSQL/Redis mode)
#   5. Install nginx reverse-proxy sites
#   6. Verify health across all environments and domains
#
# Prerequisites: Docker, PostgreSQL (5432), Redis (6380), nginx on WSL Ubuntu.
# Run from the target module workspace (e.g., sdkwork-api-cloud-gateway).
#
# Usage:
#   ../sdkwork-deployments/scripts/setup-wsl-deployment.sh
#   ../sdkwork-deployments/scripts/setup-wsl-deployment.sh --skip-build
#   ../sdkwork-deployments/scripts/setup-wsl-deployment.sh --skip-provision
#   ../sdkwork-deployments/scripts/setup-wsl-deployment.sh --environments development production
#
# Options:
#   --module <name>        Module name (auto-detected from cwd)
#   --container-port <p>   Container internal port (auto-detected)
#   --environments <list>  Comma-separated environments (default: all)
#   --skip-build           Skip container image build
#   --skip-provision       Skip database provisioning
#   --skip-nginx           Skip nginx site installation
#   --skip-verify          Skip post-deployment verification
#   --force                Overwrite existing configurations
#   -h, --help             Show help
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
source "$SCRIPT_DIR/lib/common.sh"

# ============================================================================
# Argument parsing
# ============================================================================
MODULE_NAME=""
CONTAINER_PORT=""
ENVIRONMENTS="development,test,staging,production"
SKIP_BUILD=false
SKIP_PROVISION=false
SKIP_NGINX=false
SKIP_VERIFY=false
FORCE=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --module) MODULE_NAME="$2"; shift 2 ;;
    --container-port) CONTAINER_PORT="$2"; shift 2 ;;
    --environments) ENVIRONMENTS="$2"; shift 2 ;;
    --skip-build) SKIP_BUILD=true; shift ;;
    --skip-provision) SKIP_PROVISION=true; shift ;;
    --skip-nginx) SKIP_NGINX=true; shift ;;
    --skip-verify) SKIP_VERIFY=true; shift ;;
    --force) FORCE=true; shift ;;
    -h|--help)
      head -25 "$0" | grep -E '^#' | sed 's/^# \?//'
      exit 0
      ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

# Auto-detect module name
if [ -z "$MODULE_NAME" ]; then
  MODULE_NAME=$(basename "$(pwd)")
fi

# Auto-detect container port
if [ -z "$CONTAINER_PORT" ]; then
  case "$MODULE_NAME" in
    sdkwork-api-cloud-gateway) CONTAINER_PORT=3900 ;;
    sdkwork-webserver) CONTAINER_PORT=8080 ;;
    sdkwork-cloudrouter) CONTAINER_PORT=3903 ;;
    *) CONTAINER_PORT=3900 ;;
  esac
fi

init_module_config "$MODULE_NAME" "$CONTAINER_PORT"
IMAGE_NAME="$(module_image_name)"

# ============================================================================
# Pre-flight checks
# ============================================================================
section "WSL Deployment Setup: $MODULE_NAME"
echo "  Module: $MODULE_NAME"
echo "  Container port: $CONTAINER_PORT"
echo "  Environments: $ENVIRONMENTS"
echo "  Working directory: $(pwd)"
echo ""

require_command docker || fail "Docker is not installed"

# Check Docker is running
if ! docker info >/dev/null 2>&1; then
  fail "Docker daemon is not running"
fi

# ============================================================================
# Step 1: Generate environment files
# ============================================================================
section "Step 1: Generate environment files"

local_force=""
if [ "$FORCE" = true ]; then
  local_force="--force"
fi

"$SCRIPT_DIR/generate-env.sh" \
  --module "$MODULE_NAME" \
  --container-port "$CONTAINER_PORT" \
  --image-name "$IMAGE_NAME" \
  --output-dir ./docker/env \
  --environments "$ENVIRONMENTS" \
  --secrets auto \
  $local_force

success "Environment files generated"

# ============================================================================
# Step 2: Provision databases
# ============================================================================
if [ "$SKIP_PROVISION" = true ]; then
  section "Step 2: Database provisioning (SKIPPED)"
else
  section "Step 2: Provision external databases"

  if ! command -v psql >/dev/null 2>&1; then
    warn "psql not found; install with: sudo apt install postgresql-client"
    warn "Skipping database provisioning"
  else
    if [ -z "${SDKWORK_DATABASE_ADMIN_PASSWORD:-}" ] && [ -z "${PGPASSWORD:-}" ]; then
      warn "No admin password set (SDKWORK_DATABASE_ADMIN_PASSWORD or PGPASSWORD)"
      warn "Skipping database provisioning"
    else
      "$SCRIPT_DIR/provision-databases.sh" \
        --env-dir ./docker/env \
        --admin-host "${SDKWORK_DATABASE_ADMIN_HOST:-127.0.0.1}" \
        --admin-port "${SDKWORK_DATABASE_ADMIN_PORT:-5432}" \
        --admin-user "${SDKWORK_DATABASE_ADMIN_USERNAME:-postgres}" \
        --admin-pass "${SDKWORK_DATABASE_ADMIN_PASSWORD:-${PGPASSWORD:-}}" \
        $(IFS=','; echo $ENVIRONMENTS)

      success "Databases provisioned"
    fi
  fi
fi

# ============================================================================
# Step 3: Build container image
# ============================================================================
if [ "$SKIP_BUILD" = true ]; then
  section "Step 3: Container build (SKIPPED)"
else
  section "Step 3: Build container image"

  if [ -f "package.json" ] && grep -q '"build:container"' package.json 2>/dev/null; then
    pnpm build:container --force
    success "Container image built"
  else
    # Build from the workspace root so Cargo sibling dependencies are available.
    # Context = workspace root; Dockerfile is at sdkwork-deployments/deployments/docker/Dockerfile.
    local dockerfile="$SCRIPT_DIR/../deployments/docker/Dockerfile"
    local ctx
    ctx="$(cd "$SCRIPT_DIR/.." && pwd)/.."
    if [ -f "$dockerfile" ]; then
      docker build -f "$dockerfile" -t "${IMAGE_NAME}:local" "$ctx"
      success "Container image built: ${IMAGE_NAME}:local"
    else
      warn "Dockerfile not found at $dockerfile; skipping build"
      warn "Ensure ${IMAGE_NAME}:local image exists before deploying"
    fi
  fi
fi

# ============================================================================
# Step 4: Deploy environments
# ============================================================================
section "Step 4: Deploy environments"

"$SCRIPT_DIR/deploy.sh" \
  all \
  --env-dir ./docker/env \
  --compose-dir . \
  --validate

success "All environments deployed"

# ============================================================================
# Step 5: Install nginx sites
# ============================================================================
if [ "$SKIP_NGINX" = true ]; then
  section "Step 5: Nginx sites (SKIPPED)"
else
  section "Step 5: Install nginx sites"

  if ! command -v nginx >/dev/null 2>&1; then
    warn "nginx not found; install with: sudo apt install nginx"
    warn "Skipping nginx installation"
  else
    if [ "$(id -u)" -eq 0 ]; then
      "$SCRIPT_DIR/install-nginx-sites.sh" \
        --module "$MODULE_NAME" \
        $(IFS=','; echo $ENVIRONMENTS)
    elif sudo -n true 2>/dev/null; then
      sudo "$SCRIPT_DIR/install-nginx-sites.sh" \
        --module "$MODULE_NAME" \
        $(IFS=','; echo $ENVIRONMENTS)
    else
      warn "sudo required for nginx installation"
      warn "Run manually: sudo $SCRIPT_DIR/install-nginx-sites.sh --module $MODULE_NAME $(IFS=','; echo $ENVIRONMENTS)"
    fi
    success "Nginx sites installed"
  fi
fi

# ============================================================================
# Step 6: Verify deployment
# ============================================================================
if [ "$SKIP_VERIFY" = true ]; then
  section "Step 6: Verification (SKIPPED)"
else
  section "Step 6: Verify deployment"

  # Wait briefly for services to stabilize
  sleep 5

  "$SCRIPT_DIR/verify-deployment.sh" \
    --module "$MODULE_NAME" \
    --env-dir ./docker/env \
    all

  success "Verification complete"
fi

# ============================================================================
# Summary
# ============================================================================
section "Deployment complete!"
echo ""
echo "Module: $MODULE_NAME"
echo ""
echo "Environment endpoints:"
IFS=',' read -ra ENV_LIST <<< "$ENVIRONMENTS"
for env_name in "${ENV_LIST[@]}"; do
  env_name=$(echo "$env_name" | xargs)
  port=$(env_port "$env_name")
  domain=$(env_primary_domain "$env_name")
  db=$(env_db "$env_name")
  echo "  $env_name:"
  echo "    Direct:  http://127.0.0.1:${port}/healthz"
  echo "    Domain:  http://${domain}/healthz"
  echo "    Database: $db"
done
echo ""
echo "Next steps:"
echo "  - Configure Windows hosts file for local domain resolution"
echo "  - Review secrets in docker/env/*.env files"
echo "  - For production: set up TLS certificates at /etc/ssl/sdkwork/api-gateway/"
echo ""
echo "WSL_DEPLOYMENT_DONE"
