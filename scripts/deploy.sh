#!/bin/bash
# Deploy SDKWork module environments via Docker Compose (external dependency mode).
#
# Deploys one or all lifecycle environments as isolated compose projects with
# distinct host ports, domain lists, and database identities. Uses external
# PostgreSQL and Redis (configured via .env files).
#
# This script is module-agnostic: auto-detects the current module or accepts
# --module to target a specific module workspace.
#
# Usage:
#   ../sdkwork-deployments/scripts/deploy.sh <environment|all> [options]
#   ../sdkwork-deployments/scripts/deploy.sh development
#   ../sdkwork-deployments/scripts/deploy.sh all --validate
#   ../sdkwork-deployments/scripts/deploy.sh production --down
#
# Environments: development, test, staging, production, all
#
# Options:
#   --module <name>       Target module (auto-detected from cwd)
#   --env-dir <path>      Path to env files (default: ./docker/env)
#   --compose-dir <path>  Path to compose files (default: cwd)
#   --pull                docker compose pull before up
#   --down                Stop the selected stack instead of starting
#   --validate            Validate env file before deploy
#   --force-recreate      Force container recreation
#   -h, --help            Show help
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
source "$SCRIPT_DIR/lib/common.sh"

# ============================================================================
# Argument parsing
# ============================================================================
TARGET=""
MODULE_NAME=""
ENV_DIR="./docker/env"
COMPOSE_DIR="."
PULL=false
DOWN=false
VALIDATE=false
FORCE_RECREATE=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --module) MODULE_NAME="$2"; shift 2 ;;
    --env-dir) ENV_DIR="$2"; shift 2 ;;
    --compose-dir) COMPOSE_DIR="$2"; shift 2 ;;
    --pull) PULL=true; shift ;;
    --down) DOWN=true; shift ;;
    --validate) VALIDATE=true; shift ;;
    --force-recreate) FORCE_RECREATE=true; shift ;;
    -h|--help)
      head -25 "$0" | grep -E '^#' | sed 's/^# \?//'
      exit 0
      ;;
    -*)
      echo "unknown option: $1" >&2
      exit 2
      ;;
    *)
      if [ -z "$TARGET" ]; then
        TARGET="$1"
      else
        echo "unexpected argument: $1" >&2
        exit 2
      fi
      shift
      ;;
  esac
done

if [ -z "$TARGET" ]; then
  echo "environment required (development, test, staging, production, or all)" >&2
  exit 2
fi

# Auto-detect module name
if [ -z "$MODULE_NAME" ]; then
  MODULE_NAME=$(basename "$(pwd)")
fi

init_module_config "$MODULE_NAME" "3900"

# Resolve compose file directory: use --compose-dir if valid, else cwd, else the
# bundled templates shipped at deployments/docker so the scripts work out of the box.
COMPOSE_DIR=$(resolve_compose_dir "$COMPOSE_DIR")
subsection "Compose dir: $COMPOSE_DIR"

# ============================================================================
# Deploy a single environment
# ============================================================================
deploy_one() {
  local env_name=$1
  validate_environment "$env_name" || exit 1

  local env_file="$ENV_DIR/$env_name.env"
  local project
  project=$(compose_project "$env_name")

  if [ ! -f "$env_file" ]; then
    echo "  missing env file: $env_file" >&2
    echo "  run: $SCRIPT_DIR/generate-env.sh --output-dir $ENV_DIR" >&2
    exit 1
  fi

  # Validate env file if requested
  if [ "$VALIDATE" = true ] && [ "$DOWN" = false ]; then
    subsection "Validating $env_name.env"
    # Source the env file and check required vars
    set -a
    source "$env_file"
    set +a
    # Check critical variables
    local required_vars=(
      "${MODULE_ENV_PREFIX}_HOST_PORT"
      "${MODULE_ENV_PREFIX}_POSTGRES_HOST"
      "${MODULE_ENV_PREFIX}_POSTGRES_DB"
      "${MODULE_ENV_PREFIX}_CLOUDROUTER_REDIS_HOST"
    )
    for var in "${required_vars[@]}"; do
      if [ -z "${!var:-}" ]; then
        echo "  missing required var: $var" >&2
        exit 1
      fi
    done
    success "Validation passed"
  fi

  # Build compose command
  local compose_args=(
    --env-file "$env_file"
    -p "$project"
    -f "$COMPOSE_DIR/docker-compose.yml"
    -f "$COMPOSE_DIR/docker-compose.external.yml"
  )

  if [ "$DOWN" = true ]; then
    subsection "Stopping $env_name ($project)"
    docker compose "${compose_args[@]}" down
    success "Stopped $env_name"
    return 0
  fi

  subsection "Deploying $env_name ($project)"

  if [ "$PULL" = true ]; then
    docker compose "${compose_args[@]}" pull
  fi

  local up_args=("-d")
  if [ "$FORCE_RECREATE" = true ]; then
    up_args+=("--force-recreate")
  fi

  docker compose "${compose_args[@]}" up "${up_args[@]}"

  # Extract host port for status message
  local port
  port=$(grep -E "^${MODULE_ENV_PREFIX}_HOST_PORT=" "$env_file" | cut -d= -f2-)
  success "Deployed $env_name -> http://127.0.0.1:${port}/healthz"
}

# ============================================================================
# Main
# ============================================================================
section "Deploying $MODULE_NAME"

# Check Docker is available and the daemon is running
require_command docker || fail "Docker is not installed or not in PATH"
if ! docker info >/dev/null 2>&1; then
  fail "Docker daemon is not running (start Docker and retry)"
fi

case "$TARGET" in
  development|test|staging|production)
    deploy_one "$TARGET"
    ;;
  all)
    for env_name in development test staging production; do
      deploy_one "$env_name"
    done
    ;;
  *)
    echo "unsupported environment: $TARGET" >&2
    exit 2
    ;;
esac

section "Deployment complete"
