# Add Docker GPG key
echo 'Downloading docker gpg'
curl -fsSL https://download.docker.com/linux/debian/gpg | apt-key add -

# Add Docker repository
echo 'deb [arch=amd64] https://download.docker.com/linux/debian bullseye stable' > /etc/apt/sources.list.d/docker.list

# Create directories
mkdir -p /etc/docker

# Add daemon.json
# 
# Enable live restore in order to ensure that container stay alive
# if we update Docker.
#
# Enable IPv6 on the default bridge network.
cat << 'EOF' > /etc/docker/daemon.json
{
	"experimental": true,

	"live-restore": true,

	"ipv6": true,
	"fixed-cidr-v6": "2001:db8:1::/64",
	"ip6tables": true
}
EOF

# Install Docker
apt-get update -y
apt-get install -y docker-ce docker-ce-cli containerd.io

# Test Docker installation
docker run hello-world
