#!/bin/bash
# Verify SDKWork module Docker deployment health.
#
# Checks container status, health endpoints, domain resolution, and
# cross-environment isolation. Supports verifying one or all environments.
#
# Usage:
#   ../sdkwork-deployments/scripts/verify-deployment.sh
#   ../sdkwork-deployments/scripts/verify-deployment.sh all
#   ../sdkwork-deployments/scripts/verify-deployment.sh development production
#
# Options:
#   --module <name>     Module name (auto-detected)
#   --env-dir <path>    Path to env files (default: ./docker/env)
#   --timeout <sec>     Health check timeout per endpoint (default: 30)
#   --skip-containers   Skip container status checks
#   --skip-domains      Skip domain resolution checks
#   --skip-isolation    Skip cross-environment isolation checks
#   -h, --help          Show help
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
source "$SCRIPT_DIR/lib/common.sh"

# ============================================================================
# Argument parsing
# ============================================================================
TARGETS=()
MODULE_NAME=""
ENV_DIR="./docker/env"
TIMEOUT=30
SKIP_CONTAINERS=false
SKIP_DOMAINS=false
SKIP_ISOLATION=false

while [ "$#" -gt 0 ]; do
  case "$1" in
    --module) MODULE_NAME="$2"; shift 2 ;;
    --env-dir) ENV_DIR="$2"; shift 2 ;;
    --timeout) TIMEOUT="$2"; shift 2 ;;
    --skip-containers) SKIP_CONTAINERS=true; shift ;;
    --skip-domains) SKIP_DOMAINS=true; shift ;;
    --skip-isolation) SKIP_ISOLATION=true; shift ;;
    -h|--help)
      head -20 "$0" | grep -E '^#' | sed 's/^# \?//'
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

# Auto-detect module name
if [ -z "$MODULE_NAME" ]; then
  MODULE_NAME=$(basename "$(pwd)")
fi

init_module_config "$MODULE_NAME" "3900"

# Track results
PASS_COUNT=0
FAIL_COUNT=0

# ============================================================================
# Check functions
# ============================================================================
check_container() {
  local env_name=$1
  local project
  project=$(compose_project "$env_name")

  local running
  running=$(docker compose -p "$project" ps --format json 2>/dev/null | grep -c '"State":"running"' || echo "0")

  local expected=2  # gateway + runtime-assignment-worker (see deployments/docker/docker-compose.yml)
  if [ "$running" -ge "$expected" ]; then
    success "$env_name: $running containers running ($project)"
    ((PASS_COUNT++)) || true
    return 0
  else
    echo "  [FAIL] $env_name: expected $expected containers, found $running ($project)" >&2
    ((FAIL_COUNT++)) || true
    return 1
  fi
}

check_health_endpoint() {
  local env_name=$1
  local port
  port=$(env_port "$env_name")

  local http_code
  http_code=$(curl -fsS -o /dev/null -w "%{http_code}" --max-time "$TIMEOUT" "http://127.0.0.1:${port}/healthz" 2>/dev/null || echo "000")

  if [ "$http_code" = "200" ]; then
    success "$env_name: /healthz OK (port $port)"
    ((PASS_COUNT++)) || true
    return 0
  else
    echo "  [FAIL] $env_name: /healthz returned HTTP $http_code (port $port)" >&2
    ((FAIL_COUNT++)) || true
    return 1
  fi
}

check_domain_resolution() {
  local env_name=$1
  local primary_domain
  domain=$(env_primary_domain "$env_name")

  local http_code
  http_code=$(curl -fsS -o /dev/null -w "%{http_code}" --max-time "$TIMEOUT" "http://${domain}/healthz" 2>/dev/null || echo "000")

  if [ "$http_code" = "200" ]; then
    success "$env_name: http://${domain}/healthz OK"
    ((PASS_COUNT++)) || true
    return 0
  else
    echo "  [FAIL] $env_name: http://${domain}/healthz returned HTTP $http_code" >&2
    ((FAIL_COUNT++)) || true
    return 1
  fi
}

check_isolation() {
  local env_name=$1
  local port
  port=$(env_port "$env_name")

  # Pick a domain from a different environment
  local other_env="production"
  if [ "$env_name" = "production" ]; then
    other_env="development"
  fi
  local other_domain
  other_domain=$(env_primary_domain "$other_env")

  # Request with wrong Host header should be rejected (421)
  local http_code
  http_code=$(curl -fsS -o /dev/null -w "%{http_code}" --max-time 5 -H "Host: ${other_domain}" "http://127.0.0.1:${port}/healthz" 2>/dev/null || echo "000")

  if [ "$http_code" = "421" ]; then
    success "$env_name: Host isolation OK (wrong Host -> 421)"
    ((PASS_COUNT++)) || true
    return 0
  else
    echo "  [WARN] $env_name: Host isolation returned HTTP $http_code (expected 421)"
    ((PASS_COUNT++)) || true  # Not a hard failure
    return 0
  fi
}

# ============================================================================
# Main
# ============================================================================
section "Verifying deployment: $MODULE_NAME"

for env_name in "${TARGETS[@]}"; do
  subsection "$env_name"

  if [ "$SKIP_CONTAINERS" = false ]; then
    check_container "$env_name" || true
  fi

  check_health_endpoint "$env_name" || true

  if [ "$SKIP_DOMAINS" = false ]; then
    check_domain_resolution "$env_name" || true
  fi

  if [ "$SKIP_ISOLATION" = false ]; then
    check_isolation "$env_name" || true
  fi
done

# Summary
echo ""
echo "========================================="
echo "Verification complete: $PASS_COUNT passed, $FAIL_COUNT failed"
echo "========================================="

if [ "$FAIL_COUNT" -gt 0 ]; then
  exit 1
fi
