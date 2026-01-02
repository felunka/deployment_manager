#!/usr/bin/env bash
set -euo pipefail

if [ "$EUID" -ne 0 ]; then
  echo "This script must be run as root." >&2
  exit 1
fi

BASE_URL="${1:-}"

if [ -z "$BASE_URL" ]; then
  echo "Usage: $0 BASE_URL" >&2
  echo "Example: $0 https://example.com" >&2
  exit 1
fi

export DEBIAN_FRONTEND=noninteractive

# Stop web server
systemctl stop node_agent.service

# Download and unpack agent_bundle.zip into the node_agent home
DEST_HOME=/home/node_agent
ZIP_URL="${BASE_URL%/}/agent_bundle.zip"
curl -fSL "$ZIP_URL" -o "$DEST_HOME/agent_bundle.zip"
unzip -o "$DEST_HOME/agent_bundle.zip" -d "$DEST_HOME"
rm "$DEST_HOME/agent_bundle.zip"
chown -R node_agent:node_agent "$DEST_HOME"

# Start web server
systemctl start node_agent.service
