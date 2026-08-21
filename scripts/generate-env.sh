#!/bin/bash
# Generate per-environment .env files for SDKWork module Docker deployment.
#
# Creates production-ready .env files from templates with proper secrets,
# domain lists, and database identities for each lifecycle environment.
# Uses the shared sdkwork-deployments library for consistent mapping.
#
# Usage:
#   # From any SDKWork module workspace:
#   ../sdkwork-deployments/scripts/generate-env.sh
#   ../sdkwork-deployments/scripts/generate-env.sh --module sdkwork-api-cloud-gateway --container-port 3900
#   ../sdkwork-deployments/scripts/generate-env.sh --environments development production
#   ../sdkwork-deployments/scripts/generate-env.sh --output-dir ./docker/env
#
# Options:
#   --module <name>         Module name (auto-detected from current dir if omitted)
#   --container-port <port> Container internal port (auto-detected defaults: 3900)
#   --env-prefix <prefix>   Environment variable prefix (auto-derived)
#   --output-dir <path>     Output directory (default: ./docker/env)
#   --environments <list>   Comma-separated environments (default: development,test,staging,production)
#   --secrets <mode>        Secret generation: auto (default), random, skip
#   --postgres-host <host>  External PostgreSQL host (default: host.docker.internal)
#   --postgres-port <port>  External PostgreSQL port (default: 5432)
#   --redis-host <host>     External Redis host (default: host.docker.internal)
#   --redis-port <port>     External Redis port (default: 6380)
#   --image-name <name>     Container image name (default: canonical standalone-gateway)
#   --force                 Overwrite existing .env files
#   -h, --help              Show help
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
source "$SCRIPT_DIR/lib/common.sh"

# ============================================================================
# Argument parsing
# ============================================================================
MODULE_NAME=""
CONTAINER_PORT=""
ENV_PREFIX=""
IMAGE_NAME=""
OUTPUT_DIR="./docker/env"
ENVIRONMENTS="development,test,staging,production"
SECRETS_MODE="auto"
POSTGRES_HOST="host.docker.internal"
POSTGRES_PORT="5432"
REDIS_HOST="host.docker.internal"
REDIS_PORT="6380"
FORCE=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --module) MODULE_NAME="$2"; shift 2 ;;
    --container-port) CONTAINER_PORT="$2"; shift 2 ;;
    --env-prefix) ENV_PREFIX="$2"; shift 2 ;;
    --image-name) IMAGE_NAME="$2"; shift 2 ;;
    --output-dir) OUTPUT_DIR="$2"; shift 2 ;;
    --environments) ENVIRONMENTS="$2"; shift 2 ;;
    --secrets) SECRETS_MODE="$2"; shift 2 ;;
    --postgres-host) POSTGRES_HOST="$2"; shift 2 ;;
    --postgres-port) POSTGRES_PORT="$2"; shift 2 ;;
    --redis-host) REDIS_HOST="$2"; shift 2 ;;
    --redis-port) REDIS_PORT="$2"; shift 2 ;;
    --force) FORCE=true; shift ;;
    -h|--help)
      head -30 "$0" | grep -E '^#' | sed 's/^# \?//'
      exit 0
      ;;
    *) echo "unknown option: $1" >&2; exit 2 ;;
  esac
done

# ============================================================================
# Auto-detect module name from current directory
# ============================================================================
if [ -z "$MODULE_NAME" ]; then
  MODULE_NAME=$(basename "$(pwd)")
fi

# Auto-detect container port based on module
if [ -z "$CONTAINER_PORT" ]; then
  case "$MODULE_NAME" in
    sdkwork-api-cloud-gateway) CONTAINER_PORT=3900 ;;
    sdkwork-webserver) CONTAINER_PORT=8080 ;;
    sdkwork-cloudrouter) CONTAINER_PORT=3903 ;;
    *) CONTAINER_PORT=3900 ;;
  esac
fi

# Initialize module config
init_module_config "$MODULE_NAME" "$CONTAINER_PORT" "${ENV_PREFIX:-}"

# Canonical container image (single source of truth in common.sh)
IMAGE_NAME="${IMAGE_NAME:-$(module_image_name)}"

section "Generating environment files for $MODULE_NAME"
subsection "Container port: $CONTAINER_PORT"
subsection "Output: $OUTPUT_DIR"
subsection "Environments: $ENVIRONMENTS"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# ============================================================================
# Generate env file for each environment
# ============================================================================
IFS=',' read -ra ENV_LIST <<< "$ENVIRONMENTS"
for env_name in "${ENV_LIST[@]}"; do
  env_name=$(echo "$env_name" | xargs)  # trim whitespace
  validate_environment "$env_name" || exit 1

  port=$(env_port "$env_name")
  db=$(env_db "$env_name")
  redis_db1=$(env_redis_db1 "$env_name")
  redis_db2=$(env_redis_db2 "$env_name")
  redis_prefix=$(env_redis_prefix "$env_name")
  allowed_hosts=$(env_allowed_hosts "$env_name")
  cors_origins=$(env_cors_origins "$env_name")
  ssl_mode=$(env_ssl_mode "$env_name")
  acme_profile=$(env_acme_profile "$env_name")
  acme_directory=$(env_acme_directory "$env_name")
  primary_domain=$(env_primary_domain "$env_name")

  # Generate or skip secrets
  db_password=""
  case "$SECRETS_MODE" in
    random)
      db_password=$(generate_password)
      ;;
    auto)
      # For dev, use a simple default; for others, generate random
      if [ "$env_name" = "development" ]; then
        db_password="sdkworkdev123"
      else
        db_password=$(generate_password)
      fi
      ;;
    skip)
      db_password="<CHANGE_ME>"
      ;;
  esac

  env_file="$OUTPUT_DIR/$env_name.env"

  if [ -f "$env_file" ] && [ "$FORCE" = false ]; then
    warn "$env_file already exists (use --skip or remove manually)"
    continue
  fi

  # Spec (ENVIRONMENT_SPEC): production SHOULD default AUTO_MIGRATE=false; lower
  # environments auto-migrate on start so the schema stays current without a separate job.
  db_auto_migrate="false"
  if [ "$env_name" != "production" ]; then
    db_auto_migrate="true"
  fi

  cat > "$env_file" <<ENVEOF
# $MODULE_NAME - $env_name environment
# Generated by sdkwork-deployments/scripts/generate-env.sh
# Deployment: host port $port -> container $CONTAINER_PORT

${MODULE_ENV_PREFIX}_IMAGE=${IMAGE_NAME}:local
${MODULE_ENV_PREFIX}_ENVIRONMENT=$env_name
${MODULE_ENV_PREFIX}_PROFILE_ID=standalone.$env_name
# External-dependency (compose) deployments run in standalone profile, overriding the
# Dockerfile default of "cloud". Must match PROFILE_ID above.
SDKWORK_DEPLOY_DEPLOYMENT_PROFILE=standalone
${MODULE_ENV_PREFIX}_HOST_PORT=$port

# Provisioning flags
${MODULE_ENV_PREFIX}_PROVISION_PAYMENT_CREDENTIAL_KEY=true
${MODULE_ENV_PREFIX}_PROVISION_IAM_SIGNING_MASTER_SECRET=true
${MODULE_ENV_PREFIX}_PROVISION_KNOWLEDGEBASE_RPC_SECRETS=true
${MODULE_ENV_PREFIX}_IAM_ALLOWED_AUDIENCES=${MODULE_NAME}

# Lifecycle
${MODULE_ENV_PREFIX}_MIGRATE_ON_START=true
SDKWORK_DATABASE_AUTO_MIGRATE=$db_auto_migrate

# External PostgreSQL (shared instance, per-environment database)
${MODULE_ENV_PREFIX}_POSTGRES_HOST=$POSTGRES_HOST
${MODULE_ENV_PREFIX}_POSTGRES_PORT=$POSTGRES_PORT
${MODULE_ENV_PREFIX}_POSTGRES_DB=$db
${MODULE_ENV_PREFIX}_POSTGRES_SCHEMA=$db
${MODULE_ENV_PREFIX}_POSTGRES_USER=$db
${MODULE_ENV_PREFIX}_POSTGRES_PASSWORD=$db_password
${MODULE_ENV_PREFIX}_POSTGRES_SSL_MODE=$ssl_mode

# Domain allowlist (all brands)
${MODULE_ENV_PREFIX}_ALLOWED_HOSTS=$allowed_hosts

# CORS allowlist
${MODULE_ENV_PREFIX}_CORS_ALLOWED_ORIGINS=$cors_origins

# IM principal directory
${MODULE_ENV_PREFIX}_IM_ENVIRONMENT=$env_name
${MODULE_ENV_PREFIX}_IM_PRINCIPAL_DIRECTORY=postgres
${MODULE_ENV_PREFIX}_IM_ALLOW_ALL_PRINCIPALS=false
${MODULE_ENV_PREFIX}_IM_ID_NODE_ID=2
${MODULE_ENV_PREFIX}_KNOWLEDGEBASE_TENANT_ID=100001

# ACME / TLS
${MODULE_ENV_PREFIX}_WEBSERVER_ACME_PROFILE=$acme_profile
${MODULE_ENV_PREFIX}_WEBSERVER_ACME_DIRECTORY_URL=$acme_directory
${MODULE_ENV_PREFIX}_WEBSERVER_ACME_CONTACT_EMAIL=admin@${DEPLOY_BRAND_DOMAINS[0]}
${MODULE_ENV_PREFIX}_WEBSERVER_NODE_UUID=${MODULE_NAME}-${env_name}-0

# External Redis (shared instance, per-environment logical DBs)
${MODULE_ENV_PREFIX}_CLOUDROUTER_REDIS_HOST=$REDIS_HOST
${MODULE_ENV_PREFIX}_CLOUDROUTER_REDIS_PORT=$REDIS_PORT
${MODULE_ENV_PREFIX}_CLOUDROUTER_REDIS_KEY_PREFIX=$redis_prefix
${MODULE_ENV_PREFIX}_RTC_STATE_REDIS_URL=redis://${REDIS_HOST}:${REDIS_PORT}/${redis_db1}
${MODULE_ENV_PREFIX}_WEB_REDIS_URL=redis://${REDIS_HOST}:${REDIS_PORT}/${redis_db2}

# Knowledgebase
${MODULE_ENV_PREFIX}_KNOWLEDGEBASE_ENVIRONMENT=$env_name

# CloudRouter secrets (local defaults; override for production)
${MODULE_ENV_PREFIX}_CLOUDROUTER_API_KEY_PEPPER=$(generate_secret)
${MODULE_ENV_PREFIX}_CLOUDROUTER_TRUSTED_SUBJECT_SECRET=$(generate_secret)
${MODULE_ENV_PREFIX}_CLOUDROUTER_APP_SESSION_SECRET=$(generate_secret)
${MODULE_ENV_PREFIX}_CLOUDROUTER_PAYMENT_WEBHOOK_SECRET=$(generate_secret)
${MODULE_ENV_PREFIX}_CLOUDROUTER_INTERNAL_GATEWAY_SIGNING_SECRET=$(generate_secret)

# Logging
RUST_LOG=info
ENVEOF

  # Set restrictive permissions on env file (contains secrets)
  chmod 0600 "$env_file"
  success "Generated $env_file (port $port, db $db)"
done

section "Environment files generated successfully"
echo "  Location: $OUTPUT_DIR"
echo ""
echo "Next steps:"
echo "  1. Review and adjust secrets in the .env files"
echo "  2. Provision databases: ../sdkwork-deployments/scripts/provision-databases.sh"
echo "  3. Deploy: ../sdkwork-deployments/scripts/deploy.sh all"
