#!/bin/bash
# Generate Cryptographic Secrets for Dis
# Creates secure random keys for encryption and authentication

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Generating cryptographic secrets...${NC}"

# Create secrets directory if it doesn't exist
mkdir -p secrets

# Generate secrets
MONGODB_PASSWORD=$(openssl rand -base64 32)
MONGODB_ENCRYPTION_KEY=$(openssl rand -base64 32)
REDIS_PASSWORD=$(openssl rand -base64 32)
MINIO_ROOT_USER=$(openssl rand -base64 16 | tr -d '+/=' | cut -c1-16)
MINIO_ROOT_PASSWORD=$(openssl rand -base64 32)
DELTA_SECRET=$(openssl rand -hex 32)
E2E_MASTER_KEY=$(openssl rand -base64 32)
SIGNAL_SERVER_KEY=$(openssl rand -base64 32)
JWT_SECRET=$(openssl rand -base64 64)
EMAIL_ENCRYPTION_KEY=$(openssl rand -base64 32)
BACKUP_PASSWORD=$(openssl rand -base64 32)

# Save to file
cat > secrets/secrets.env << EOF
# Generated secrets for Dis
# Generated on: $(date)
# KEEP THIS FILE SECURE!

MONGODB_PASSWORD=${MONGODB_PASSWORD}
MONGODB_ENCRYPTION_KEY=${MONGODB_ENCRYPTION_KEY}
REDIS_PASSWORD=${REDIS_PASSWORD}
MINIO_ROOT_USER=${MINIO_ROOT_USER}
MINIO_ROOT_PASSWORD=${MINIO_ROOT_PASSWORD}
DELTA_SECRET=${DELTA_SECRET}
E2E_MASTER_KEY=${E2E_MASTER_KEY}
SIGNAL_SERVER_KEY=${SIGNAL_SERVER_KEY}
JWT_SECRET=${JWT_SECRET}
EMAIL_ENCRYPTION_KEY=${EMAIL_ENCRYPTION_KEY}
BACKUP_PASSWORD=${BACKUP_PASSWORD}
EOF

# Set secure permissions
chmod 600 secrets/secrets.env

echo -e "${GREEN}✓ Secrets generated and saved to secrets/secrets.env${NC}"
echo -e "${YELLOW}⚠️  Keep this file secure and never commit it to version control!${NC}"

# Display secrets (optional, for immediate use)
if [ "$1" == "--show" ]; then
    echo ""
    echo "Generated secrets:"
    cat secrets/secrets.env
fi
