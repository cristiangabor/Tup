#!/usr/bin/env bash
# Updates a DuckDNS domain to point at the current public IP.
#
# Required environment variables (set DUCKDNS_SUBDOMAIN and DUCKDNS_TOKEN in
# /etc/duckdns.env, see install.sh in this folder):
#   DUCKDNS_SUBDOMAIN  e.g. "mytup" for mytup.duckdns.org
#   DUCKDNS_TOKEN      token from the duckdns.org dashboard
set -euo pipefail

ENV_FILE="/etc/duckdns.env"
if [[ -f "$ENV_FILE" ]]; then
  # shellcheck disable=SC1090
  source "$ENV_FILE"
fi

: "${DUCKDNS_SUBDOMAIN:?DUCKDNS_SUBDOMAIN not set (check /etc/duckdns.env)}"
: "${DUCKDNS_TOKEN:?DUCKDNS_TOKEN not set (check /etc/duckdns.env)}"

LOG_FILE="/var/log/duckdns.log"
RESPONSE=$(curl -fsS "https://www.duckdns.org/update?domains=${DUCKDNS_SUBDOMAIN}&token=${DUCKDNS_TOKEN}&ip=")

echo "$(date -Is) ${DUCKDNS_SUBDOMAIN}.duckdns.org -> ${RESPONSE}" >> "$LOG_FILE"

if [[ "$RESPONSE" != "OK"* ]]; then
  echo "DuckDNS update failed: $RESPONSE" >&2
  exit 1
fi
