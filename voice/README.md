# 🎙️ Voice Channels Configuration

## Overview

Dis uses WebRTC for real-time voice communication with the Vortex voice server. This provides encrypted peer-to-peer audio streaming with low latency.

## Features

- **WebRTC-based**: Modern peer-to-peer audio streaming
- **SRTP Encryption**: Secure Real-time Transport Protocol for encrypted audio
- **Low Latency**: Direct peer connections when possible
- **STUN/TURN Support**: NAT traversal for firewall compatibility

## Configuration

### STUN Servers

STUN (Session Traversal Utilities for NAT) helps clients discover their public IP addresses.

Default configuration in `.env`:
```
WEBRTC_STUN_SERVERS=stun:stun.l.google.com:19302
```

You can add multiple STUN servers (comma-separated):
```
WEBRTC_STUN_SERVERS=stun:stun.l.google.com:19302,stun:stun1.l.google.com:19302
```

### TURN Servers (Optional)

TURN (Traversal Using Relays around NAT) relays traffic when direct connections fail.

Format:
```
WEBRTC_TURN_SERVER=turn:username:password@turn.example.com:3478
```

#### Setting up your own TURN server:

1. Install coturn:
```bash
apt-get install coturn
```

2. Configure coturn (`/etc/turnserver.conf`):
```
listening-port=3478
fingerprint
lt-cred-mech
use-auth-secret
static-auth-secret=YOUR_SECURE_SECRET
realm=turn.example.com
total-quota=100
stale-nonce=600
cert=/path/to/cert.pem
pkey=/path/to/key.pem
```

3. Start coturn:
```bash
systemctl start coturn
systemctl enable coturn
```

4. Update `.env`:
```
WEBRTC_TURN_SERVER=turn:username:password@your-domain.com:3478
```

## Vortex Configuration

The Vortex voice server is configured in `docker-compose.yml`:

```yaml
vortex:
  image: ghcr.io/revoltchat/vortex:latest
  environment:
    VORTEX_HOST: 0.0.0.0
    VORTEX_PORT: 9001
    VORTEX_EXTERNAL_HOST: ${VORTEX_EXTERNAL_HOST}
    VORTEX_EXTERNAL_PORT: ${VORTEX_EXTERNAL_PORT}
```

### Port Requirements

- **9001**: Vortex signaling server (WebSocket)
- **443**: Reverse proxy (HTTPS)
- **50000-50100**: UDP range for WebRTC media (optional, configure firewall)

## Security Considerations

### SRTP Encryption

All audio is encrypted using SRTP (Secure Real-time Transport Protocol):
- AES-128 encryption
- HMAC-SHA1 authentication
- Perfect forward secrecy with DTLS

### Network Privacy

- No IP addresses are logged by the voice server
- TURN server credentials are temporary and rotated
- WebRTC uses ephemeral keys for each connection

### Firewall Configuration

For optimal connectivity, allow these ports:

```bash
# UDP for WebRTC media
ufw allow 50000:50100/udp

# TCP for signaling
ufw allow 9001/tcp
```

## Troubleshooting

### Voice not connecting?

1. Check browser console for WebRTC errors
2. Verify STUN server is accessible:
   ```bash
   stunclient stun.l.google.com 19302
   ```
3. Check Vortex logs:
   ```bash
   docker-compose logs vortex
   ```

### Poor audio quality?

1. Check network bandwidth
2. Consider using a TURN server for relay
3. Adjust codec settings (requires Vortex configuration)

### NAT/Firewall issues?

1. Enable TURN server (required for strict NAT)
2. Check UDP ports are not blocked
3. Verify external host/port configuration

## Advanced Configuration

### Custom Codecs

Edit Vortex configuration to prefer specific codecs:
- Opus (default, best quality)
- G.711
- iLBC

### Bandwidth Limits

Set maximum bitrate per connection:
```
VORTEX_MAX_BITRATE=128000
```

### Connection Limits

Limit simultaneous voice connections:
```
VORTEX_MAX_CONNECTIONS=100
```

## Monitoring

View voice server statistics:
```bash
docker-compose logs vortex | grep "stats"
```

Check active connections:
```bash
curl http://localhost:9001/health
```

## Performance Tips

1. **Use UDP**: Ensure UDP is not blocked for best quality
2. **Local TURN**: Run TURN server on same network as Dis
3. **Quality Network**: Voice requires stable ~50kbps per connection
4. **Low Latency**: Keep RTT under 100ms for best experience

## References

- [WebRTC Documentation](https://webrtc.org/)
- [Revolt Vortex](https://github.com/revoltchat/vortex)
- [STUN/TURN Setup](https://webrtc.org/getting-started/turn-server)
- [Coturn Documentation](https://github.com/coturn/coturn)
