#!/bin/bash
# Dis Setup Script
# Automated installation and configuration

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}   Dis - Anonymous Chat Platform    ${NC}"
echo -e "${GREEN}=====================================${NC}"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo -e "${RED}Error: Do not run this script as root${NC}"
    exit 1
fi

# Check Docker installation
echo -e "${YELLOW}Checking Docker installation...${NC}"
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Docker is not installed. Please install Docker first.${NC}"
    echo "Visit: https://docs.docker.com/get-docker/"
    exit 1
fi

if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
    echo -e "${RED}Docker Compose is not installed. Please install Docker Compose first.${NC}"
    echo "Visit: https://docs.docker.com/compose/install/"
    exit 1
fi

echo -e "${GREEN}✓ Docker is installed${NC}"

# Check Docker daemon
if ! docker info &> /dev/null; then
    echo -e "${RED}Docker daemon is not running. Please start Docker.${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Docker daemon is running${NC}"

# Create directories
echo ""
echo -e "${YELLOW}Creating directories...${NC}"
mkdir -p data/{mongodb,redis,minio,caddy,caddy-config,tor}
mkdir -p backups
mkdir -p secrets
mkdir -p logs

echo -e "${GREEN}✓ Directories created${NC}"

# Generate secrets
echo ""
echo -e "${YELLOW}Generating secrets...${NC}"
./scripts/generate-secrets.sh

echo -e "${GREEN}✓ Secrets generated${NC}"

# Create .env file from example
if [ -f .env ]; then
    echo ""
    echo -e "${YELLOW}Warning: .env file already exists${NC}"
    read -p "Do you want to overwrite it? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Keeping existing .env file"
    else
        cp .env.example .env
        echo -e "${GREEN}✓ Created new .env file${NC}"
    fi
else
    cp .env.example .env
    echo -e "${GREEN}✓ Created .env file${NC}"
fi

# Load secrets into .env
if [ -f secrets/secrets.env ]; then
    echo ""
    echo -e "${YELLOW}Updating .env with generated secrets...${NC}"
    
    # Read secrets and update .env
    source secrets/secrets.env
    
    # Update .env file with actual secrets
    sed -i "s/CHANGE_ME_MONGODB_PASSWORD/${MONGODB_PASSWORD}/" .env
    sed -i "s/CHANGE_ME_MONGODB_ENCRYPTION_KEY/${MONGODB_ENCRYPTION_KEY}/" .env
    sed -i "s/CHANGE_ME_REDIS_PASSWORD/${REDIS_PASSWORD}/" .env
    sed -i "s/CHANGE_ME_MINIO_USER/${MINIO_ROOT_USER}/" .env
    sed -i "s/CHANGE_ME_MINIO_PASSWORD/${MINIO_ROOT_PASSWORD}/" .env
    sed -i "s/CHANGE_ME_DELTA_SECRET/${DELTA_SECRET}/" .env
    sed -i "s/CHANGE_ME_E2E_MASTER_KEY/${E2E_MASTER_KEY}/" .env
    sed -i "s/CHANGE_ME_SIGNAL_SERVER_KEY/${SIGNAL_SERVER_KEY}/" .env
    sed -i "s/CHANGE_ME_JWT_SECRET/${JWT_SECRET}/" .env
    sed -i "s/CHANGE_ME_EMAIL_ENCRYPTION_KEY/${EMAIL_ENCRYPTION_KEY}/" .env
    sed -i "s/CHANGE_ME_BACKUP_PASSWORD/${BACKUP_PASSWORD}/" .env
    
    echo -e "${GREEN}✓ Secrets loaded into .env${NC}"
fi

# Prompt for domain configuration
echo ""
echo -e "${YELLOW}Configuration${NC}"
read -p "Enter your domain (or IP address): " DOMAIN
sed -i "s/DOMAIN=example.com/DOMAIN=${DOMAIN}/" .env

read -p "Enable HTTPS? (Y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Nn]$ ]]; then
    sed -i "s/PROTOCOL=https/PROTOCOL=http/" .env
    sed -i "s/AUTO_HTTPS=true/AUTO_HTTPS=false/" .env
fi

read -p "Enter email for SSL certificates (optional): " ACME_EMAIL
if [ ! -z "$ACME_EMAIL" ]; then
    sed -i "s/ACME_EMAIL=/ACME_EMAIL=${ACME_EMAIL}/" .env
fi

read -p "Enable Tor hidden service? (Y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Nn]$ ]]; then
    sed -i "s/TOR_ENABLED=true/TOR_ENABLED=false/" .env
fi

# Set permissions
echo ""
echo -e "${YELLOW}Setting permissions...${NC}"
chmod 600 .env
chmod 600 secrets/secrets.env
chmod -R 700 data
chmod +x scripts/*.sh

echo -e "${GREEN}✓ Permissions set${NC}"

# Build custom images
echo ""
echo -e "${YELLOW}Building custom Docker images...${NC}"
docker-compose build

echo -e "${GREEN}✓ Docker images built${NC}"

# Display next steps
echo ""
echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}     Setup Complete!                 ${NC}"
echo -e "${GREEN}=====================================${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo ""
echo "1. Review and edit configuration:"
echo "   nano .env"
echo ""
echo "2. Start the services:"
echo "   docker-compose up -d"
echo ""
echo "3. Check the logs:"
echo "   docker-compose logs -f"
echo ""
echo "4. Access your instance:"
if grep -q "PROTOCOL=https" .env; then
    echo "   https://${DOMAIN}"
else
    echo "   http://${DOMAIN}"
fi
echo ""

if grep -q "TOR_ENABLED=true" .env; then
    echo "5. Get your Tor .onion address:"
    echo "   docker-compose logs tor | grep 'onion'"
    echo ""
fi

echo -e "${YELLOW}For more information, see:${NC}"
echo "   docs/INSTALL.md"
echo ""
