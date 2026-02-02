# 👤 Anonymity - Dis

## Overview

Dis is designed from the ground up to protect user anonymity. This document explains what anonymity guarantees Dis provides, how they are achieved, and their limitations.

## Anonymity Guarantees

### What Dis Protects

✅ **Identity Anonymity**: No real name required
✅ **Registration Anonymity**: No email/phone required
✅ **Connection Anonymity**: Tor support
✅ **Metadata Privacy**: Minimal logging
✅ **Content Privacy**: E2E encryption
✅ **Payment Anonymity**: No payment required (self-hosted)

### What Dis Does NOT Protect

❌ **Browser fingerprinting**: Use Tor Browser
❌ **Timing attacks**: Use Tor for timing protection
❌ **Traffic analysis**: Tor provides partial protection
❌ **Social graphs**: Contact patterns still visible
❌ **User behavior**: Writing style, active hours
❌ **Voluntary disclosure**: Users revealing identity

## Anonymous Registration

### No Personal Information Required

Registration process:
```
1. Choose username (can be pseudonymous)
2. Generate anonymous ID: "anon_[UUID]"
3. Create password (never transmitted plaintext)
4. Optional: Encrypted email for recovery
5. Complete PoW CAPTCHA (client-side)
```

**No tracking data collected:**
- ❌ Real name
- ❌ Email address (unless optional)
- ❌ Phone number
- ❌ Payment information
- ❌ Government ID
- ❌ Social media accounts

### Identity Keys

Each user gets cryptographic identity:
- Generated client-side
- Never linked to real identity
- Can be rotated
- Multiple identities supported

## Network Anonymity

### Tor Integration

**Hidden Service**: Dis provides .onion address

**Benefits:**
- No IP address exposure
- Censorship resistance
- Location anonymity
- ISP-level privacy

**Access:**
```
http://[random].onion
```

**Setup:**
```bash
# Enable in .env
TOR_ENABLED=true

# Get .onion address
docker-compose logs tor | grep "onion"
```

### Tor Browser Recommended

For maximum anonymity:
1. Download Tor Browser
2. Access via .onion address
3. Don't login to clearnet accounts
4. Don't reveal personal information

### Connection Privacy

**What's hidden:**
- Source IP address (with Tor)
- Geolocation
- ISP information
- Network path

**What's visible:**
- You're accessing Dis (traffic analysis)
- Approximate timing of access
- Amount of data transferred

## Metadata Privacy

### Minimal Data Collection

**What we store:**
```
User Profile:
- Anonymous user ID
- Password hash (Argon2)
- Identity key (E2E encryption)
- Last active timestamp
- Encrypted email (optional)

Messages:
- Encrypted content
- Timestamp
- Channel ID
- Sender ID (anonymous)
```

**What we DON'T store:**
```
- IP addresses
- User-Agent strings
- Geolocation
- Device information
- Browser fingerprints
- Unencrypted message content
```

### Log Anonymization

All logs automatically anonymized:

**IP Addresses:**
```
Before: 192.168.1.100
After:  IP_HASH_a1b2c3d4
```

**Emails:**
```
Before: user@example.com
After:  EMAIL_HASH_x7y8z9
```

**Tokens:**
```
Before: eyJhbGciOiJIUzI1NiIs...
After:  TOKEN_REDACTED
```

**Implementation:** `logs/anonymizer.py`

**Automatic:**
- Real-time anonymization
- Regex-based pattern matching
- Consistent hashing
- No raw data retained

### Log Retention

**Default retention**: 7 days

**Automatic deletion:**
```python
# After 7 days, logs are:
1. Automatically deleted
2. Securely overwritten
3. No backups retained
```

**Configuration:**
```bash
# In .env
LOG_RETENTION_DAYS=7
```

## Communication Anonymity

### End-to-End Encryption

Messages encrypted before leaving device:

```
Alice Device:
1. Write message
2. Encrypt with Bob's key
3. Send ciphertext

Dis Server:
- Stores encrypted blob
- Cannot read content
- No decryption keys

Bob Device:
1. Receive ciphertext
2. Decrypt with Bob's key
3. Read message
```

**Server never sees:**
- Message plaintext
- Attachments (also encrypted)
- Reactions (encrypted)
- User status (encrypted)

### Metadata Leakage

**What metadata is visible to server:**

✅ **User exists**: Anonymous ID visible
✅ **User online**: Last active timestamp
✅ **Message sent**: Sender and recipient IDs
✅ **Message timing**: When message sent
✅ **Message size**: Approximate length (encrypted)
✅ **Channel membership**: Who's in what channel

❌ **Message content**: Always encrypted
❌ **Real identity**: Only anonymous IDs
❌ **Location**: Not logged (IP anonymized)

### Protecting Metadata

**Recommendations:**

1. **Use Tor**: Hides connection metadata
2. **Random delays**: Don't respond immediately
3. **Dummy traffic**: Future feature
4. **Group chats**: Hides who talks to whom
5. **Ephemeral messages**: Delete after read

## Anonymity vs. Pseudonymity

### Anonymous Mode (Default)

- No persistent identity required
- Create throwaway accounts
- No reputation system
- Fresh start anytime

### Pseudonymous Mode (Optional)

- Consistent identity across sessions
- Build reputation
- Persistent username
- Identity verification (optional)

**Users can choose based on needs.**

## Threat Models

### Against Server Operator

**Assumptions:**
- Operator is honest but curious
- Operator might be compromised
- Operator might be compelled to log

**Protections:**
- E2E encryption (operator can't read)
- Minimal metadata (little to log)
- Tor access (hide IP from operator)
- Anonymous IDs (no real identity)

**Limits:**
- Operator sees who talks to whom
- Operator sees timing information
- Operator sees user is online

### Against Network Adversary

**Assumptions:**
- ISP can see all traffic
- Government can monitor network
- Adversary can do traffic analysis

**Protections:**
- TLS encryption (content hidden)
- Tor hidden service (IP hidden)
- No identifying information transmitted

**Limits:**
- Can see you're using Dis
- Can do timing correlation
- Can block Tor (use bridges)

### Against Global Passive Adversary

**Assumptions:**
- Adversary can see all Internet traffic
- Can correlate timing and size
- Has vast computational resources

**Protections:**
- Tor provides some protection
- Timing obfuscation (future)
- Cover traffic (future)

**Limits:**
- Timing attacks still possible
- Traffic correlation possible
- Very resourced adversary

### Against Legal Compulsion

**Assumptions:**
- Court order for data
- Subpoena for logs
- National security letter

**Protections:**
- E2E encryption (no keys to provide)
- Minimal logging (little to provide)
- Anonymous IDs (no real identity)
- Self-hosted (you control data)

**Limits:**
- Must comply with lawful orders
- Metadata still exists
- Warrant canary (consider)

## Best Practices for Anonymity

### For Maximum Anonymity

1. **Access via Tor**
   ```
   - Use Tor Browser
   - Access .onion address
   - Don't use VPN + Tor
   ```

2. **Create New Identity**
   ```
   - Don't reuse usernames
   - Fresh account per context
   - No identifying information
   ```

3. **Secure Operation**
   ```
   - Use Tails or Whonix
   - Dedicated device
   - No personal accounts logged in
   ```

4. **Communication Practices**
   ```
   - Random delays between messages
   - Don't reveal timezone
   - Avoid unique writing style
   - Don't share photos with metadata
   ```

5. **Don't Mix Identities**
   ```
   - Separate contexts
   - Different pseudonyms
   - Don't cross-reference
   ```

### For Moderate Anonymity

1. **Basic Protection**
   ```
   - Access via HTTPS
   - Use anonymous registration
   - Don't share personal info
   ```

2. **Pseudonymous Identity**
   ```
   - Consistent username okay
   - Build reputation
   - Still no real identity
   ```

3. **Optional Email**
   ```
   - Use anonymous email
   - ProtonMail, Tutanota
   - Or disposable address
   ```

### Common Mistakes

❌ **Using real email**: Use anonymous email service
❌ **Reusing passwords**: Use unique password per service
❌ **Revealing location**: Don't mention where you are
❌ **Posting identifying photos**: Remove EXIF data
❌ **Unique writing style**: Be aware of style analysis
❌ **Revealing timezone**: Avoid time-specific references
❌ **Cross-referencing**: Don't link to personal accounts
❌ **Browser fingerprinting**: Use Tor Browser

## Anonymity Limitations

### Technical Limitations

1. **Timing Correlation**
   - Attacker can correlate message timing
   - Mitigation: Random delays

2. **Traffic Analysis**
   - Pattern of communication visible
   - Mitigation: Tor, dummy traffic

3. **Browser Fingerprinting**
   - Unique browser configuration
   - Mitigation: Tor Browser

4. **Social Graph Analysis**
   - Who talks to whom visible
   - Mitigation: Group chats

### Operational Limitations

1. **User Error**
   - Accidentally revealing information
   - Mitigation: Education, warnings

2. **Software Bugs**
   - Bugs may leak information
   - Mitigation: Regular audits, updates

3. **Side Channels**
   - Timing, power consumption
   - Mitigation: Constant-time operations

### Legal Limitations

1. **Lawful Intercept**
   - Court orders
   - Mitigation: E2E encryption, minimal data

2. **Jurisdiction**
   - Different laws in different countries
   - Mitigation: Self-hosted, choose jurisdiction

## Comparison with Other Platforms

### Dis vs. Signal

| Feature | Dis | Signal |
|---------|-----|--------|
| E2E Encryption | ✅ | ✅ |
| Phone Required | ❌ | ✅ |
| Anonymous Registration | ✅ | ❌ |
| Self-Hosted | ✅ | ❌ |
| Tor Support | ✅ | Limited |
| Metadata | Minimal | Phone numbers |

### Dis vs. Discord

| Feature | Dis | Discord |
|---------|-----|--------|
| E2E Encryption | ✅ | ❌ |
| Email Required | ❌ | ✅ |
| Anonymous | ✅ | ❌ |
| Self-Hosted | ✅ | ❌ |
| Privacy | High | Low |

### Dis vs. Matrix

| Feature | Dis | Matrix |
|---------|-----|--------|
| E2E Encryption | ✅ | ✅ |
| Anonymous | ✅ | Partial |
| Self-Hosted | ✅ | ✅ |
| Metadata | Minimal | More |
| Tor | Native | Possible |

## Anonymity Checklist

### Setup

- [ ] Enable Tor hidden service
- [ ] Configure log anonymization
- [ ] Set short log retention
- [ ] Disable unnecessary logging
- [ ] Use HTTPS with HSTS
- [ ] Configure security headers

### Usage

- [ ] Access via Tor Browser
- [ ] Use anonymous registration
- [ ] Don't share personal info
- [ ] Random delays between messages
- [ ] Remove photo metadata
- [ ] Don't link to real identity

### Verification

- [ ] Check .onion works
- [ ] Verify logs are anonymized
- [ ] Test with Tor Browser
- [ ] Confirm no IP logging
- [ ] Review metadata exposure
- [ ] Test key verification

## Future Improvements

### Planned Features

1. **Dummy Traffic**: Random messages to hide patterns
2. **Timing Obfuscation**: Random delays built-in
3. **Mix Networks**: Message routing for metadata privacy
4. **Decentralization**: Federated servers
5. **Anonymous Credentials**: Zero-knowledge proofs
6. **Post-Quantum**: Quantum-resistant encryption

### Research Areas

- Advanced traffic analysis resistance
- Better metadata protection
- Social graph obfuscation
- Anonymous authentication

## Conclusion

Dis provides strong anonymity guarantees, but perfect anonymity is impossible. Users must understand the limitations and follow best practices. Use Tor, avoid revealing information, and be aware of metadata.

**Key Takeaway**: Dis makes anonymity easy, but can't protect against user mistakes. Think before you share.

## Resources

- [Tor Project](https://www.torproject.org/)
- [EFF Surveillance Self-Defense](https://ssd.eff.org/)
- [PRISM Break](https://prism-break.org/)
- [Whonix](https://www.whonix.org/)
- [Tails](https://tails.boum.org/)

## Disclaimer

While Dis implements strong privacy measures, no system is perfect. Users are responsible for their own operational security. Always assess your threat model and act accordingly.
