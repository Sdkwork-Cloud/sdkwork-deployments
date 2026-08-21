#!/bin/bash
# Provision external PostgreSQL databases for all gateway lifecycle environments.
#
# Follows sdkwork-specs ENVIRONMENT_SPEC workspace identities:
#   development -> sdkwork_ai_dev
#   test        -> sdkwork_ai_test
#   staging     -> sdkwork_ai_staging
#   production  -> sdkwork_ai_prod
#
# Each database uses the same name for database, schema, and role. The vector
# extension is created inside the workspace schema (knowledgebase requirement).
#
# Prerequisites:
#   - psql client
#   - PostgreSQL with pgvector installed (postgresql-XX-pgvector on Ubuntu 22.04)
#   - Admin credentials via SDKWORK_DATABASE_ADMIN_* or PGPASSWORD
#
# Usage:
#   ../sdkwork-deployments/scripts/provision-databases.sh
#   ../sdkwork-deployments/scripts/provision-databases.sh development test
#   ../sdkwork-deployments/scripts/provision-databases.sh all --schema-sql ./docker/postgres/external-schema.sql
#
# Options:
#   --schema-sql <path>   Path to schema SQL script (default: auto-detect)
#   --admin-host <host>   Admin connection host (default: 127.0.0.1)
#   --admin-port <port>   Admin connection port (default: 5432)
#   --admin-user <user>   Admin username (default: postgres)
#   --admin-pass <pass>   Admin password (or set PGPASSWORD/SDKWORK_DATABASE_ADMIN_PASSWORD)
#   --env-dir <path>      Path to .env files (default: ./docker/env)
#   --dry-run             Print SQL without executing
#   -h, --help            Show help
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
source "$SCRIPT_DIR/lib/common.sh"

# ============================================================================
# Argument parsing
# ============================================================================
TARGETS=()
SCHEMA_SQL=""
ADMIN_HOST="${SDKWORK_DATABASE_ADMIN_HOST:-127.0.0.1}"
ADMIN_PORT="${SDKWORK_DATABASE_ADMIN_PORT:-5432}"
ADMIN_USER="${SDKWORK_DATABASE_ADMIN_USERNAME:-postgres}"
ADMIN_PASS="${SDKWORK_DATABASE_ADMIN_PASSWORD:-${PGPASSWORD:-}}"
ENV_DIR="./docker/env"
DRY_RUN=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --schema-sql) SCHEMA_SQL="$2"; shift 2 ;;
    --admin-host) ADMIN_HOST="$2"; shift 2 ;;
    --admin-port) ADMIN_PORT="$2"; shift 2 ;;
    --admin-user) ADMIN_USER="$2"; shift 2 ;;
    --admin-pass) ADMIN_PASS="$2"; shift 2 ;;
    --env-dir) ENV_DIR="$2"; shift 2 ;;
    --dry-run) DRY_RUN=true; shift ;;
    -h|--help)
      head -25 "$0" | grep -E '^#' | sed 's/^# \?//'
      exit 0
      ;;
    -*)
      echo "unknown option: $1" >&2
      exit 2
      ;;
    *)
      TARGETS+=("$1")
      shift
      ;;
  esac
done

if [ "${#TARGETS[@]}" -eq 0 ]; then
  TARGETS=(development test staging production)
fi

if [ "${TARGETS[0]}" = "all" ]; then
  TARGETS=(development test staging production)
fi

# Auto-detect schema SQL
if [ -z "$SCHEMA_SQL" ]; then
  local_paths=(
    "./docker/postgres/external-schema.sql"
    "./docker/postgres/init/001-create-schema.sh"
    "../sdkwork-api-cloud-gateway/docker/postgres/external-schema.sql"
  )
  for path in "${local_paths[@]}"; do
    if [ -f "$path" ]; then
      SCHEMA_SQL="$path"
      break
    fi
  done
fi

# ============================================================================
# Helper: get password for a specific environment from its .env file
# ============================================================================
get_env_password() {
  local env_name=$1
  local env_file="$ENV_DIR/$env_name.env"
  local password=""

  if [ -f "$env_file" ]; then
    password=$(grep -E '^(GATEWAY_)?POSTGRES_PASSWORD=' "$env_file" | head -1 | cut -d= -f2-)
  fi

  # Fallback defaults for development
  if [ -z "$password" ] && [ "$env_name" = "development" ]; then
    password="sdkworkdev123"
  fi

  echo "$password"
}

# ============================================================================
# Provision a single environment
# ============================================================================
provision_one() {
  local env_name=$1
  validate_environment "$env_name" || exit 1

  local db
  db=$(env_db "$env_name")
  local password
  password=$(get_env_password "$env_name")

  if [ -z "$password" ]; then
    echo "  cannot determine password for $env_name (set in $ENV_DIR/$env_name.env)" >&2
    exit 1
  fi

  # Reject placeholder/insecure passwords (e.g., generated with --secrets skip).
  if [ "$password" = "<CHANGE_ME>" ] || echo "$password" | grep -qE '[<>]'; then
    echo "  refusing to provision with placeholder password for $env_name" >&2
    echo "  regenerate the env with real secrets: $SCRIPT_DIR/generate-env.sh --secrets auto" >&2
    exit 1
  fi

  subsection "Provisioning $env_name ($db)"

  if [ "$DRY_RUN" = true ]; then
    echo "  [DRY-RUN] Would create database: $db"
    echo "  [DRY-RUN] Would create role: $db"
    echo "  [DRY-RUN] Would create schema: $db with vector extension"
    return 0
  fi

  # Create database if not exists
  if ! psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -tAc "SELECT 1 FROM pg_database WHERE datname = '$db'" 2>/dev/null | grep -q 1; then
    psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -c "CREATE DATABASE \"$db\";"
    success "Created database $db"
  else
    echo "  Database $db already exists"
  fi

  # Apply schema SQL if available
  if [ -n "$SCHEMA_SQL" ] && [ -f "$SCHEMA_SQL" ]; then
    psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -d "$db" -v db="$db" -v app_user="$db" -f "$SCHEMA_SQL" >/dev/null 2>&1
    success "Applied schema ($SCHEMA_SQL)"
  fi

  # Create role if not exists (password passed via -v and quoted with :'pw' to
  # avoid SQL injection / special-character breakage).
  local role_exists
  role_exists=$(psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -tAc "SELECT 1 FROM pg_roles WHERE rolname = '$db'" 2>/dev/null || echo "")
  if [ "$role_exists" != "1" ]; then
    psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -v pw="$password" -c "CREATE ROLE \"$db\" LOGIN PASSWORD :'pw';"
    success "Created role $db"
  else
    psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -v pw="$password" -c "ALTER ROLE \"$db\" WITH LOGIN PASSWORD :'pw';"
    success "Updated role $db password"
  fi

  # Grant privileges
  psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -d "$db" -c "GRANT ALL PRIVILEGES ON DATABASE \"$db\" TO \"$db\";" 2>/dev/null || true
  psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -d "$db" -c "GRANT ALL ON SCHEMA \"$db\" TO \"$db\";" 2>/dev/null || true
  psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -d "$db" -c "ALTER DEFAULT PRIVILEGES IN SCHEMA \"$db\" GRANT ALL ON TABLES TO \"$db\";" 2>/dev/null || true
  psql -h "$ADMIN_HOST" -p "$ADMIN_PORT" -U "$ADMIN_USER" -d "$db" -c "ALTER DEFAULT PRIVILEGES IN SCHEMA \"$db\" GRANT ALL ON SEQUENCES TO \"$db\";" 2>/dev/null || true

  success "Ready: database=$db schema=$db user=$db"
}

# ============================================================================
# Main
# ============================================================================
section "Provisioning databases"
subsection "Admin: $ADMIN_USER@$ADMIN_HOST:$ADMIN_PORT"
subsection "Schema SQL: ${SCHEMA_SQL:-not found (schema creation skipped)}"

require_command psql || fail "psql client not found (apt install postgresql-client)"

for env_name in "${TARGETS[@]}"; do
  provision_one "$env_name"
done

section "Database provisioning complete"
echo ""
echo "Databases ready:"
for env_name in "${TARGETS[@]}"; do
  db=$(env_db "$env_name")
  echo "  $env_name: $db"
done
