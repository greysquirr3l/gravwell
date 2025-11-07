---
applyTo: "**/*"
description: "Comprehensive security patterns and standards for enterprise-grade secure development"
references:
  - security_guide: "[CUSTOMIZE: Link to your security documentation]"
  - compliance_docs: "[CUSTOMIZE: Industry compliance requirements]"
---

# Universal Security Standards and Patterns

## Security-First Development Principles

### Defense in Depth

```text
- Multiple layers of security controls
- Fail-safe defaults (deny by default)
- Principle of least privilege
- Security at every architectural layer
- Regular security assessments and updates
```

### Zero Trust Architecture

```text
- Never trust, always verify
- Verify explicitly (identity, device, location)
- Use least privilege access principles
- Assume breach and minimize impact
- Secure all communications end-to-end
```

## Input Validation and Sanitization

### Universal Input Validation Rules

```text
- Validate all inputs at system boundaries
- Use allowlists (whitelist) over blocklists (blacklist)
- Validate data type, length, format, and range
- Sanitize data before processing or storage
- Reject invalid input, don't try to clean it
- Log all validation failures for monitoring
```

### Language-Specific Input Validation

#### JavaScript/TypeScript

```typescript
import { z } from 'zod';

// Schema-based validation
const UserCreateSchema = z.object({
    email: z.string().email().max(255),
    name: z.string().min(1).max(100).regex(/^[a-zA-Z\s]+$/),
    age: z.number().int().min(18).max(120)
});

class InputValidator {
    static validateUserInput(input: unknown): UserCreateRequest {
        try {
            return UserCreateSchema.parse(input);
        } catch (error) {
            if (error instanceof z.ZodError) {
                throw new ValidationError('Invalid input', error.errors);
            }
            throw new SecurityError('Input validation failed');
        }
    }

    // SQL injection prevention
    static sanitizeForQuery(input: string): string {
        return input.replace(/['";\\]/g, '');
    }

    // XSS prevention
    static sanitizeHtml(input: string): string {
        return input
            .replace(/</g, '&lt;')
            .replace(/>/g, '&gt;')
            .replace(/"/g, '&quot;')
            .replace(/'/g, '&#x27;')
            .replace(/\//g, '&#x2F;');
    }
}
```

#### Python

```python
from pydantic import BaseModel, validator, ValidationError
from typing import Optional
import re
import html

class UserCreateRequest(BaseModel):
    email: str
    name: str
    age: int

    @validator('email')
    def validate_email(cls, v):
        email_pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
        if not re.match(email_pattern, v):
            raise ValueError('Invalid email format')
        if len(v) > 255:
            raise ValueError('Email too long')
        return v

    @validator('name')
    def validate_name(cls, v):
        if not re.match(r'^[a-zA-Z\s]+$', v):
            raise ValueError('Name contains invalid characters')
        if len(v) > 100:
            raise ValueError('Name too long')
        return v

    @validator('age')
    def validate_age(cls, v):
        if not 18 <= v <= 120:
            raise ValueError('Age must be between 18 and 120')
        return v

class SecurityValidator:
    @staticmethod
    def sanitize_sql_input(input_str: str) -> str:
        """Sanitize input for SQL queries (use parameterized queries instead)"""
        dangerous_chars = ["'", '"', ';', '--', '/*', '*/']
        for char in dangerous_chars:
            input_str = input_str.replace(char, '')
        return input_str

    @staticmethod
    def sanitize_html(input_str: str) -> str:
        """Sanitize HTML input to prevent XSS"""
        return html.escape(input_str)
```

#### Go

```go
import (
    "fmt"
    "net/mail"
    "regexp"
    "strings"
    "html/template"
)

type UserCreateRequest struct {
    Email string `json:"email"`
    Name  string `json:"name"`
    Age   int    `json:"age"`
}

type ValidationError struct {
    Field   string
    Message string
}

func (e ValidationError) Error() string {
    return fmt.Sprintf("validation failed for %s: %s", e.Field, e.Message)
}

type InputValidator struct {
    emailRegex *regexp.Regexp
    nameRegex  *regexp.Regexp
}

func NewInputValidator() *InputValidator {
    return &InputValidator{
        emailRegex: regexp.MustCompile(`^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$`),
        nameRegex:  regexp.MustCompile(`^[a-zA-Z\s]+$`),
    }
}

func (v *InputValidator) ValidateUserCreate(req UserCreateRequest) error {
    if _, err := mail.ParseAddress(req.Email); err != nil {
        return ValidationError{Field: "email", Message: "invalid email format"}
    }
    if len(req.Email) > 255 {
        return ValidationError{Field: "email", Message: "email too long"}
    }

    if !v.nameRegex.MatchString(req.Name) {
        return ValidationError{Field: "name", Message: "name contains invalid characters"}
    }
    if len(req.Name) > 100 {
        return ValidationError{Field: "name", Message: "name too long"}
    }

    if req.Age < 18 || req.Age > 120 {
        return ValidationError{Field: "age", Message: "age must be between 18 and 120"}
    }

    return nil
}

// HTML sanitization
func SanitizeHTML(input string) string {
    return template.HTMLEscapeString(input)
}

// SQL input cleaning (use parameterized queries instead)
func CleanSQLInput(input string) string {
    dangerous := []string{"'", "\"", ";", "--", "/*", "*/"}
    cleaned := input
    for _, char := range dangerous {
        cleaned = strings.ReplaceAll(cleaned, char, "")
    }
    return cleaned
}
```

#### Rust

```rust
use regex::Regex;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use validator::{Validate, ValidationError as ValidatorError};

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Validation failed: {message}")]
    Validation { message: String },

    #[error("Authentication failed")]
    Authentication,

    #[error("Authorization denied")]
    Authorization,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email, length(max = 255))]
    pub email: String,

    #[validate(length(min = 1, max = 100), regex = "NAME_REGEX")]
    pub name: String,

    #[validate(range(min = 18, max = 120))]
    pub age: i32,
}

lazy_static::lazy_static! {
    static ref NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z\s]+$").unwrap();
    static ref DANGEROUS_CHARS: Regex = Regex::new(r"[<>&\"']").unwrap();
}

pub fn validate_and_sanitize_input(req: &CreateUserRequest) -> Result<CreateUserRequest, SecurityError> {
    // Validate using validator crate
    req.validate().map_err(|e| SecurityError::Validation {
        message: format!("Input validation failed: {}", e),
    })?;

    // Additional manual validation
    if req.email.len() > 255 {
        return Err(SecurityError::Validation {
            message: "Email too long".to_string(),
        });
    }

    // Create sanitized version
    Ok(CreateUserRequest {
        email: sanitize_string(&req.email),
        name: sanitize_string(&req.name),
        age: req.age,
    })
}

pub fn sanitize_string(input: &str) -> String {
    // HTML encode dangerous characters
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&#x27;")
}

// Secure password hashing using argon2
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};

pub fn hash_password(password: &str) -> Result<String, SecurityError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| SecurityError::Validation {
            message: "Password hashing failed".to_string(),
        })
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, SecurityError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|_| SecurityError::Validation {
            message: "Invalid password hash".to_string(),
        })?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}
```

## Authentication and Authorization

### Authentication Patterns

#### JWT Token Management

```text
- Use strong signing algorithms (RS256, ES256)
- Implement proper token expiration (15-30 minutes for access tokens)
- Use refresh tokens with longer expiration (7-30 days)
- Store tokens securely (httpOnly cookies for web, secure storage for mobile)
- Implement token revocation mechanisms
- Validate token signature and claims on every request
```

#### Multi-Factor Authentication (MFA)

```text
- Require MFA for sensitive operations
- Support multiple MFA methods (TOTP, SMS, hardware keys)
- Implement backup recovery codes
- Use time-based one-time passwords (TOTP) over SMS when possible
- Implement rate limiting for MFA attempts
```

### Authorization Patterns

#### Role-Based Access Control (RBAC)

```typescript
// TypeScript RBAC implementation
interface User {
    id: string;
    roles: Role[];
}

interface Role {
    name: string;
    permissions: Permission[];
}

interface Permission {
    resource: string;
    action: string;
}

class AuthorizationService {
    hasPermission(user: User, resource: string, action: string): boolean {
        return user.roles.some(role =>
            role.permissions.some(permission =>
                permission.resource === resource &&
                permission.action === action
            )
        );
    }

    requirePermission(user: User, resource: string, action: string): void {
        if (!this.hasPermission(user, resource, action)) {
            throw new UnauthorizedError(
                `User ${user.id} lacks permission for ${action} on ${resource}`
            );
        }
    }
}

// Usage in API endpoints
app.post('/api/users', requireAuth, async (req, res) => {
    const user = req.user;
    authService.requirePermission(user, 'users', 'create');

    // Proceed with user creation
});
```

#### Attribute-Based Access Control (ABAC)

```python
from typing import Dict, Any
from dataclasses import dataclass

@dataclass
class PolicyContext:
    user: Dict[str, Any]
    resource: Dict[str, Any]
    environment: Dict[str, Any]
    action: str

class ABACEngine:
    def __init__(self):
        self.policies = []

    def evaluate(self, context: PolicyContext) -> bool:
        """Evaluate all policies against the context"""
        for policy in self.policies:
            if not policy.evaluate(context):
                return False
        return True

    def add_policy(self, policy):
        self.policies.append(policy)

class OwnershipPolicy:
    def evaluate(self, context: PolicyContext) -> bool:
        if context.action in ['update', 'delete']:
            return context.resource.get('owner_id') == context.user.get('id')
        return True

class BusinessHoursPolicy:
    def evaluate(self, context: PolicyContext) -> bool:
        if context.action == 'sensitive_operation':
            hour = context.environment.get('current_hour', 0)
            return 9 <= hour <= 17
        return True
```

## Secure Data Handling

### Encryption Patterns

#### Data at Rest

```go
package encryption

import (
    "crypto/aes"
    "crypto/cipher"
    "crypto/rand"
    "fmt"
    "io"
)

type EncryptionService struct {
    gcm cipher.AEAD
}

func NewEncryptionService(key []byte) (*EncryptionService, error) {
    block, err := aes.NewCipher(key)
    if err != nil {
        return nil, fmt.Errorf("failed to create cipher: %w", err)
    }

    gcm, err := cipher.NewGCM(block)
    if err != nil {
        return nil, fmt.Errorf("failed to create GCM: %w", err)
    }

    return &EncryptionService{gcm: gcm}, nil
}

func (e *EncryptionService) Encrypt(plaintext []byte) ([]byte, error) {
    nonce := make([]byte, e.gcm.NonceSize())
    if _, err := io.ReadFull(rand.Reader, nonce); err != nil {
        return nil, fmt.Errorf("failed to generate nonce: %w", err)
    }

    ciphertext := e.gcm.Seal(nonce, nonce, plaintext, nil)
    return ciphertext, nil
}

func (e *EncryptionService) Decrypt(ciphertext []byte) ([]byte, error) {
    nonceSize := e.gcm.NonceSize()
    if len(ciphertext) < nonceSize {
        return nil, fmt.Errorf("ciphertext too short")
    }

    nonce, ciphertext := ciphertext[:nonceSize], ciphertext[nonceSize:]
    plaintext, err := e.gcm.Open(nil, nonce, ciphertext, nil)
    if err != nil {
        return nil, fmt.Errorf("failed to decrypt: %w", err)
    }

    return plaintext, nil
}
```

#### Data in Transit

```javascript
// TLS configuration for Node.js
import https from 'https';
import fs from 'fs';

const tlsOptions = {
    key: fs.readFileSync('private-key.pem'),
    cert: fs.readFileSync('certificate.pem'),
    // Enforce strong TLS versions
    secureProtocol: 'TLSv1_2_method',
    // Strong cipher suites
    ciphers: [
        'ECDHE-RSA-AES128-GCM-SHA256',
        'ECDHE-RSA-AES256-GCM-SHA384',
        'ECDHE-RSA-AES128-SHA256',
        'ECDHE-RSA-AES256-SHA384'
    ].join(':'),
    honorCipherOrder: true
};

const server = https.createServer(tlsOptions, app);
```

### Password Security

#### Password Hashing

```python
import bcrypt
import secrets
from typing import str

class PasswordSecurity:
    def __init__(self, rounds: int = 12):
        self.rounds = rounds

    def hash_password(self, password: str) -> str:
        """Hash a password using bcrypt with salt"""
        salt = bcrypt.gensalt(rounds=self.rounds)
        hashed = bcrypt.hashpw(password.encode('utf-8'), salt)
        return hashed.decode('utf-8')

    def verify_password(self, password: str, hashed: str) -> bool:
        """Verify a password against its hash"""
        try:
            return bcrypt.checkpw(password.encode('utf-8'), hashed.encode('utf-8'))
        except Exception:
            return False

    def generate_secure_token(self, length: int = 32) -> str:
        """Generate a cryptographically secure random token"""
        return secrets.token_urlsafe(length)

    @staticmethod
    def check_password_strength(password: str) -> tuple[bool, list[str]]:
        """Check password strength and return issues"""
        issues = []

        if len(password) < 8:
            issues.append("Password must be at least 8 characters long")

        if not any(c.isupper() for c in password):
            issues.append("Password must contain at least one uppercase letter")

        if not any(c.islower() for c in password):
            issues.append("Password must contain at least one lowercase letter")

        if not any(c.isdigit() for c in password):
            issues.append("Password must contain at least one digit")

        if not any(c in "!@#$%^&*()_+-=[]{}|;:,.<>?" for c in password):
            issues.append("Password must contain at least one special character")

        return len(issues) == 0, issues
```

## API Security Patterns

### Rate Limiting

```typescript
import { RateLimiterRedis } from 'rate-limiter-flexible';
import Redis from 'ioredis';

const redis = new Redis({
    host: process.env.REDIS_HOST,
    port: parseInt(process.env.REDIS_PORT || '6379'),
});

const rateLimiters = {
    auth: new RateLimiterRedis({
        storeClient: redis,
        keyPrefixFromEnv: true,
        points: 5, // Number of attempts
        duration: 900, // Per 15 minutes
        blockDuration: 900, // Block for 15 minutes
    }),
    api: new RateLimiterRedis({
        storeClient: redis,
        keyPrefixFromEnv: true,
        points: 100, // Number of requests
        duration: 60, // Per minute
        blockDuration: 60, // Block for 1 minute
    }),
};

export async function rateLimitMiddleware(
    req: Request,
    res: Response,
    next: NextFunction,
    type: 'auth' | 'api' = 'api'
) {
    try {
        const identifier = req.ip || req.socket.remoteAddress;
        await rateLimiters[type].consume(identifier);
        next();
    } catch (rateLimiterRes) {
        const remainingPoints = rateLimiterRes.remainingPoints || 0;
        const msBeforeNext = rateLimiterRes.msBeforeNext || 0;

        res.set({
            'X-RateLimit-Limit': rateLimiters[type].points,
            'X-RateLimit-Remaining': remainingPoints,
            'X-RateLimit-Reset': Math.round(msBeforeNext / 1000),
        });

        res.status(429).json({
            error: 'Too many requests',
            retryAfter: Math.round(msBeforeNext / 1000),
        });
    }
}
```

### CORS Configuration

```javascript
const corsOptions = {
    origin: function (origin, callback) {
        const allowedOrigins = process.env.ALLOWED_ORIGINS?.split(',') || [];

        // Allow requests with no origin (mobile apps, etc.)
        if (!origin) return callback(null, true);

        if (allowedOrigins.includes(origin)) {
            callback(null, true);
        } else {
            callback(new Error('Not allowed by CORS'));
        }
    },
    credentials: true, // Allow cookies
    methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
    allowedHeaders: [
        'Content-Type',
        'Authorization',
        'X-Requested-With',
        'X-CSRF-Token'
    ],
    exposedHeaders: ['X-RateLimit-Remaining', 'X-RateLimit-Reset'],
    maxAge: 86400 // 24 hours
};
```

## Security Logging and Monitoring

### Security Event Logging

```python
import logging
from enum import Enum
from dataclasses import dataclass
from typing import Optional, Dict, Any
import json

class SecurityEventType(Enum):
    LOGIN_SUCCESS = "login_success"
    LOGIN_FAILURE = "login_failure"
    UNAUTHORIZED_ACCESS = "unauthorized_access"
    PRIVILEGE_ESCALATION = "privilege_escalation"
    DATA_ACCESS = "data_access"
    DATA_MODIFICATION = "data_modification"
    SUSPICIOUS_ACTIVITY = "suspicious_activity"

@dataclass
class SecurityEvent:
    event_type: SecurityEventType
    user_id: Optional[str]
    ip_address: str
    user_agent: str
    resource: Optional[str]
    action: Optional[str]
    success: bool
    details: Dict[str, Any]
    timestamp: float

class SecurityLogger:
    def __init__(self):
        self.logger = logging.getLogger('security')
        handler = logging.StreamHandler()
        formatter = logging.Formatter(
            '%(asctime)s - SECURITY - %(levelname)s - %(message)s'
        )
        handler.setFormatter(formatter)
        self.logger.addHandler(handler)
        self.logger.setLevel(logging.INFO)

    def log_event(self, event: SecurityEvent) -> None:
        log_data = {
            'event_type': event.event_type.value,
            'user_id': event.user_id,
            'ip_address': event.ip_address,
            'user_agent': event.user_agent,
            'resource': event.resource,
            'action': event.action,
            'success': event.success,
            'details': event.details,
            'timestamp': event.timestamp
        }

        if event.success:
            self.logger.info(json.dumps(log_data))
        else:
            self.logger.warning(json.dumps(log_data))

    def log_login_attempt(self, user_id: str, ip_address: str,
                         user_agent: str, success: bool,
                         details: Dict[str, Any] = None) -> None:
        event = SecurityEvent(
            event_type=SecurityEventType.LOGIN_SUCCESS if success else SecurityEventType.LOGIN_FAILURE,
            user_id=user_id,
            ip_address=ip_address,
            user_agent=user_agent,
            resource=None,
            action='login',
            success=success,
            details=details or {},
            timestamp=time.time()
        )
        self.log_event(event)
```

## Secure Configuration Management

### Environment Configuration

```bash
# .env.example - Template for environment variables
# Copy to .env and fill in actual values

# Database
DATABASE_URL="postgresql://username:password@localhost:5432/dbname"
DATABASE_SSL_MODE="require"
DATABASE_MAX_CONNECTIONS="20"

# Redis
REDIS_URL="redis://localhost:6379"
REDIS_PASSWORD=""

# JWT Configuration
JWT_SECRET="[GENERATE_STRONG_SECRET_HERE]"
JWT_ACCESS_TOKEN_EXPIRES="15m"
JWT_REFRESH_TOKEN_EXPIRES="7d"

# Encryption
ENCRYPTION_KEY="[GENERATE_32_BYTE_KEY_HERE]"

# API Configuration
API_RATE_LIMIT_REQUESTS="100"
API_RATE_LIMIT_WINDOW="60"
ALLOWED_ORIGINS="https://yourdomain.com,https://www.yourdomain.com"

# Logging
LOG_LEVEL="info"
LOG_FORMAT="json"

# Security
BCRYPT_ROUNDS="12"
SESSION_SECRET="[GENERATE_STRONG_SECRET_HERE]"
CSRF_SECRET="[GENERATE_STRONG_SECRET_HERE]"

# External Services
EXTERNAL_API_KEY="[YOUR_API_KEY_HERE]"
EXTERNAL_API_URL="https://api.external-service.com"

# Monitoring
SENTRY_DSN="[YOUR_SENTRY_DSN_HERE]"
METRICS_ENDPOINT="https://your-metrics-service.com"
```

### Configuration Validation

```go
package config

import (
    "fmt"
    "os"
    "strconv"
    "time"
)

type Config struct {
    Database DatabaseConfig
    Redis    RedisConfig
    JWT      JWTConfig
    API      APIConfig
    Security SecurityConfig
}

type DatabaseConfig struct {
    URL            string
    SSLMode        string
    MaxConnections int
}

type JWTConfig struct {
    Secret                string
    AccessTokenExpires    time.Duration
    RefreshTokenExpires   time.Duration
}

type SecurityConfig struct {
    BcryptRounds    int
    EncryptionKey   []byte
    AllowedOrigins  []string
}

func LoadConfig() (*Config, error) {
    config := &Config{}

    // Database configuration
    config.Database.URL = getEnvRequired("DATABASE_URL")
    config.Database.SSLMode = getEnvWithDefault("DATABASE_SSL_MODE", "require")
    maxConn, err := strconv.Atoi(getEnvWithDefault("DATABASE_MAX_CONNECTIONS", "20"))
    if err != nil {
        return nil, fmt.Errorf("invalid DATABASE_MAX_CONNECTIONS: %w", err)
    }
    config.Database.MaxConnections = maxConn

    // JWT configuration
    config.JWT.Secret = getEnvRequired("JWT_SECRET")
    if len(config.JWT.Secret) < 32 {
        return nil, fmt.Errorf("JWT_SECRET must be at least 32 characters")
    }

    accessExpires, err := time.ParseDuration(getEnvWithDefault("JWT_ACCESS_TOKEN_EXPIRES", "15m"))
    if err != nil {
        return nil, fmt.Errorf("invalid JWT_ACCESS_TOKEN_EXPIRES: %w", err)
    }
    config.JWT.AccessTokenExpires = accessExpires

    // Security configuration
    rounds, err := strconv.Atoi(getEnvWithDefault("BCRYPT_ROUNDS", "12"))
    if err != nil || rounds < 10 || rounds > 15 {
        return nil, fmt.Errorf("BCRYPT_ROUNDS must be between 10 and 15")
    }
    config.Security.BcryptRounds = rounds

    encryptionKey := getEnvRequired("ENCRYPTION_KEY")
    if len(encryptionKey) != 32 {
        return nil, fmt.Errorf("ENCRYPTION_KEY must be exactly 32 bytes")
    }
    config.Security.EncryptionKey = []byte(encryptionKey)

    return config, nil
}

func getEnvRequired(key string) string {
    value := os.Getenv(key)
    if value == "" {
        panic(fmt.Sprintf("required environment variable %s not set", key))
    }
    return value
}

func getEnvWithDefault(key, defaultValue string) string {
    value := os.Getenv(key)
    if value == "" {
        return defaultValue
    }
    return value
}
```

---

## Security Checklist for AI Code Generation

When generating code, ensure:

### Input Handling

- [ ] All external inputs are validated and sanitized
- [ ] Parameterized queries are used for database operations
- [ ] File uploads are properly validated (type, size, content)
- [ ] Rate limiting is implemented for public endpoints

### Authentication & Authorization

- [ ] Strong authentication mechanisms are in place
- [ ] Authorization checks are performed for all protected resources
- [ ] JWT tokens use strong algorithms and appropriate expiration
- [ ] Session management follows security best practices

### Data Protection

- [ ] Sensitive data is encrypted at rest and in transit
- [ ] Passwords are properly hashed with salt
- [ ] Secrets are managed through environment variables or secret managers
- [ ] Personal data handling complies with privacy regulations

### Error Handling

- [ ] Error messages don't leak sensitive information
- [ ] All errors are properly logged for security monitoring
- [ ] Failed authentication attempts are rate-limited
- [ ] Stack traces are not exposed in production

### Security Headers & Configuration

- [ ] Appropriate security headers are set (CSP, HSTS, etc.)
- [ ] CORS is properly configured with specific origins
- [ ] TLS/HTTPS is enforced for all communications
- [ ] Default configurations are secure

**This security instruction file ensures all AI-generated code follows enterprise-grade security practices across any technology stack.**
