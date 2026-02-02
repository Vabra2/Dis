# 📖 Installation Guide - Dis

## Table of Contents

- [System Requirements](#system-requirements)
- [Quick Start](#quick-start)
- [Manual Installation](#manual-installation)
- [Configuration](#configuration)
- [First Run](#first-run)
- [Troubleshooting](#troubleshooting)

## System Requirements

### Minimum Requirements

- **CPU**: 2 cores
- **RAM**: 2GB available
- **Storage**: 10GB free space
- **OS**: Linux (Ubuntu 20.04+ recommended), macOS, or Windows with WSL2

### Software Requirements

- Docker 20.10 or later
- Docker Compose 2.0 or later
- OpenSSL (for secret generation)
- Git

### Network Requirements

- Ports 80 and 443 available (or custom ports)
- Domain name (optional, but recommended for HTTPS)
- Static IP or DDNS (for production deployment)

## Quick Start

The fastest way to get Dis running:

```bash
# Clone the repository
git clone https://github.com/Vabra2/Dis.git
cd Dis

# Run setup script
chmod +x scripts/setup.sh
./scripts/setup.sh

# Edit configuration (set your domain, etc.)
nano .env

# Start the services
docker-compose up -d

# Check status
docker-compose ps
```

Your Dis instance should now be running!

## Manual Installation

### Step 1: Clone Repository

```bash
git clone https://github.com/Vabra2/Dis.git
cd Dis
```

### Step 2: Create Directories

```bash
mkdir -p data/{mongodb,redis,minio,caddy,caddy-config,tor}
mkdir -p backups
mkdir -p secrets
mkdir -p logs
```

### Step 3: Generate Secrets

```bash
chmod +x scripts/generate-secrets.sh
./scripts/generate-secrets.sh
```

This creates `secrets/secrets.env` with all cryptographic keys.

### Step 4: Configure Environment

```bash
# Copy example configuration
cp .env.example .env

# Load generated secrets
source secrets/secrets.env

# Update .env with your configuration
nano .env
```

Key settings to configure:

```bash
# Your domain or IP
DOMAIN=your-domain.com

# Protocol (http or https)
PROTOCOL=https

# Email for SSL certificates
ACME_EMAIL=admin@your-domain.com

# Enable/disable features
TOR_ENABLED=true
EMAIL_ENABLED=false
```

### Step 5: Configure Secrets

Update `.env` with secrets from `secrets/secrets.env`:

```bash
# Update MongoDB password
sed -i "s/CHANGE_ME_MONGODB_PASSWORD/${MONGODB_PASSWORD}/" .env

# Update all other secrets similarly
# (or use the setup script which does this automatically)
```

### Step 6: Build Images

```bash
docker-compose build
```

### Step 7: Start Services

```bash
# Start in background
docker-compose up -d

# Or start with logs visible
docker-compose up
```

### Step 8: Verify Installation

```bash
# Check all containers are running
docker-compose ps

# View logs
docker-compose logs -f

# Check specific service
docker-compose logs delta
```

## Configuration

### Basic Configuration

Edit `.env` file for basic settings:

```bash
# Server
DOMAIN=example.com
PROTOCOL=https
TZ=UTC

# Database
MONGODB_PASSWORD=your-secure-password
REDIS_PASSWORD=your-secure-password

# Storage
MINIO_ROOT_USER=admin
MINIO_ROOT_PASSWORD=your-secure-password

# Security
JWT_SECRET=your-jwt-secret
E2E_MASTER_KEY=your-encryption-key
```

### Advanced Configuration

#### Custom Ports

If ports 80/443 are already in use:

```yaml
# In docker-compose.yml, change caddy ports:
caddy:
  ports:
    - "8080:80"    # HTTP on 8080
    - "8443:443"   # HTTPS on 8443
```

#### Self-Signed Certificates

For testing without a domain:

```caddyfile
# In Caddyfile, uncomment:
tls internal
```

#### Email Configuration

To enable email verification:

```bash
# In .env
EMAIL_ENABLED=true
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
SMTP_USE_TLS=true
```

#### Tor Hidden Service

Tor is enabled by default. To get your .onion address:

```bash
docker-compose logs tor | grep "onion"
```

Or check the file:
```bash
cat data/tor/hostname
```

### Docker Compose Profiles

Dis uses profiles to enable/disable features:

```bash
# Start with email service
docker-compose --profile email up -d

# Start with log anonymization
docker-compose --profile logging up -d

# Start with Tor
docker-compose --profile tor up -d

# Start with all features
docker-compose --profile email --profile logging --profile tor up -d
```

## First Run

### 1. Access Web Interface

Open your browser and navigate to:
- HTTP: `http://your-domain.com`
- HTTPS: `https://your-domain.com`
- Tor: `http://your-onion-address.onion`

### 2. Create Admin Account

On first launch, create an admin account:

1. Click "Register"
2. Choose a username (anonymous ID will be generated)
3. Enter a strong password
4. Optionally add encrypted email for recovery
5. Complete the PoW CAPTCHA

### 3. Create First Server

1. Click "Create Server"
2. Enter server name and description
3. Configure channels and roles
4. Invite members (share server link)

### 4. Configure E2E Encryption

Encryption keys are generated automatically, but you can:

1. Backup your identity key
2. Verify other users' keys
3. Enable encryption for private channels

## Monitoring

### View Logs

```bash
# All services
docker-compose logs -f

# Specific service
docker-compose logs -f delta
docker-compose logs -f bonfire
docker-compose logs -f mongodb

# Last 100 lines
docker-compose logs --tail=100
```

### Check Resources

```bash
# Container resource usage
docker stats

# Disk usage
docker system df
```

### Health Checks

```bash
# Check all containers
docker-compose ps

# Test API endpoint
curl http://localhost:8000/

# Test WebSocket
curl http://localhost:9000/
```

## Backup

### Manual Backup

```bash
chmod +x scripts/backup.sh
./scripts/backup.sh
```

Backups are stored in `backups/` directory, encrypted with `BACKUP_PASSWORD`.

### Automated Backups

Add to crontab:

```bash
# Backup daily at 2 AM
0 2 * * * cd /path/to/Dis && ./scripts/backup.sh
```

### Restore Backup

```bash
# Decrypt backup
openssl enc -aes-256-cbc -d -pbkdf2 \
  -in backups/dis_backup_20240101_020000.tar.gz.enc \
  -out backup.tar.gz \
  -pass "pass:${BACKUP_PASSWORD}"

# Extract
tar -xzf backup.tar.gz

# Restore MongoDB
docker exec -i dis-mongodb mongorestore \
  --username="${MONGODB_USERNAME}" \
  --password="${MONGODB_PASSWORD}" \
  --authenticationDatabase=admin \
  /path/to/backup/mongodb
```

## Updates

### Update Dis

```bash
# Pull latest changes
git pull

# Rebuild containers
docker-compose build

# Restart services
docker-compose down
docker-compose up -d
```

### Update Revolt Images

```bash
# Pull latest Revolt images
docker-compose pull

# Restart services
docker-compose down
docker-compose up -d
```

## Uninstall

### Stop and Remove

```bash
# Stop all services
docker-compose down

# Remove volumes (⚠️ deletes all data!)
docker-compose down -v

# Remove images
docker-compose down --rmi all
```

### Complete Cleanup

```bash
# Remove all Dis data
rm -rf data/
rm -rf backups/
rm -rf secrets/
rm -rf logs/

# Remove configuration
rm .env
```

## Troubleshooting

### Services Won't Start

**Check Docker:**
```bash
docker info
docker-compose version
```

**Check logs:**
```bash
docker-compose logs
```

**Check ports:**
```bash
# Check if ports are in use
netstat -tulpn | grep -E '(80|443|8000|9000)'
```

### Can't Connect to Web Interface

**Check Caddy logs:**
```bash
docker-compose logs caddy
```

**Check firewall:**
```bash
# Ubuntu/Debian
sudo ufw status
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
```

**Test directly:**
```bash
curl http://localhost:3000  # Frontend
curl http://localhost:8000  # API
```

### Database Connection Errors

**Check MongoDB:**
```bash
docker-compose logs mongodb
docker exec -it dis-mongodb mongosh
```

**Check Redis:**
```bash
docker-compose logs redis
docker exec -it dis-redis redis-cli ping
```

**Verify credentials:**
```bash
# Check .env file has correct passwords
grep MONGODB_PASSWORD .env
grep REDIS_PASSWORD .env
```

### SSL Certificate Issues

**Let's Encrypt rate limits:**
- Use staging environment first
- Check certificate logs

**Self-signed for testing:**
```caddyfile
# In Caddyfile
tls internal
```

### Tor Hidden Service Not Working

**Check Tor logs:**
```bash
docker-compose logs tor
```

**Verify hidden service:**
```bash
cat data/tor/hostname
```

**Test connectivity:**
```bash
# From within Tor network
curl --socks5 localhost:9050 http://your-onion.onion
```

### Performance Issues

**Increase resources:**
```yaml
# In docker-compose.yml
services:
  delta:
    deploy:
      resources:
        limits:
          memory: 1G
```

**Check resource usage:**
```bash
docker stats
```

**Optimize MongoDB:**
```bash
# Increase cache size
# In docker-compose.yml:
command: --wiredTigerCacheSizeGB 2.0
```

## Getting Help

- 📖 Documentation: `docs/`
- 🐛 Issues: [GitHub Issues](https://github.com/Vabra2/Dis/issues)
- 💬 Community: Create a discussion
- 📧 Security: Report privately to security@example.com

## Next Steps

After installation:

1. Read [SECURITY.md](SECURITY.md) for security best practices
2. Read [ANONYMITY.md](ANONYMITY.md) for anonymity guarantees
3. Configure backups and monitoring
4. Set up firewall and security hardening
5. Test disaster recovery procedures
