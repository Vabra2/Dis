# 🏗️ Architecture - Dis

## Overview

Dis is a self-hosted, anonymous, encrypted chat platform built on Revolt with enhanced privacy and security features. This document describes the system architecture, components, and data flows.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         Client Layer                         │
├─────────────────────────────────────────────────────────────┤
│  Web Browser │ Tor Browser │ Mobile App │ Desktop App      │
└──────┬───────────────┬────────────────────┬─────────────────┘
       │               │                    │
       │ HTTPS         │ .onion            │ WSS
       │               │                    │
┌──────▼───────────────▼────────────────────▼─────────────────┐
│                   Reverse Proxy (Caddy)                      │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │   SSL/TLS   │  │ Rate Limit   │  │    Headers   │       │
│  └─────────────┘  └──────────────┘  └──────────────┘       │
└──────┬──────────────────┬────────────────┬──────────────────┘
       │                  │                │
┌──────▼──────┐    ┌──────▼──────┐  ┌─────▼──────┐
│   January   │    │    Delta    │  │  Bonfire   │
│  (Frontend) │    │ (REST API)  │  │ (WebSocket)│
└─────────────┘    └──────┬──────┘  └─────┬──────┘
                          │                │
       ┌──────────────────┴────────────────┴──────────────────┐
       │                                                       │
┌──────▼───────┐  ┌────────────┐  ┌──────────┐  ┌──────────┐│
│   MongoDB    │  │   Redis    │  │  MinIO   │  │  Vortex  ││
│  (Database)  │  │  (Cache)   │  │  (S3)    │  │ (Voice)  ││
└──────────────┘  └────────────┘  └──────────┘  └──────────┘│
                                                               │
┌──────────────────────────────────────────────────────────────┤
│                    Support Services                          │
├──────────────────────────────────────────────────────────────┤
│  Email Service │ Log Anonymizer │ Tor Service │ Backup      │
└──────────────────────────────────────────────────────────────┘
```

## Component Details

### 1. Client Layer

#### Web Client (January)

**Technology**: Preact (React-like)
**Port**: 3000 (internal)
**Purpose**: User interface

**Features:**
- Responsive design
- Real-time updates
- E2E encryption UI
- PWA support

**Security:**
- CSP headers
- XSS protection
- CSRF tokens
- Secure session storage

#### Tor Browser Support

**Access**: Via .onion address
**Benefits**: IP anonymity, censorship resistance
**Configuration**: Automatic with Tor service

### 2. Reverse Proxy (Caddy)

**Technology**: Caddy 2
**Ports**: 80 (HTTP), 443 (HTTPS)
**Purpose**: Reverse proxy, SSL, routing

**Responsibilities:**
```
1. SSL/TLS termination
2. HTTP → HTTPS redirect
3. Security headers
4. Rate limiting
5. Load balancing
6. Log anonymization
7. Health checks
```

**Configuration**: `Caddyfile`

**Routes:**
```
/           → January (Frontend)
/api/*      → Delta (REST API)
/ws/*       → Bonfire (WebSocket)
/voice/*    → Vortex (Voice)
/files/*    → MinIO (Storage)
```

**Security Features:**
- Automatic HTTPS (Let's Encrypt)
- TLS 1.3 only
- Strong cipher suites
- HSTS enabled
- Security headers
- Request sanitization

### 3. Backend Services

#### Delta (REST API)

**Technology**: Rust (Rocket framework)
**Port**: 8000
**Purpose**: HTTP REST API

**Endpoints:**
```
POST   /auth/register          - User registration
POST   /auth/login             - User login
GET    /users/:id              - User info
POST   /servers                - Create server
GET    /servers/:id            - Server info
POST   /channels/:id/messages  - Send message
GET    /channels/:id/messages  - Get messages
```

**Features:**
- JSON API
- JWT authentication
- Rate limiting
- Input validation
- Error handling

**Database**: MongoDB
**Cache**: Redis

#### Bonfire (WebSocket)

**Technology**: Rust (Actix Web)
**Port**: 9000
**Purpose**: Real-time communication

**Events:**
```
Client → Server:
- Authenticate
- Subscribe to channels
- Send messages
- User presence
- Typing indicators

Server → Client:
- New messages
- User joined/left
- Presence updates
- Notifications
```

**Features:**
- WebSocket protocol
- Message queue (Redis)
- Pub/sub for scaling
- Connection pooling
- Heartbeat/ping-pong

#### Vortex (Voice)

**Technology**: Rust (WebRTC)
**Port**: 9001
**Purpose**: Voice channels

**Features:**
- WebRTC signaling
- SRTP encryption
- Opus codec
- Low latency
- TURN/STUN support

**Security:**
- DTLS key exchange
- SRTP encryption
- No server-side recording
- Ephemeral keys

### 4. Data Layer

#### MongoDB

**Version**: 7.0
**Port**: 27017 (internal)
**Purpose**: Primary database

**Collections:**
```
users:
  - _id (anonymous ID)
  - username
  - password_hash (Argon2)
  - identity_key
  - encrypted_email
  - created_at

messages:
  - _id
  - channel_id
  - author_id
  - encrypted_content
  - timestamp

channels:
  - _id
  - server_id
  - name
  - type

servers:
  - _id
  - name
  - owner_id
  - members
```

**Security:**
- Authentication enabled
- Encrypted at rest (WiredTiger)
- Encrypted connections
- Field-level encryption
- Minimal profiling

**Initialization**: `database/mongodb/init.js`

#### Redis

**Version**: 7
**Port**: 6379 (internal)
**Purpose**: Cache, sessions, pub/sub

**Data Structures:**
```
Sessions:
  session:{token} → user_id, expires_at

Presence:
  presence:{user_id} → online, last_active

Cache:
  cache:{key} → value (TTL)

Pub/Sub:
  channel:{id} → messages
```

**Security:**
- Password protected
- TLS optional
- Dangerous commands disabled
- Limited memory
- Encrypted backups

**Configuration**: `database/redis/redis.conf`

#### MinIO (Object Storage)

**Version**: Latest
**Port**: 9000 (internal)
**Purpose**: File storage (S3-compatible)

**Buckets:**
```
revolt-files:
  - User avatars
  - Attachments
  - Server icons
  - Emojis
```

**Features:**
- S3-compatible API
- Server-side encryption
- Access policies
- Versioning
- Lifecycle policies

**Security:**
- Encrypted storage
- Access key authentication
- Bucket policies
- Encrypted backups

### 5. Support Services

#### Email Service

**Technology**: Python
**Purpose**: Send verification emails (optional)

**Features:**
- SMTP integration
- Fernet encryption
- Email hashing
- HTML templates
- Queue support

**Location**: `email/`

**Security:**
- Emails encrypted before storage
- SHA-256 hash for lookup
- TLS for SMTP
- No plaintext storage

#### Log Anonymizer

**Technology**: Python
**Purpose**: Remove PII from logs

**Process:**
```
1. Monitor log files
2. Regex pattern matching
3. Hash IP addresses
4. Redact sensitive data
5. Write clean logs
```

**Location**: `logs/anonymizer.py`

**Patterns Removed:**
- IP addresses → Hash
- Email addresses → Redacted
- Tokens → Redacted
- User-Agent → Removed
- Cookies → Removed

#### Log Retention Manager

**Technology**: Python
**Purpose**: Delete old logs

**Process:**
```
1. Check log age
2. Delete logs older than retention period
3. Secure deletion
4. Report statistics
```

**Location**: `logs/retention.py`

**Configuration**: `LOG_RETENTION_DAYS=7`

#### Tor Service

**Technology**: Tor
**Purpose**: Hidden service (.onion)

**Features:**
- v3 onion address
- Anonymous access
- Automatic setup
- Persistent keys

**Configuration**: `tor/torrc`

**Ports:**
```
80  → Caddy (HTTP)
443 → Caddy (HTTPS)
```

## Data Flow

### User Registration

```
1. Client:
   - User enters username & password
   - Generate anonymous ID: "anon_[UUID]"
   - Solve PoW CAPTCHA
   - Hash password with Argon2 (client-side)

2. Client → Delta API:
   POST /auth/register
   {
     "username": "alice",
     "password_hash": "$argon2...",
     "captcha_solution": 12345
   }

3. Delta:
   - Verify CAPTCHA solution
   - Check username availability
   - Generate identity key pair
   - Store in MongoDB

4. Delta → Client:
   {
     "user_id": "anon_abc123",
     "token": "jwt_token",
     "identity_key": "public_key"
   }

5. Client:
   - Store token securely
   - Store private key locally
```

### Sending Message

```
1. Client:
   - Write message
   - Encrypt with recipient's public key (E2E)
   - Sign with sender's private key

2. Client → Bonfire WebSocket:
   {
     "type": "message",
     "channel_id": "channel_123",
     "encrypted_content": "base64_ciphertext",
     "signature": "base64_signature"
   }

3. Bonfire:
   - Verify JWT token
   - Check channel permissions
   - Store in MongoDB (encrypted)
   - Publish to Redis pub/sub

4. Bonfire → Other Clients:
   {
     "type": "message",
     "channel_id": "channel_123",
     "author_id": "anon_abc123",
     "encrypted_content": "base64_ciphertext",
     "timestamp": "2024-01-01T00:00:00Z"
   }

5. Recipients:
   - Decrypt with private key
   - Verify signature
   - Display message
```

### Voice Call

```
1. Client A:
   - Request voice channel join
   
2. Vortex:
   - Generate ICE candidates
   - STUN/TURN for NAT traversal
   
3. Client A ↔ Vortex:
   - WebRTC signaling (SDP exchange)
   - DTLS handshake
   - SRTP key exchange
   
4. Client A ↔ Client B:
   - Direct P2P connection (if possible)
   - Or relay through TURN
   - Encrypted audio (SRTP)
   - Opus codec
```

## Security Architecture

### Encryption Layers

```
┌─────────────────────────────────────────┐
│  Application Layer (E2E)                │
│  Signal Protocol, AES-256-GCM           │
├─────────────────────────────────────────┤
│  Transport Layer                        │
│  TLS 1.3, Perfect Forward Secrecy       │
├─────────────────────────────────────────┤
│  Storage Layer                          │
│  MongoDB Encryption, MinIO Encryption   │
├─────────────────────────────────────────┤
│  Network Layer (Optional)               │
│  Tor, Anonymous Routing                 │
└─────────────────────────────────────────┘
```

### Authentication Flow

```
1. Anonymous Registration:
   Username + Password → Argon2 Hash → MongoDB

2. Login:
   Username + Password → Verify Hash → JWT Token

3. API Requests:
   JWT Token → Verify → Access Granted

4. WebSocket:
   JWT Token → Authenticate → Subscribe

5. E2E Encryption:
   Identity Keys → X3DH → Double Ratchet
```

## Deployment Architecture

### Single Server

```
┌────────────────────────────────────────┐
│         Docker Host                    │
│  ┌──────────────────────────────────┐ │
│  │  All containers on one machine   │ │
│  │  - Caddy                          │ │
│  │  - January                        │ │
│  │  - Delta                          │ │
│  │  - Bonfire                        │ │
│  │  - MongoDB                        │ │
│  │  - Redis                          │ │
│  │  - MinIO                          │ │
│  │  - Support services               │ │
│  └──────────────────────────────────┘ │
└────────────────────────────────────────┘

Pros: Simple, low cost
Cons: Single point of failure
Use: Development, small deployments
```

### Multi-Server (Future)

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Frontend   │     │   Backend   │     │  Database   │
│   Cluster   │────▶│   Cluster   │────▶│   Cluster   │
│  Caddy x3   │     │ Delta x3    │     │ MongoDB x3  │
│  January x3 │     │ Bonfire x3  │     │ Redis x3    │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                   │
       └───────────────────┴───────────────────┘
                           │
                    ┌──────▼──────┐
                    │Load Balancer│
                    └─────────────┘

Pros: Scalable, redundant
Cons: Complex, expensive
Use: Large deployments
```

## Scaling Considerations

### Vertical Scaling

Increase resources per container:
```yaml
services:
  delta:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
```

### Horizontal Scaling

Multiple instances:
```yaml
services:
  delta:
    deploy:
      replicas: 3
```

### Database Scaling

- MongoDB replica set
- Redis cluster
- MinIO distributed mode

### CDN Integration

For static assets:
- CloudFlare
- Fastly
- Custom CDN

## Monitoring & Observability

### Metrics

**What to monitor:**
- CPU/Memory usage
- Disk I/O
- Network traffic
- Request rate
- Error rate
- Response time
- WebSocket connections
- Database queries

**Tools:**
- Prometheus (metrics)
- Grafana (dashboards)
- cAdvisor (container stats)
- Node Exporter (system stats)

### Logging

**Log levels:**
- ERROR: Critical issues
- WARN: Important events
- INFO: General information
- DEBUG: Detailed debugging (dev only)

**Log aggregation:**
- Loki (log aggregation)
- Grafana (visualization)
- Fluent Bit (log forwarding)

### Alerting

**Alert on:**
- Service down
- High error rate
- Disk space low
- Memory usage high
- SSL certificate expiring

**Tools:**
- Alertmanager
- PagerDuty
- Email/SMS

## Backup & Recovery

### Backup Strategy

**What to backup:**
1. MongoDB data
2. Configuration files (.env)
3. Tor hidden service keys
4. SSL certificates

**Backup frequency:**
- Daily: Full backup
- Hourly: Incremental (optional)

**Backup script:** `scripts/backup.sh`

### Disaster Recovery

**Recovery Time Objective (RTO):** < 1 hour
**Recovery Point Objective (RPO):** < 24 hours

**Process:**
```
1. Deploy new server
2. Install Docker
3. Clone repository
4. Restore .env
5. Restore MongoDB from backup
6. Restore Tor keys
7. Start services
8. Verify operation
```

## Performance Optimization

### Database Optimization

```javascript
// Create indexes
db.messages.createIndex({ channel_id: 1, timestamp: -1 })
db.users.createIndex({ username: 1 })

// Query optimization
db.messages.find({ channel_id: "123" })
  .sort({ timestamp: -1 })
  .limit(50)
```

### Caching Strategy

```
Level 1: Browser cache (static assets)
Level 2: CDN cache (images, JS)
Level 3: Redis cache (API responses)
Level 4: MongoDB (persistent storage)
```

### Connection Pooling

```yaml
# Database connection pool
DB_POOL_SIZE=50
MAX_WS_CONNECTIONS=10000
```

## Security Hardening

### Network Security

```bash
# Firewall rules
ufw allow 80/tcp   # HTTP
ufw allow 443/tcp  # HTTPS
ufw deny 8000/tcp  # Block direct Delta access
ufw deny 9000/tcp  # Block direct Bonfire access
```

### Container Security

```yaml
# Run as non-root
user: "1000:1000"

# Read-only filesystem
read_only: true

# Drop capabilities
cap_drop:
  - ALL

# Security options
security_opt:
  - no-new-privileges:true
```

### Secret Management

```bash
# Use secrets manager
docker secret create mongodb_password secrets/mongodb.pass

# Or encrypted env files
ansible-vault encrypt .env
```

## Troubleshooting

### Common Issues

1. **Containers won't start**
   - Check logs: `docker-compose logs`
   - Check ports: `netstat -tulpn`
   - Check resources: `docker stats`

2. **Database connection failed**
   - Verify credentials in .env
   - Check MongoDB logs
   - Test connection: `mongosh`

3. **SSL certificate issues**
   - Check domain DNS
   - Verify ports 80/443 open
   - Check Caddy logs

4. **WebSocket disconnects**
   - Check reverse proxy config
   - Verify timeout settings
   - Check network stability

## Development

### Local Development

```bash
# Start only required services
docker-compose up mongodb redis minio

# Run backend locally
cd delta && cargo run

# Run frontend locally
cd january && npm run dev
```

### Testing

```bash
# Unit tests
cargo test

# Integration tests
./scripts/test-integration.sh

# Load testing
k6 run tests/load-test.js
```

## References

- [Revolt Documentation](https://developers.revolt.chat/)
- [Signal Protocol](https://signal.org/docs/)
- [Docker Documentation](https://docs.docker.com/)
- [MongoDB Documentation](https://docs.mongodb.com/)
- [Redis Documentation](https://redis.io/documentation)

## Conclusion

Dis architecture is designed for privacy, security, and ease of deployment. The modular design allows for customization and scaling based on needs.

For questions or improvements, open an issue on GitHub.
