// MongoDB Initialization Script for Dis
// Creates collections, indexes, and configures encryption

print('Starting Dis MongoDB initialization...');

// Switch to Revolt database
db = db.getSiblingDB('revolt');

// Create collections with validation
db.createCollection('users', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['_id', 'username', 'password_hash'],
            properties: {
                _id: {
                    bsonType: 'string',
                    description: 'Anonymous user ID'
                },
                username: {
                    bsonType: 'string',
                    description: 'Display username'
                },
                password_hash: {
                    bsonType: 'string',
                    description: 'Argon2 password hash'
                },
                email_hash: {
                    bsonType: ['string', 'null'],
                    description: 'SHA-256 hash of email for lookup'
                },
                encrypted_email: {
                    bsonType: ['string', 'null'],
                    description: 'AES-256-GCM encrypted email'
                },
                identity_key: {
                    bsonType: 'object',
                    description: 'E2E encryption identity key'
                },
                created_at: {
                    bsonType: 'date',
                    description: 'Account creation timestamp'
                },
                last_active: {
                    bsonType: 'date',
                    description: 'Last activity timestamp'
                }
            }
        }
    }
});

db.createCollection('messages', {
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['_id', 'channel_id', 'author_id', 'encrypted_content'],
            properties: {
                _id: {
                    bsonType: 'string'
                },
                channel_id: {
                    bsonType: 'string'
                },
                author_id: {
                    bsonType: 'string'
                },
                encrypted_content: {
                    bsonType: 'string',
                    description: 'E2E encrypted message content'
                },
                timestamp: {
                    bsonType: 'date'
                }
            }
        }
    }
});

db.createCollection('channels');
db.createCollection('servers');
db.createCollection('sessions');
db.createCollection('prekeys');

print('Collections created');

// Create indexes for performance
print('Creating indexes...');

// Users indexes
db.users.createIndex({ '_id': 1 }, { unique: true });
db.users.createIndex({ 'email_hash': 1 }, { sparse: true });
db.users.createIndex({ 'username': 1 });
db.users.createIndex({ 'created_at': 1 });

// Messages indexes
db.messages.createIndex({ 'channel_id': 1, 'timestamp': -1 });
db.messages.createIndex({ 'author_id': 1 });
db.messages.createIndex({ '_id': 1 }, { unique: true });

// Channels indexes
db.channels.createIndex({ '_id': 1 }, { unique: true });
db.channels.createIndex({ 'server_id': 1 });

// Servers indexes
db.servers.createIndex({ '_id': 1 }, { unique: true });
db.servers.createIndex({ 'owner_id': 1 });

// Sessions indexes (with TTL for auto-expiration)
db.sessions.createIndex({ 'expires_at': 1 }, { expireAfterSeconds: 0 });
db.sessions.createIndex({ 'user_id': 1 });

// PreKeys indexes
db.prekeys.createIndex({ 'user_id': 1 });
db.prekeys.createIndex({ 'used': 1 });

print('Indexes created');

// Configure security settings
print('Configuring security...');

// Set profiling level (0 = off, to reduce logging)
db.setProfilingLevel(0);

// Disable command logging for privacy
db.adminCommand({
    setParameter: 1,
    logLevel: 0
});

print('Security configured');

// Create initial admin user (optional)
// db.users.insertOne({
//     _id: 'admin',
//     username: 'admin',
//     password_hash: '$argon2id$v=19$m=65536,t=3,p=4$...',
//     created_at: new Date(),
//     last_active: new Date()
// });

print('Dis MongoDB initialization complete!');
