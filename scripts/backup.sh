#!/bin/bash
# Dis Backup Script
# Creates encrypted backups of MongoDB and configuration

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Load environment
if [ -f .env ]; then
    source .env
else
    echo -e "${RED}Error: .env file not found${NC}"
    exit 1
fi

# Configuration
BACKUP_DIR="./backups"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_NAME="dis_backup_${TIMESTAMP}"
TEMP_DIR="/tmp/${BACKUP_NAME}"

echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}   Dis Backup Script                ${NC}"
echo -e "${GREEN}=====================================${NC}"
echo ""

# Create directories
mkdir -p "${BACKUP_DIR}"
mkdir -p "${TEMP_DIR}"

echo -e "${YELLOW}Creating backup: ${BACKUP_NAME}${NC}"

# Backup MongoDB
echo -e "${YELLOW}Backing up MongoDB...${NC}"
docker exec dis-mongodb mongodump \
    --username="${MONGODB_USERNAME}" \
    --password="${MONGODB_PASSWORD}" \
    --authenticationDatabase=admin \
    --out="${TEMP_DIR}/mongodb" \
    --gzip

echo -e "${GREEN}✓ MongoDB backed up${NC}"

# Backup configuration files
echo -e "${YELLOW}Backing up configuration...${NC}"
mkdir -p "${TEMP_DIR}/config"
cp .env "${TEMP_DIR}/config/" 2>/dev/null || true
cp docker-compose.yml "${TEMP_DIR}/config/"
cp Caddyfile "${TEMP_DIR}/config/"

echo -e "${GREEN}✓ Configuration backed up${NC}"

# Backup Tor hidden service keys (if exists)
if [ -d "data/tor" ]; then
    echo -e "${YELLOW}Backing up Tor keys...${NC}"
    mkdir -p "${TEMP_DIR}/tor"
    cp -r data/tor/* "${TEMP_DIR}/tor/" 2>/dev/null || true
    echo -e "${GREEN}✓ Tor keys backed up${NC}"
fi

# Create archive
echo -e "${YELLOW}Creating archive...${NC}"
cd /tmp
tar -czf "${BACKUP_NAME}.tar.gz" "${BACKUP_NAME}"

# Encrypt backup
echo -e "${YELLOW}Encrypting backup...${NC}"
if [ -z "${BACKUP_PASSWORD}" ]; then
    echo -e "${RED}Error: BACKUP_PASSWORD not set in .env${NC}"
    exit 1
fi

openssl enc -aes-256-cbc -salt -pbkdf2 \
    -in "${BACKUP_NAME}.tar.gz" \
    -out "${BACKUP_NAME}.tar.gz.enc" \
    -pass "pass:${BACKUP_PASSWORD}"

# Move to backup directory
mv "${BACKUP_NAME}.tar.gz.enc" "${BACKUP_DIR}/"

# Cleanup
rm -rf "${TEMP_DIR}"
rm "${BACKUP_NAME}.tar.gz"

# Get file size
BACKUP_SIZE=$(du -h "${BACKUP_DIR}/${BACKUP_NAME}.tar.gz.enc" | cut -f1)

echo -e "${GREEN}✓ Backup completed${NC}"
echo ""
echo "Backup file: ${BACKUP_DIR}/${BACKUP_NAME}.tar.gz.enc"
echo "Size: ${BACKUP_SIZE}"
echo ""

# Cleanup old backups based on retention policy
if [ ! -z "${BACKUP_RETENTION}" ]; then
    echo -e "${YELLOW}Cleaning up old backups (keeping last ${BACKUP_RETENTION})...${NC}"
    cd "${BACKUP_DIR}"
    ls -t dis_backup_*.tar.gz.enc | tail -n +$((BACKUP_RETENTION + 1)) | xargs -r rm
    echo -e "${GREEN}✓ Old backups cleaned up${NC}"
fi

echo ""
echo -e "${GREEN}=====================================${NC}"
echo -e "${GREEN}   Backup Complete!                 ${NC}"
echo -e "${GREEN}=====================================${NC}"
echo ""
echo "To restore this backup, run:"
echo "  ./scripts/restore.sh ${BACKUP_NAME}.tar.gz.enc"
echo ""
