# AgentTrust Protocol

Secure payment infrastructure for AI agents using DID-based identity and Ed25519 cryptography.

## Overview

AgentTrust Protocol provides:
- **DID Identity**: W3C DID Core compliant identities using Ed25519 keypairs (`did:key` method)
- **JWT Authentication**: Scoped access tokens signed with Ed25519
- **Payment Execution**: Stripe-backed payments with amount limits enforced by JWT scopes
- **Audit Hash Chain**: SHA-256 hash chain for tamper-evident transaction logging
- **SDK**: AgentWallet class and LangChain tool integration

## Quick Start

### Install dependencies

```bash
pip install -e ".[dev]"
```

### Configure environment

```bash
cp .env.example .env
# Edit .env with your Stripe test key
```

### Run the server

```bash
uvicorn server.main:app --reload
```

### Run tests

```bash
pytest tests/
```

### Run demo

```bash
python demo.py
```

## Project Structure

```
agenttrust/
├── server/          # FastAPI backend
│   ├── routers/     # API endpoints (DID, Auth, Payment, Audit)
│   ├── services/    # Business logic
│   ├── models/      # SQLAlchemy ORM models
│   ├── schemas/     # Pydantic request/response schemas
│   └── crypto/      # Ed25519 and SHA-256 primitives
├── sdk/             # Agent developer SDK
│   ├── wallet.py    # AgentWallet class
│   ├── tools.py     # LangChain BaseTool integration
│   └── client.py    # HTTP client
├── tests/           # pytest test suite
├── data/dids/       # DID document store
└── demo.py          # End-to-end demo
```

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/did/create` | Create new DID and register agent |
| GET | `/did/resolve/{did}` | Resolve DID to DID Document |
| POST | `/did/verify` | Verify Ed25519 signature |
| POST | `/auth/token` | Issue scoped JWT |
| POST | `/auth/verify-token` | Verify JWT |
| POST | `/payment/execute` | Execute payment (requires JWT) |
| GET | `/payment/{transaction_id}` | Get payment status |
| GET | `/audit/{agent_did}` | Get audit hash chain |
| POST | `/audit/verify` | Verify chain integrity |
| GET | `/health` | Health check |

## SDK Usage

```python
from sdk.wallet import AgentWallet

async def main():
    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="my-agent", max_limit=50000)

    result = await wallet.pay(amount=5000, description="Purchase")
    print(result["transaction_id"])

    chain = await wallet.get_audit_chain()
    print(f"Chain valid: {chain['chain_valid']}")

    await wallet.close()
```

### LangChain Integration

```python
from sdk.wallet import AgentWallet
from sdk.tools import PaymentTool

wallet = AgentWallet(...)
await wallet.create(...)

tool = PaymentTool(wallet=wallet)
# Add tool to your LangChain agent
```
