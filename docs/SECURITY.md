# 🛡️ Security - Dis

## Overview

Dis implements multiple layers of security to protect user communications and privacy. This document describes the security architecture, cryptographic implementations, and best practices.

## Security Architecture

### Defense in Depth

Dis uses a multi-layered security approach:

1. **Transport Layer**: TLS 1.3 for all connections
2. **Application Layer**: E2E encryption for messages
3. **Storage Layer**: Encrypted at rest
4. **Network Layer**: Tor support for anonymity
5. **Authentication**: Anonymous with strong passwords

## Cryptography

### End-to-End Encryption

**Protocol**: Signal Protocol (X3DH + Double Ratchet)

#### Key Exchange (X3DH)

```
Alice → Bob: Identity Key, Ephemeral Key
Bob → Alice: Signed PreKey, One-Time PreKey
Shared Secret = DH(EphemeralA, SignedPrekeyB) || 
                DH(EphemeralA, IdentityB) ||
                DH(EphemeralA, OnetimePreKeyB)
```

**Key Properties:**
- Perfect Forward Secrecy
- Future Secrecy (Break-in Recovery)
- Deniability
- Asynchronous (offline messaging)

#### Double Ratchet

Continuous key rotation for messages:
- Symmetric-key ratchet for new keys per message
- Diffie-Hellman ratchet for session renewal
- Out-of-order message handling

#### Implementation

**Location**: `encryption/src/signal_protocol.rs`

**Algorithms:**
- **Key Exchange**: X25519 (Curve25519 ECDH)
- **Signing**: Ed25519
- **Encryption**: AES-256-GCM
- **KDF**: HMAC-SHA256

**Key Sizes:**
- Identity Key: 256 bits
- Pre-Keys: 256 bits
- Message Keys: 256 bits
- Chain Keys: 256 bits

### Group Chat Encryption

**Protocol**: Sender Keys (Signal Groups)

Each message is encrypted with:
- Unique session key per group
- Message counter for replay protection
- AES-256-GCM for encryption

**Location**: `encryption/src/signal_protocol.rs` - `GroupSession`

### Password Hashing

**Algorithm**: Argon2id

**Parameters:**
```
Memory: 65536 KB (64 MB)
Iterations: 3
Parallelism: 4
Output: 32 bytes
```

**Location**: `auth/src/anonymous_token.rs`

**Security Properties:**
- Resistant to GPU attacks
- Resistant to side-channel attacks
- Memory-hard function
- Time-cost trade-off resistant

### Authentication Tokens

**Format**: JWT (JSON Web Tokens)

**Algorithm**: HMAC-SHA256

**Structure:**
```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "anon_user_id",
    "iat": 1234567890,
    "exp": 1234567890,
    "jti": "random_uuid"
  }
}
```

**Expiration**: 30 days (configurable)

**Location**: `auth/src/anonymous_token.rs`

### Email Encryption

**Algorithm**: AES-256-GCM

**Implementation**: Fernet (symmetric encryption)

Email addresses (if provided) are:
1. Encrypted with AES-256-GCM
2. Hashed with SHA-256 for lookup
3. Never stored in plaintext

**Location**: `email/sender.py`

### Database Encryption

**MongoDB**: Client-Side Field Level Encryption

Sensitive fields encrypted before storage:
- Email addresses
- Identity keys
- Session tokens

**Redis**: Encrypted connection (optional TLS)

### Transport Security

**TLS 1.3** enforced for all connections:
- Perfect Forward Secrecy (PFS)
- Modern cipher suites only
- HSTS enabled
- Certificate pinning (recommended for clients)

**Ciphers Allowed:**
```
TLS_AES_128_GCM_SHA256
TLS_AES_256_GCM_SHA384
TLS_CHACHA20_POLY1305_SHA256
```

## Authentication & Authorization

### Anonymous Registration

Users register without personal information:

```
1. Generate random user ID: "anon_[UUID]"
2. User chooses password
3. Hash password with Argon2id
4. Optional: Encrypt email for recovery
5. Complete PoW CAPTCHA
```

**No tracking:**
- No email required
- No phone number required
- No real name required
- No IP address stored

### Proof-of-Work CAPTCHA

**Purpose**: Prevent automated registration spam

**Algorithm**: SHA-256 hash puzzle

**Difficulty**: Configurable (default: 4 leading zero bits)

**Process:**
```python
challenge = random_bytes(32)
solution = find_nonce_where(
    SHA256(challenge + nonce) starts with N zero bits
)
```

**Location**: `auth/src/captcha.rs`

**Anti-DDoS:**
- Client-side computation
- Adjustable difficulty
- Rate limiting per IP hash
- No external services

### Session Management

**Token Storage**: Redis with expiration

**Session Lifetime:**
- Default: 30 days
- Max: 90 days
- Idle timeout: 1 hour (configurable)

**Security Features:**
- Automatic token rotation
- Revocation support
- Device fingerprinting (optional)
- Concurrent session limits

## Network Security

### Tor Support

**Hidden Service**: v3 .onion address

**Features:**
- Anonymous access
- No IP address exposure
- Censorship resistance
- NAT traversal

**Configuration**: `tor/torrc`

### Reverse Proxy (Caddy)

**Security Headers:**
```
X-Frame-Options: SAMEORIGIN
X-Content-Type-Options: nosniff
X-XSS-Protection: 1; mode=block
Strict-Transport-Security: max-age=31536000
Content-Security-Policy: [restrictive policy]
Referrer-Policy: strict-origin-when-cross-origin
```

**Rate Limiting:**
- 100 requests per minute per IP
- Adjustable per endpoint
- DDoS protection

**Location**: `Caddyfile`

## Data Security

### Data at Rest

**MongoDB:**
- WiredTiger encryption
- Encrypted backups
- Secure deletion

**MinIO (Files):**
- Server-side encryption
- Encrypted backups
- Access control

**Redis:**
- Encrypted backups
- Secure deletion of expired keys

### Data in Transit

- TLS 1.3 for all HTTP/WebSocket
- SRTP for voice (WebRTC)
- Encrypted backups

### Data Minimization

**What we store:**
- Anonymous user IDs
- Encrypted messages (E2E)
- Password hashes (Argon2)
- Encrypted emails (optional)

**What we DON'T store:**
- IP addresses (anonymized in logs)
- User-Agent strings (removed)
- Email plaintext
- Unencrypted messages
- Personal information

## Logging & Monitoring

### Log Anonymization

**Automatic removal of:**
- IP addresses → SHA-256 hash
- Email addresses → redacted
- JWT tokens → redacted
- Session IDs → redacted
- User-Agent → removed

**Location**: `logs/anonymizer.py`

**Process:**
```python
1. Regex patterns detect sensitive data
2. IP addresses hashed with salt
3. Tokens redacted
4. Clean logs written
```

### Log Retention

**Default**: 7 days

**Features:**
- Automatic deletion
- Configurable retention period
- Encrypted backups
- Secure deletion

**Location**: `logs/retention.py`

### Security Monitoring

Logs are monitored for:
- Failed authentication attempts
- Unusual access patterns
- Potential attacks
- Service errors

**No personal data logged.**

## Backup Security

### Encrypted Backups

**Algorithm**: AES-256-CBC with PBKDF2

**Process:**
```bash
1. Create backup archive
2. Encrypt with openssl
3. Store with access controls
4. Automatic rotation
```

**Location**: `scripts/backup.sh`

**Best Practices:**
- Store backups off-site
- Use strong backup password
- Regular backup testing
- Encrypted transport

## Vulnerability Management

### Dependency Security

**Rust Dependencies:**
```bash
cargo audit
cargo outdated
```

**Python Dependencies:**
```bash
pip-audit
safety check
```

**Docker Images:**
```bash
docker scan
trivy scan
```

### Security Updates

**Process:**
1. Monitor security advisories
2. Test updates in staging
3. Apply patches promptly
4. Document changes

**Channels:**
- GitHub Security Advisories
- Rust Security Advisory Database
- Python CVE Database
- Docker Security Announcements

## Threat Model

### Protected Against

✅ Man-in-the-middle attacks (TLS + E2E)
✅ Server compromise (E2E encryption)
✅ Database breach (encrypted storage)
✅ Network surveillance (Tor support)
✅ Metadata collection (minimal logging)
✅ Spam/DDoS (PoW CAPTCHA + rate limiting)
✅ Password attacks (Argon2id)

### NOT Protected Against

❌ Client compromise (keylogger, malware)
❌ User sharing credentials
❌ Quantum computers (future threat)
❌ Advanced persistent threats with client access
❌ Social engineering

## Security Best Practices

### For Operators

1. **Use strong secrets**: Generate with `generate-secrets.sh`
2. **Enable HTTPS**: Use Let's Encrypt or valid certificates
3. **Regular backups**: Automated and encrypted
4. **Update regularly**: Monitor security advisories
5. **Monitor logs**: Check for unusual activity
6. **Limit access**: Use firewall rules
7. **Use Tor**: Enable hidden service
8. **Secure server**: Follow OS hardening guides

### For Users

1. **Strong passwords**: Use password manager
2. **Verify keys**: Check identity keys of contacts
3. **Enable 2FA**: If email recovery enabled
4. **Use Tor**: For maximum anonymity
5. **Backup keys**: Save identity key securely
6. **Update client**: Keep browser/app updated
7. **Trust on first use**: Verify keys after initial connection

## Security Audits

### Planned Audits

- [ ] Third-party security audit
- [ ] Penetration testing
- [ ] Code review by security experts
- [ ] Cryptography review

### Self-Audit Checklist

- [ ] All dependencies updated
- [ ] No hardcoded secrets
- [ ] TLS properly configured
- [ ] Logs properly anonymized
- [ ] Backups encrypted
- [ ] Rate limiting enabled
- [ ] Firewall configured
- [ ] Monitoring active

## Reporting Security Issues

### Responsible Disclosure

**DO:**
- Report privately first
- Provide detailed information
- Allow time to fix

**DON'T:**
- Publicly disclose before fix
- Exploit for malicious purposes
- Access systems without permission

### Contact

**Email**: security@example.com (use PGP)

**PGP Key**: [Provide PGP key fingerprint]

**Response Time**: Within 48 hours

**Disclosure Timeline**: 90 days

## Compliance

### GDPR Considerations

Dis is designed for privacy:
- Minimal data collection
- Anonymous by default
- User controls their data
- Right to be forgotten (account deletion)
- No personal data processing

### Legal Disclaimer

**Operator responsibilities:**
- Comply with local laws
- Terms of service
- Content moderation policy
- Legal hold procedures

## Cryptographic Keys

### Key Hierarchy

```
Master Key (E2E_MASTER_KEY)
├── User Identity Keys (per user)
│   ├── Signed PreKeys (rotated monthly)
│   └── One-Time PreKeys (used once)
├── Group Session Keys (per group)
└── Database Encryption Keys (per collection)
```

### Key Rotation

**Identity Keys**: Never (permanent)
**Signed PreKeys**: Monthly
**One-Time PreKeys**: Used once
**Session Keys**: Per conversation
**JWT Secret**: Yearly (recommended)

### Key Backup

**User keys:**
- Stored client-side
- Optional encrypted backup
- QR code export

**Server keys:**
- Stored in secrets/
- Encrypted backups
- Never in version control

## Security Checklist

### Initial Setup

- [ ] Generate strong secrets
- [ ] Configure TLS/HTTPS
- [ ] Enable log anonymization
- [ ] Set up firewall
- [ ] Configure backups
- [ ] Enable Tor (optional)
- [ ] Review security headers
- [ ] Test disaster recovery

### Regular Maintenance

- [ ] Update dependencies (monthly)
- [ ] Rotate secrets (yearly)
- [ ] Review logs (daily)
- [ ] Test backups (weekly)
- [ ] Security scan (monthly)
- [ ] Update documentation

### Incident Response

1. Identify issue
2. Contain threat
3. Investigate impact
4. Remediate vulnerability
5. Notify affected users
6. Document and learn

## Conclusion

Dis implements strong security measures, but security is a continuous process. Regular updates, monitoring, and following best practices are essential for maintaining a secure platform.

For questions or concerns, contact the security team.
