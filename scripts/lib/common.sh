#!/bin/bash
# Shared deployment library for SDKWork multi-module Docker deployments.
# Single source of truth for environment/port/domain mapping.
# Sourced by all deployment scripts to ensure consistency and avoid duplication.
#
# This library is module-agnostic: pass MODULE_NAME to customize for any SDKWork
# module (sdkwork-api-cloud-gateway, sdkwork-webserver, etc.).
#
# Usage:
#   source scripts/lib/common.sh
#   init_module_config "sdkwork-api-cloud-gateway" "3900"
#
# Or import from any SDKWork module workspace:
#   source ../sdkwork-deployments/scripts/lib/common.sh
#   init_module_config "sdkwork-webserver" "8080"

set -eu

# ============================================================================
# Brand Domains (sdkwork-specs multi-brand support)
# ============================================================================
DEPLOY_BRAND_DOMAINS=("sdkwork.com" "birdcoder.com" "dtupay.com")

# ============================================================================
# Canonical container image (sdkwork-specs NAMING_SPEC: sdkwork-api-<app>-standalone-gateway)
# Single source of truth for the image name used by generate-env, deploy, and build.
# The Dockerfile, Kubernetes manifest, and docker/README all use this exact name.
# ============================================================================
DEPLOY_GATEWAY_IMAGE="sdkwork-api-deployments-standalone-gateway"

# Resolve the container image name (override via --image-name when needed).
module_image_name() {
  echo "${DEPLOY_GATEWAY_IMAGE}"
}

# ============================================================================
# Environment Port Matrix (deployments/README.md)
# 391x band reserved for gateway-style module containers
# ============================================================================
ENV_DEV_PORT=3910
ENV_TEST_PORT=3911
ENV_STAGING_PORT=3912
ENV_PROD_PORT=3913

# ============================================================================
# Database Identities (sdkwork-specs ENVIRONMENT_SPEC §7.1)
# ============================================================================
ENV_DEV_DB="sdkwork_ai_dev"
ENV_TEST_DB="sdkwork_ai_test"
ENV_STAGING_DB="sdkwork_ai_staging"
ENV_PROD_DB="sdkwork_ai_prod"

# ============================================================================
# Redis Logical DB Isolation (shared instance, per-environment DB indexes)
# ============================================================================
ENV_DEV_REDIS_DB1=1
ENV_DEV_REDIS_DB2=2
ENV_TEST_REDIS_DB1=3
ENV_TEST_REDIS_DB2=4
ENV_STAGING_REDIS_DB1=5
ENV_STAGING_REDIS_DB2=6
ENV_PROD_REDIS_DB1=7
ENV_PROD_REDIS_DB2=8

# ============================================================================
# Nginx Defaults
# ============================================================================
NGINX_PROXY_TIMEOUT=300
NGINX_CLIENT_MAX_BODY=1100m
NGINX_SITES_DIR="/etc/nginx/sites-enabled"

# ============================================================================
# Module Configuration (set by init_module_config)
# ============================================================================
MODULE_NAME=""
MODULE_CONTAINER_PORT=""
MODULE_ENV_PREFIX="GATEWAY"
# Module-specific single-source-of-truth overrides (populated by init_module_config):
MODULE_DOMAIN_PREFIX="api"     # subdomain stem: api / server / router
MODULE_HOST_PORT_DEV=3910
MODULE_HOST_PORT_TEST=3911
MODULE_HOST_PORT_STAGING=3912
MODULE_HOST_PORT_PROD=3913
MODULE_CERT_DIR="/etc/ssl/sdkwork/api-gateway"
MODULE_REDIS_SCHEME="GATEWAY"  # env-var family: GATEWAY (CLOUDROUTER_REDIS_*) | WEBSERVER (WEBSERVER_REDIS_*)

# ============================================================================
# Module Initialization
# ============================================================================

# Initialize module-specific configuration
# Args:
#   $1 - module name (e.g., "sdkwork-api-cloud-gateway")
#   $2 - container internal port (e.g., "3900")
#   $3 - optional env var prefix (defaults to uppercase module name)
init_module_config() {
  MODULE_NAME="${1:-}"
  MODULE_CONTAINER_PORT="${2:-}"
  if [ -n "${3:-}" ]; then
    MODULE_ENV_PREFIX="$3"
  else
    # Auto-derive prefix: sdkwork-api-cloud-gateway -> GATEWAY, sdkwork-cloudrouter -> CLOUDROUTER
    MODULE_ENV_PREFIX=$(echo "$MODULE_NAME" | sed 's/^sdkwork-//; s/^api-//; s/cloud-gateway$//; s/-webserver$//' | tr '[:lower:]' '[:upper:]')
    if [ -z "$MODULE_ENV_PREFIX" ]; then
      MODULE_ENV_PREFIX="GATEWAY"
    fi
  fi

  # --------------------------------------------------------------------------
  # Module-specific single-source-of-truth overrides.
  # Every module derives its domain prefix, host-port band, TLS cert directory,
  # and env-var family from this single block so the encapsulated scripts stay
  # module-agnostic (high cohesion / low coupling).
  # --------------------------------------------------------------------------
  case "$MODULE_NAME" in
    sdkwork-webserver)
      MODULE_DOMAIN_PREFIX="server"
      MODULE_HOST_PORT_DEV=13800
      MODULE_HOST_PORT_TEST=18888
      MODULE_HOST_PORT_STAGING=13812
      MODULE_HOST_PORT_PROD=18080
      MODULE_CERT_DIR="/etc/ssl/sdkwork/webserver"
      MODULE_REDIS_SCHEME="WEBSERVER"
      if [ "$MODULE_ENV_PREFIX" = "GATEWAY" ]; then MODULE_ENV_PREFIX="WEBSERVER"; fi
      ;;
    sdkwork-cloudrouter)
      MODULE_DOMAIN_PREFIX="router"
      MODULE_HOST_PORT_DEV=3901
      MODULE_HOST_PORT_TEST=3902
      MODULE_HOST_PORT_STAGING=3903
      MODULE_HOST_PORT_PROD=3904
      MODULE_CERT_DIR="/etc/ssl/sdkwork/cloudrouter"
      MODULE_REDIS_SCHEME="GATEWAY"
      if [ "$MODULE_ENV_PREFIX" = "GATEWAY" ]; then MODULE_ENV_PREFIX="CLOUDROUTER"; fi
      ;;
    sdkwork-api-cloud-gateway|*)
      MODULE_DOMAIN_PREFIX="api"
      MODULE_HOST_PORT_DEV=3910
      MODULE_HOST_PORT_TEST=3911
      MODULE_HOST_PORT_STAGING=3912
      MODULE_HOST_PORT_PROD=3913
      MODULE_CERT_DIR="/etc/ssl/sdkwork/api-gateway"
      MODULE_REDIS_SCHEME="GATEWAY"
      ;;
  esac
  return 0
}

# ============================================================================
# Environment Mapping Functions
# ============================================================================

# Get environment suffix: development -> dev, test -> test, staging -> staging, production -> ""
env_suffix() {
  case "$1" in
    development) echo "dev" ;;
    test) echo "test" ;;
    staging) echo "staging" ;;
    production) echo "" ;;
    *) return 1 ;;
  esac
}

# Get API/Web subdomain: development -> <prefix>-dev, production -> <prefix>
# The prefix is module-specific (api / server / router) from init_module_config.
env_subdomain() {
  local suffix
  suffix=$(env_suffix "$1")
  if [ -z "$suffix" ]; then
    echo "$MODULE_DOMAIN_PREFIX"
  else
    echo "${MODULE_DOMAIN_PREFIX}-${suffix}"
  fi
}

# Get host port for an environment (module-specific band)
env_port() {
  case "$1" in
    development) echo "$MODULE_HOST_PORT_DEV" ;;
    test) echo "$MODULE_HOST_PORT_TEST" ;;
    staging) echo "$MODULE_HOST_PORT_STAGING" ;;
    production) echo "$MODULE_HOST_PORT_PROD" ;;
    *) return 1 ;;
  esac
}

# Get the TLS certificate directory for the active module.
env_cert_dir() {
  echo "$MODULE_CERT_DIR"
}

# Get the host-port environment-variable name used by a module's compose file.
# webserver uses SDKWORK_WEBSERVER_<ENV>_HOST_PORT; gateway uses <PREFIX>_HOST_PORT.
env_host_port_var() {
  case "$MODULE_REDIS_SCHEME" in
    WEBSERVER)
      case "$1" in
        development) echo "SDKWORK_WEBSERVER_DEV_HOST_PORT" ;;
        test) echo "SDKWORK_WEBSERVER_TEST_HOST_PORT" ;;
        staging) echo "SDKWORK_WEBSERVER_STAGING_HOST_PORT" ;;
        production) echo "SDKWORK_WEBSERVER_PROD_HOST_PORT" ;;
      esac
      ;;
    *)
      echo "${MODULE_ENV_PREFIX}_HOST_PORT"
      ;;
  esac
}

# Get the number of containers expected per environment (for verify).
# webserver runs a single standalone container; gateway runs gateway + rpc.
env_expected_containers() {
  if [ "$MODULE_REDIS_SCHEME" = "WEBSERVER" ]; then
    echo "1"
  else
    echo "2"
  fi
}

# Get database identity for an environment
env_db() {
  case "$1" in
    development) echo "$ENV_DEV_DB" ;;
    test) echo "$ENV_TEST_DB" ;;
    staging) echo "$ENV_STAGING_DB" ;;
    production) echo "$ENV_PROD_DB" ;;
    *) return 1 ;;
  esac
}

# Get Redis DB index for RTC state
env_redis_db1() {
  case "$1" in
    development) echo "$ENV_DEV_REDIS_DB1" ;;
    test) echo "$ENV_TEST_REDIS_DB1" ;;
    staging) echo "$ENV_STAGING_REDIS_DB1" ;;
    production) echo "$ENV_PROD_REDIS_DB1" ;;
    *) return 1 ;;
  esac
}

# Get Redis DB index for Web
env_redis_db2() {
  case "$1" in
    development) echo "$ENV_DEV_REDIS_DB2" ;;
    test) echo "$ENV_TEST_REDIS_DB2" ;;
    staging) echo "$ENV_STAGING_REDIS_DB2" ;;
    production) echo "$ENV_PROD_REDIS_DB2" ;;
    *) return 1 ;;
  esac
}

# Get Redis key prefix for environment isolation
env_redis_prefix() {
  case "$1" in
    development) echo "cloudrouter-dev" ;;
    test) echo "cloudrouter-test" ;;
    staging) echo "cloudrouter-staging" ;;
    production) echo "cloudrouter-prod" ;;
    *) return 1 ;;
  esac
}

# Get compose project name for an environment
compose_project() {
  echo "${MODULE_NAME}-$1"
}

# Resolve the directory that contains docker-compose.yml / docker-compose.external.yml.
# Priority:
#   1. explicit dir passed by caller (when --compose-dir is given and valid)
#   2. ./docker-compose.yml present in the current working directory
#   3. bundled templates shipped at <repo>/deployments/docker
#   4. fallback to the caller's dir (docker will then report a clear missing-file error)
# SCRIPT_DIR must be set by the sourcing script before calling this.
resolve_compose_dir() {
  local user_dir="${1:-}"
  if [ -n "$user_dir" ] && [ -f "$user_dir/docker-compose.yml" ]; then
    echo "$user_dir"; return 0
  fi
  if [ -f "./docker-compose.yml" ]; then
    echo "."; return 0
  fi
  local bundled
  bundled="$(cd "$SCRIPT_DIR/.." && pwd)/deployments/docker"
  if [ -f "$bundled/docker-compose.yml" ]; then
    echo "$bundled"; return 0
  fi
  echo "${user_dir:-.}"
}

# Get domain list for an environment across all brands
env_domains() {
  local subdomain
  subdomain=$(env_subdomain "$1")
  for domain in "${DEPLOY_BRAND_DOMAINS[@]}"; do
    echo "${subdomain}.${domain}"
  done
}

# Get primary domain (first brand)
env_primary_domain() {
  local subdomain
  subdomain=$(env_subdomain "$1")
  echo "${subdomain}.${DEPLOY_BRAND_DOMAINS[0]}"
}

# Get allowed hosts CSV for an environment
env_allowed_hosts() {
  local port
  port=$(env_port "$1")
  local hosts=""
  for domain in $(env_domains "$1"); do
    if [ -n "$hosts" ]; then
      hosts="${hosts},"
    fi
    hosts="${hosts}${domain}"
  done
  echo "${hosts},localhost:${port},127.0.0.1:${port}"
}

# Get CORS allowed origins CSV for an environment
env_cors_origins() {
  local port
  port=$(env_port "$1")
  local scheme="https"
  if [ "$1" != "production" ]; then
    scheme="http"
  fi
  local origins=""
  for domain in $(env_domains "$1"); do
    if [ -n "$origins" ]; then
      origins="${origins},"
    fi
    origins="${origins}${scheme}://${domain}"
  done
  # Localhost origins for non-production
  if [ "$1" != "production" ]; then
    origins="${origins},http://localhost:${port},http://127.0.0.1:${port}"
  fi
  # Desktop shell schemes and mini-program
  origins="${origins},app://dsh,app://birdcoder,app://sdkwork,app://dtupay,tauri://localhost,https://servicewechat.com"
  # Console origin for production
  if [ "$1" = "production" ]; then
    origins="${origins},https://console.sdkwork.com"
  fi
  echo "$origins"
}

# Get ACME profile for an environment
env_acme_profile() {
  case "$1" in
    production) echo "production" ;;
    *) echo "staging" ;;
  esac
}

# Get ACME directory URL
env_acme_directory() {
  case "$1" in
    production) echo "https://acme-v02.api.letsencrypt.org/directory" ;;
    *) echo "https://acme-staging-v02.api.letsencrypt.org/directory" ;;
  esac
}

# Get SSL mode for an environment
env_ssl_mode() {
  case "$1" in
    production|staging) echo "require" ;;
    *) echo "disable" ;;
  esac
}

# Validate environment name
validate_environment() {
  case "$1" in
    development|test|staging|production) return 0 ;;
    *)
      echo "unsupported environment: $1 (expected: development, test, staging, production)" >&2
      return 1
      ;;
  esac
}

# ============================================================================
# Utility Functions
# ============================================================================

# Check if a command exists
require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "required command not found: $1" >&2
    return 1
  fi
}

# Generate a random password (32 chars, URL-safe)
generate_password() {
  openssl rand -base64 32 | tr -d '=+/' | cut -c1-32
}

# Generate a random hex secret (64 chars)
generate_secret() {
  openssl rand -hex 32
}

# Print section header
section() {
  echo ""
  echo "==> $1"
}

# Print subsection header
subsection() {
  echo "  -> $1"
}

# Print success
success() {
  echo "  [OK] $1"
}

# Print warning
warn() {
  echo "  [WARN] $1" >&2
}

# Print error and exit
fail() {
  echo "  [FAIL] $1" >&2
  exit 1
}

# Confirm action (interactive)
confirm() {
  local prompt="${1:-Continue?}"
  local response
  read -r -p "$prompt [y/N] " response
  case "$response" in
    [yY][eE][sS]|[yY]) return 0 ;;
    *) return 1 ;;
  esac
}
