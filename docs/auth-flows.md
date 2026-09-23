# Identity flows

## Registration

```mermaid
sequenceDiagram
    actor U as User
    participant A as Mobile app
    participant I as Identity API
    participant DB as PostgreSQL
    U->>A: Phone, password, question + answer
    A->>I: POST /auth/register over HTTPS
    I->>I: Normalize phone; Argon2id hashes
    I->>DB: Create user + trusted first device
    DB-->>I: Commit
    I-->>A: Access + rotating refresh token
```

## Password login and unfamiliar device

```mermaid
sequenceDiagram
    actor U as User
    participant A as Mobile app
    participant I as Identity API
    participant DB as PostgreSQL
    U->>A: Phone + password
    A->>I: POST /auth/login
    I->>DB: Verify account and device ID
    alt Trusted device
        I-->>A: Tokens
    else Unfamiliar device
        I-->>A: Challenge ID + security question
        U->>A: Security answer
        A->>I: POST /auth/device/verify
        I->>DB: Mark one-time challenge used; trust device
        I-->>A: Tokens
    end
```

## Forgotten and authenticated password change

OTP is only used for recovery. Request always returns `accepted` to prevent phone enumeration. The production OTP adapter must rate-limit by phone/device/IP, cap verification attempts, bind the challenge to a purpose, and send via a provider without logging plaintext OTPs.

Authenticated password change requires the current password. Password reset revokes all refresh tokens. Access-token revocation is bounded by its 15-minute lifetime; high-risk deployments should also check a session epoch.

