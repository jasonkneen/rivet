# Create dir to hold TLS certs
#
# The Traefik install script also creates these directories (and chown them),
# but we need the dirs to exist for the rivet_fetch_gg_tls.sh script to run before
# Traefik is installed when using initialize_immediately.
mkdir -p /etc/__TRAEFIK_INSTANCE_NAME__/dynamic/tls /etc/__TRAEFIK_INSTANCE_NAME__/tls

# Write script
cat << 'EOF' > /usr/bin/rivet_fetch_gg_tls.sh
#!/usr/bin/env bash
set -eu -o pipefail

CERT_ID="job"
STUB="/etc/__TRAEFIK_INSTANCE_NAME__/tls/$CERT_ID"


# Retry script every 5 seconds until success
echo 'Fetching rivet tls'
while true; do
  response=$(
    curl -f \
      -H "Authorization: Bearer __SERVER_TOKEN__" \
      "__TUNNEL_API_EDGE_API__/provision/datacenters/__DATACENTER_ID__/tls"
  ) && break || sleep 5
done

echo "TLS received"

# Write tls certs
echo $response | jq -r .job_cert_pem > "${STUB}_cert.pem"
echo $response | jq -r .job_private_key_pem > "${STUB}_key.pem"

# Write traefik config file
cat << EOF2 > "/etc/__TRAEFIK_INSTANCE_NAME__/dynamic/tls/${CERT_ID}.toml"
[[tls.certificates]]
  certFile = "${STUB}_cert.pem"
  keyFile = "${STUB}_key.pem"
EOF2

# Force config reload
touch /etc/__TRAEFIK_INSTANCE_NAME__/dynamic
EOF

chmod +x /usr/bin/rivet_fetch_gg_tls.sh

# Create systemd service file
cat << 'EOF' > /etc/systemd/system/rivet_fetch_gg_tls.service
[Unit]
Description=Rivet TLS Fetch
Requires=network-online.target
After=network-online.target

[Service]
User=root
Group=root
Type=oneshot
ExecStart=/usr/bin/rivet_fetch_gg_tls.sh

# Real time service
CPUSchedulingPolicy=fifo
# High CPU priority
CPUSchedulingPriority=90
# Prevent killing from system OOM
OOMScoreAdjust=-800

[Install]
WantedBy=multi-user.target
EOF

# Create systemd timer file
cat << 'EOF' > /etc/systemd/system/rivet_fetch_gg_tls.timer
[Unit]
Description=Runs TLS fetch every minute
Requires=network-online.target
After=network-online.target

[Timer]
# Run immediately on startup
OnBootSec=0
# Trigger every hour
OnCalendar=*:0
# Prevent stampeding herd
RandomizedDelaySec=60
Unit=rivet_fetch_gg_tls.service

[Install]
WantedBy=timers.target
EOF

# Enable tls fetch script to run on reboot
systemctl daemon-reload
systemctl enable rivet_fetch_gg_tls.timer
systemctl enable rivet_fetch_gg_tls.service
