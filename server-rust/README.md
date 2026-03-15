# AgentTrust Server (Rust/Axum) — Phase 2

High-performance API server rewritten from Python/FastAPI to Rust/Axum.
100% backward compatible with Phase 1 SDK (Python + TypeScript + MCP).

## Requirements

- Rust 1.77+
- PostgreSQL 16
- Redis 7

## Quick Start

```bash
# Start PostgreSQL and Redis
cd ../docker
docker-compose up postgres redis -d

# Copy and configure environment
cp .env.example .env
# Edit .env: set STRIPE_SECRET_KEY, DATABASE_URL, REDIS_URL

# Run server (migrations applied automatically)
cargo run
```

## Docker (full stack)

```bash
cd ../docker
STRIPE_SECRET_KEY=sk_test_... docker-compose up
```

## Environment Variables

| Variable | Required | Default | Description |
|---|---|---|---|
| `DATABASE_URL` | ✅ | — | PostgreSQL connection string |
| `REDIS_URL` | — | `redis://localhost:6379` | Redis connection string |
| `STRIPE_SECRET_KEY` | ✅ | — | Stripe API key |
| `JWT_SERVER_PRIVATE_KEY` | — | auto-generated | Base64 Ed25519 private key (32 bytes) |
| `JWT_SERVER_PUBLIC_KEY` | — | auto-generated | Base64 Ed25519 public key (32 bytes) |
| `PORT` | — | `8000` | HTTP listen port |
| `APPROVAL_REQUIRED_ABOVE` | — | `30000` | JPY threshold for Human-in-the-Loop approval |

## API Endpoints

### MVP Compatible (Phase 1 → Phase 2, no changes needed)
- `POST /did/create`
- `GET /did/resolve/{did}`
- `POST /did/verify`
- `POST /auth/token`
- `POST /auth/verify-token`
- `POST /payment/execute`
- `GET /payment/{transaction_id}`
- `GET /audit/{agent_did}`
- `POST /audit/verify`

### Phase 2 New Endpoints
- `POST /oauth/authorize` — OAuth 2.0 Authorization Code Grant
- `POST /oauth/token` — Token issuance (4 grant types)
- `POST /oauth/revoke` — Token revocation
- `GET /oauth/jwks` — JWK Set for JWT verification
- `POST /oauth/register` — Register OAuth client
- `POST /approval/request` — Request Human-in-the-Loop approval
- `POST /approval/{id}/approve` — Approve a pending transaction
- `POST /approval/{id}/reject` — Reject a pending transaction
- `POST /payment/refund` — Refund a payment
- `GET /payment/methods` — List available payment methods
- `GET /health` — Health check

## Testing

```bash
# Unit tests (no database needed)
cargo test test_crypto test_hashing test_circuit_breaker

# Integration tests (requires running server)
TEST_SERVER_URL=http://localhost:8000 cargo test test_did test_auth

# All tests
cargo test
```

## Architecture

```
src/
├── main.rs          — Server startup, router assembly
├── config.rs        — Environment configuration
├── error.rs         — Unified AppError type
├── state.rs         — Shared application state (DB, Redis, circuit breaker)
├── crypto/          — Ed25519, SHA-256, JWT (no external dependencies)
├── models/          — SQLx row types (PostgreSQL)
├── services/        — Business logic (DID, Auth, OAuth, Payment, Audit, Approval)
├── routes/          — Axum HTTP handlers
├── middleware/       — JWT auth, rate limiting, circuit breaker
└── payment_providers/ — Stripe and PayPay (trait-based abstraction)
```
