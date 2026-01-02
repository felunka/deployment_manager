#!/usr/bin/env bash
set -euo pipefail

if [ "$EUID" -ne 0 ]; then
  echo "This script must be run as root." >&2
  exit 1
fi

BASE_URL="${1:-}"
BASE_DOMAIN="${2:-}"
ACME_EMAIL="${3:-}"

if [ -z "$BASE_URL" ] || [ -z "$BASE_DOMAIN" ] || [ -z "$ACME_EMAIL" ]; then
  echo "Usage: $0 BASE_URL BASE_DOMAIN ACME_EMAIL" >&2
  echo "Example: $0 https://example.com example.com test@example.com" >&2
  exit 1
fi

export DEBIAN_FRONTEND=noninteractive

hostnamectl set-hostname $BASE_DOMAIN

apt-get update
apt-get install -y --no-install-recommends \
  curl wget unzip gnupg2 ca-certificates lsb-release software-properties-common \
  apt-transport-https jq apache2-utils git

# Install or update Docker (Debian/Ubuntu)
if ! command -v docker >/dev/null 2>&1; then
  apt-get remove -y docker docker-engine docker.io containerd runc || true
  curl -fsSL https://download.docker.com/linux/$(. /etc/os-release; echo "$ID")/gpg | gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg
  echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/$(. /etc/os-release; echo "$ID") noble stable" > /etc/apt/sources.list.d/docker.list
  apt-get update
  apt-get install -y --no-install-recommends docker-ce docker-ce-cli containerd.io docker-compose-plugin
fi

# Create system user `node_agent` with home and no login
if ! id -u node_agent >/dev/null 2>&1; then
  useradd --system --create-home --home-dir /home/node_agent --shell /usr/sbin/nologin node_agent
fi
chown -R node_agent:node_agent /home/node_agent

# Ensure docker group exists and add node_agent to it so it can access the docker socket
groupadd -f docker || true
usermod -aG docker node_agent || true

# Add user to sudoers
sudo adduser node_agent sudo
# Allow node_agent to run svc.sh install/start/stop/status anywhere under /home/node_agent
echo 'node_agent ALL=(root) NOPASSWD: /home/node_agent/**/svc.sh install, /home/node_agent/**/svc.sh start, /home/node_agent/**/svc.sh stop, /home/node_agent/**/svc.sh status' \
| sudo tee /etc/sudoers.d/node_agent >/dev/null \
&& sudo chmod 0440 /etc/sudoers.d/node_agent \
&& sudo visudo -cf /etc/sudoers.d/node_agent

# Generate a secure 50-character key (25 hex chars) and hash with bcrypt
KEY=$(openssl rand -hex 25)
# Use htpasswd (apache2-utils) to create a bcrypt hash without exposing the password on the command line
HASH=$(printf "%s" "$KEY" | htpasswd -inB -C 10 node_agent_dummy 2>/dev/null | cut -d: -f2)
printf "%s" "$HASH" > /home/node_agent/.key.hash
chown node_agent:node_agent /home/node_agent/.key.hash
chmod 600 /home/node_agent/.key.hash

# Download and unpack agent_bundle.zip into the node_agent home
DEST_HOME=/home/node_agent
ZIP_URL="${BASE_URL%/}/agent_bundle.zip"
curl -fSL "$ZIP_URL" -o "$DEST_HOME/agent_bundle.zip"
unzip -o "$DEST_HOME/agent_bundle.zip" -d "$DEST_HOME"
rm "$DEST_HOME/agent_bundle.zip"
chown -R node_agent:node_agent "$DEST_HOME"

# Install systemd service to run the API host
cat >/etc/systemd/system/node_agent.service <<'EOF'
[Unit]
Description=Node Manager API Host
After=network.target

[Service]
Type=simple
User=node_agent
Group=node_agent
WorkingDirectory=/home/node_agent
ExecStart=/home/node_agent/api_host/server_agent 0.0.0.0:8080
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable --now node_agent.service || true

# Clone traefik repo and run docker compose
TRAEFIK_DIR=/home/node_agent/docker-traefik-letsencrypt
if [ ! -d "$TRAEFIK_DIR" ]; then
  git clone https://github.com/conscribtor/docker-traefik-letsencrypt "$TRAEFIK_DIR"
else
  (cd "$TRAEFIK_DIR" && git pull --ff-only) || true
fi

# move into traefik dir and write env + dynamic config
cd "$TRAEFIK_DIR"
## Generate DASHBOARD_DIGESTAUTH_TOKEN if not provided
if [ -z "${DASHBOARD_DIGESTAUTH_TOKEN:-}" ]; then
  DASHBOARD_USER=traefik
  DASHBOARD_REALM=traefik
  DASHBOARD_PW=$(tr -dc 'A-Za-z0-9' </dev/urandom | head -c28 || true)
  if [ -z "$DASHBOARD_PW" ]; then
    DASHBOARD_PW=$(openssl rand -base64 21)
  fi
  DASHBOARD_HA1=$(printf "%s:%s:%s" "$DASHBOARD_USER" "$DASHBOARD_REALM" "$DASHBOARD_PW" | md5sum | cut -d' ' -f1)
  DASHBOARD_DIGESTAUTH_TOKEN="${DASHBOARD_USER}:${DASHBOARD_REALM}:${DASHBOARD_HA1}"
  
  printf "%s\n" "$DASHBOARD_DIGESTAUTH_TOKEN" > /home/node_agent/.dashboard_auth
  chown node_agent:node_agent /home/node_agent/.dashboard_auth || true
  chmod 600 /home/node_agent/.dashboard_auth || true
fi
cat >"$TRAEFIK_DIR/.env" <<EOF
DOMAIN=$BASE_DOMAIN
ACME_EMAIL=$ACME_EMAIL
DASHBOARD_DIGESTAUTH_TOKEN=$DASHBOARD_DIGESTAUTH_TOKEN
EOF

DOCKER_BRIDGE_IP=$(ip -4 addr show docker0 | awk '/inet / {print $2}' | cut -d/ -f1)

cat >"$TRAEFIK_DIR/dynamic.yml" <<YML
http:
  routers:
    management-api:
      rule: "Host(\`management-api.$BASE_DOMAIN\`)"
      entryPoints:
        - websecure
      service: management-api-service
      tls:
        certResolver: letsencrypt
  services:
    management-api-service:
      loadBalancer:
        servers:
          - url: "http://$DOCKER_BRIDGE_IP:8080"
YML

# Compose override to mount the dynamic file into Traefik
cat >"$TRAEFIK_DIR/docker-compose.override.yml" <<'YML'
services:
  traefik:
    volumes:
      - ./dynamic.yml:/dynamic.yml:ro
    environment:
      - TRAEFIK_PROVIDERS_FILE_FILENAME=/dynamic.yml
YML

# Ensure files are owned by node_agent
chown node_agent:node_agent "$TRAEFIK_DIR/.env" "$TRAEFIK_DIR/dynamic.yml" "$TRAEFIK_DIR/docker-compose.override.yml"
if command -v docker >/dev/null 2>&1; then
  if docker compose version >/dev/null 2>&1; then
    docker compose -f docker-compose.yml -f docker-compose.prod.yml -f docker-compose.override.yml up -d
  elif command -v docker-compose >/dev/null 2>&1; then
    docker-compose -f docker-compose.yml -f docker-compose.prod.yml -f docker-compose.override.yml up -d
  else
    echo "Docker is installed but no compose plugin found. Please install docker-compose or docker compose plugin." >&2
  fi
fi

echo
echo
echo "Installation finished. Service enabled and started."
echo
echo
echo "Generated Traefik dashboard digest auth credentials:"
echo "User: $DASHBOARD_USER"
echo "Password: $DASHBOARD_PW"
echo
echo
echo "Generated key (paste to web frontend):"
echo "$KEY"
echo
echo
echo "ALL OF THESE VALUES CAN NEVER BE SHOWN AGAIN!"
