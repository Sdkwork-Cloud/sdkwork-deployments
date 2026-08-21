#!/bin/bash
# Install nginx reverse-proxy sites for SDKWork module containers.
#
# Generates and installs nginx server blocks that map environment-specific
# domains (api-dev.*, api-test.*, api-staging.*, api.*) to the per-environment
# host ports. Supports all brand domains as server_name aliases.
#
# Non-production environments listen on port 80 and proxy to the per-environment
# host port. Production also terminates TLS on 443 with HTTP->HTTPS redirect.
#
# Usage:
#   sudo ../sdkwork-deployments/scripts/install-nginx-sites.sh all
#   sudo ../sdkwork-deployments/scripts/install-nginx-sites.sh development production
#   sudo ../sdkwork-deployments/scripts/install-nginx-sites.sh all --module sdkwork-api-cloud-gateway --nginx-dir /etc/nginx/sites-enabled
#
# Options:
#   --module <name>      Module name for project identification
#   --nginx-dir <path>   Target nginx sites-enabled directory
#   --dry-run            Print configs without installing
#   --skip-reload        Skip nginx reload after install
#   -h, --help           Show help
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
source "$SCRIPT_DIR/lib/common.sh"

# ============================================================================
# Argument parsing
# ============================================================================
MODULE_NAME=""
NGINX_DIR="/etc/nginx/sites-enabled"
DRY_RUN=false
SKIP_RELOAD=false
TARGETS=()

while [ "$#" -gt 0 ]; do
  case "$1" in
    --module) MODULE_NAME="$2"; shift 2 ;;
    --nginx-dir) NGINX_DIR="$2"; shift 2 ;;
    --dry-run) DRY_RUN=true; shift ;;
    --skip-reload) SKIP_RELOAD=true; shift ;;
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
  echo "environment required (development, test, staging, production, or all)" >&2
  exit 2
fi

# Auto-detect module name
if [ -z "$MODULE_NAME" ]; then
  MODULE_NAME=$(basename "$(pwd)")
fi

init_module_config "$MODULE_NAME" "3900"

# Expand "all"
if [ "${TARGETS[0]}" = "all" ]; then
  TARGETS=(development test staging production)
fi

# ============================================================================
# Generate and install nginx config for each environment
# ============================================================================
install_one() {
  local env_name=$1
  validate_environment "$env_name" || exit 1

  local port
  port=$(env_port "$env_name")
  local domain
  domain=$(env_primary_domain "$env_name")
  local server_names=""
  local acme_section=""
  local tls_section=""

  # Build server_name list
  for d in $(env_domains "$env_name"); do
    if [ -n "$server_names" ]; then
      server_names="$server_names "
    fi
    server_names="$server_names$d"
  done

  # ACME challenge location (shared for all)
  acme_section="
    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }"

  # Production TLS server block
  if [ "$env_name" = "production" ]; then
    tls_section="
server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name $server_names;

    ssl_certificate /etc/ssl/sdkwork/api-gateway/fullchain.pem;
    ssl_certificate_key /etc/ssl/sdkwork/api-gateway/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;

    access_log /var/log/nginx/${domain}.access.log;
    error_log /var/log/nginx/${domain}.error.log;

    location / {
        proxy_pass http://127.0.0.1:${port};
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Forwarded-Host \$host;
        proxy_set_header X-Forwarded-Proto https;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection \"upgrade\";
        proxy_buffering off;
        proxy_read_timeout ${NGINX_PROXY_TIMEOUT}s;
        proxy_send_timeout ${NGINX_PROXY_TIMEOUT}s;
        client_max_body_size ${NGINX_CLIENT_MAX_BODY};
    }
}"
  fi

  # Generate config
  local config="# $domain (+ brand aliases) -> $env_name container gateway.
# Module: $MODULE_NAME | Host port: $port -> container $CONTAINER_PORT
# Installed by sdkwork-deployments/scripts/install-nginx-sites.sh
server {
    listen 80;
    listen [::]:80;
    server_name $server_names;
${acme_section}
$(if [ "$env_name" = "production" ]; then echo "
    location / {
        return 301 https://\$host\$request_uri;
    }"; else echo "
    access_log /var/log/nginx/${domain}.access.log;
    error_log /var/log/nginx/${domain}.error.log;

    location / {
        proxy_pass http://127.0.0.1:${port};
        proxy_http_version 1.1;
        proxy_set_header Host \$host;
        proxy_set_header X-Forwarded-Host \$host;
        proxy_set_header X-Forwarded-Proto \$scheme;
        proxy_set_header X-Real-IP \$remote_addr;
        proxy_set_header X-Forwarded-For \$proxy_add_x_forwarded_for;
        proxy_set_header Upgrade \$http_upgrade;
        proxy_set_header Connection \"upgrade\";
        proxy_buffering off;
        proxy_read_timeout ${NGINX_PROXY_TIMEOUT}s;
        proxy_send_timeout ${NGINX_PROXY_TIMEOUT}s;
        client_max_body_size ${NGINX_CLIENT_MAX_BODY}.
    }"; fi)
}${tls_section}
"

  if [ "$DRY_RUN" = true ]; then
    echo "--- $env_name ($domain.conf) ---"
    echo "$config"
    return 0
  fi

  # Install the config
  local target_dir="$NGINX_DIR/sdkwork"
  mkdir -p "$target_dir"
  echo "$config" > "$target_dir/$domain.conf"
  chmod 0644 "$target_dir/$domain.conf"
  success "Installed $domain.conf (port $port)"
}

# ============================================================================
# Main
# ============================================================================
section "Installing nginx sites for $MODULE_NAME"

if [ "$DRY_RUN" = true ]; then
  for env_name in "${TARGETS[@]}"; do
    install_one "$env_name"
  done
  echo ""
  echo "Dry run complete. Remove --dry-run to install."
  exit 0
fi

# Require root for actual installation
if [ "$(id -u)" -ne 0 ]; then
  echo "nginx installation requires root; rerun with sudo" >&2
  exit 1
fi

for env_name in "${TARGETS[@]}"; do
  install_one "$env_name"
done

# Create ACME webroot
mkdir -p /var/www/certbot
chmod 0755 /var/www/certbot

# Test and reload nginx
nginx -t
if [ "$SKIP_RELOAD" = false ]; then
  systemctl reload nginx || service nginx reload
  success "nginx reloaded; port 80 sites active"
else
  success "nginx config test passed (reload skipped)"
fi
