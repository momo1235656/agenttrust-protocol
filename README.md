# AgentTrust Protocol
<img width="1353" height="399" alt="スクリーンショット 2026-03-12 3 03 03" src="https://github.com/user-attachments/assets/88a06134-2a3b-4ea0-8b14-e469a0d67ef8" />

**Secure Payment Infrastructure for AI Agents**
**AIエージェントのための安全な決済インフラ**

[![Python Tests](https://github.com/momo1235656/agenttrust-protocol/actions/workflows/test-python.yml/badge.svg)](https://github.com/momo1235656/agenttrust-protocol/actions/workflows/test-python.yml)
[![TypeScript Tests](https://github.com/momo1235656/agenttrust-protocol/actions/workflows/test-typescript.yml/badge.svg)](https://github.com/momo1235656/agenttrust-protocol/actions/workflows/test-typescript.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## What is AgentTrust? / AgentTrustとは？

**English:**
AI agents are increasingly capable of taking real-world actions — including making purchases. But they have no passport, no bank account, and no credit history. AgentTrust solves this by giving every AI agent a cryptographic identity (DID), a permission-scoped access token (JWT), and a tamper-proof transaction log (hash chain).

Think of it as **OAuth + identity + audit logging, purpose-built for AI agents**.

**日本語:**
AIエージェントは購入・予約・契約といったリアルワールドのアクションを実行できるようになっています。しかし、エージェントには「身分証明書」も「銀行口座」も「信用情報」もありません。AgentTrustはこの問題を解決します。すべてのAIエージェントに、暗号学的なID（DID）、権限スコープ付きアクセストークン（JWT）、改ざん不能な取引記録（ハッシュチェーン）を付与します。

一言でいえば、**「AIエージェント専用のOAuth + ID + 監査ログ基盤」**です。

---

## Core Concepts / コアコンセプト

### 🪪 DID — Agent Identity / エージェントの身元証明

**English:**
Each agent is assigned a [W3C DID](https://www.w3.org/TR/did-core/) (`did:key` method) derived from an Ed25519 public key. Only the holder of the private key can prove ownership of the DID. No central authority issues or revokes it.

**日本語:**
各エージェントにはEd25519公開鍵から導出されたW3C準拠のDID（`did:key`方式）が付与されます。秘密鍵を持つ者だけがDIDの所有権を証明できます。中央機関による発行・失効は不要です。

```
did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
         ^
         Ed25519公開鍵（multicodec + base58btc エンコード）
```

### 🔑 Scoped JWT — Fine-grained Authorization / きめ細かい権限制御

**English:**
After proving DID ownership via Ed25519 signature, an agent receives a JWT that embeds its spending limits and allowed categories. The payment API enforces these limits server-side — the agent literally cannot overspend.

**日本語:**
Ed25519署名でDIDの所有権を証明したエージェントは、支払い上限と許可カテゴリが組み込まれたJWTを受け取ります。決済APIはこの制限をサーバー側で強制します。エージェントは物理的に限度を超えた支払いができません。

```json
{
  "sub": "did:key:z6Mk...",
  "scopes": ["payment:execute"],
  "max_amount": 50000,
  "allowed_categories": ["electronics", "software"],
  "exp": 1710001800
}
```

### 🔗 Hash Chain Audit Log / ハッシュチェーン監査ログ

**English:**
Every transaction is recorded in an append-only SHA-256 hash chain. Each entry contains the hash of the previous entry, making it impossible to alter or delete past records without detection.

**日本語:**
すべての取引は追記専用のSHA-256ハッシュチェーンに記録されます。各エントリには前のエントリのハッシュが含まれるため、過去の記録を改ざん・削除すると検知されます。

```
取引 #0: hash = SHA256(0 | tx_001 | 3000 | succeeded | timestamp | "000...0")
取引 #1: hash = SHA256(1 | tx_002 | 5000 | succeeded | timestamp | hash_0)
取引 #2: hash = SHA256(2 | tx_003 | 2000 | failed    | timestamp | hash_1)
                                                                    ^^^^^^
                                                                    前のハッシュへの参照
```

---

## Architecture / アーキテクチャ

### Phase 2 (Current) — Rust/Axum Server

```
┌─────────────────────────────────────────────────────────────┐
│                    AI Agent (your code)                      │
│            LangChain / AutoGen / OpenClaw / MCP             │
└──────────────────────┬──────────────────────────────────────┘
                       │ SDK（変更なし）
          ┌────────────┼────────────┐
          │            │            │
    Python SDK   TypeScript SDK   MCP Server
    (sdk/)       (sdk-ts/)        (mcp-server/)
          │            │            │
          └────────────┼────────────┘
                       │ HTTP REST API（100% 後方互換）
┌──────────────────────▼──────────────────────────────────────┐
│         AgentTrust Server — Rust / Axum 0.7 (Phase 2)       │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐      │
│  │  DID API │ │ Auth API │ │  Pay API │ │ Audit API │      │
│  └──────────┘ └──────────┘ └──────────┘ └───────────┘      │
│  ┌──────────┐ ┌──────────────┐ ┌────────────────────────┐  │
│  │ OAuth2.0 │ │ Approval API │ │ Circuit Breaker        │  │
│  │  (RFC    │ │ Human-in-    │ │ Rate Limiter (Redis)   │  │
│  │  6749)   │ │ the-Loop     │ │ Stripe / PayPay        │  │
│  └──────────┘ └──────────────┘ └────────────────────────┘  │
│  ┌──────────────────────────┐  ┌──────────────────────────┐ │
│  │  PostgreSQL 16           │  │  Ed25519 Crypto (pure    │ │
│  │  (agents, transactions,  │  │  Rust, no OpenSSL)       │ │
│  │  audit_logs, oauth_*,    │  │  SHA-256 Hash Chain      │ │
│  │  approvals)              │  │  JWT via PKCS8 DER       │ │
│  └──────────────────────────┘  └──────────────────────────┘ │
│                     ┌──────────┐                             │
│                     │  Redis 7 │ (session / rate limit)      │
│                     └──────────┘                             │
└─────────────────────────────────────────────────────────────┘
```

---

## Tech Stack / 技術スタック

### Phase 2 — Rust Server (Current)

| Layer / 層 | Technology / 技術 | Reason / 採用理由 |
|-----------|-------------------|-----------------|
| API Server | **Axum 0.7 + tokio** | 1,000+ req/s, async |
| Cryptography | **ed25519-dalek** (pure Rust) | No OpenSSL dependency |
| JWT | **jsonwebtoken** (EdDSA) | Ed25519-signed tokens |
| Database | **sqlx + PostgreSQL 16** | Async, connection pool |
| Cache / Rate Limit | **Redis 7** | Sliding window rate limiting |
| Payment | **reqwest** → Stripe REST API | No heavy SDK dependency |
| OAuth 2.0 | Custom implementation | RFC 6749 compliant |
| Human-in-Loop | Webhook + approval table | Async approval flow |
| Circuit Breaker | In-memory (atomic counters) | Zero-latency state check |

### Phase 1 — Python Server (Legacy, still functional)

| Layer / 層 | Technology / 技術 |
|-----------|-------------------|
| API Server | FastAPI + Uvicorn |
| Cryptography | PyNaCl (Ed25519) |
| JWT | PyJWT + cryptography |
| Database | SQLite + SQLAlchemy (async) |
| Payment | Stripe Python SDK |

### SDKs (Unchanged across phases)

| SDK | Technology |
|-----|------------|
| Python SDK | Pure Python + httpx |
| TypeScript SDK | @noble/ed25519 + fetch |
| MCP Server | Anthropic MCP SDK |

---

## Project Structure / プロジェクト構造

```
agenttrust-protocol/
│
├── server-rust/              # ★ Phase 2: Rust/Axum サーバー
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs           #   エントリーポイント
│   │   ├── config.rs         #   環境変数設定
│   │   ├── error.rs          #   統一エラー型
│   │   ├── state.rs          #   AppState (DB/Redis/CircuitBreaker)
│   │   ├── crypto/           #   Ed25519、SHA-256、JWT（純粋Rust）
│   │   ├── routes/           #   Axum HTTPハンドラ（19エンドポイント）
│   │   ├── services/         #   ビジネスロジック
│   │   ├── models/           #   SQLxモデル（PostgreSQL）
│   │   ├── middleware/        #   JWT認証・レート制限・CB
│   │   └── payment_providers/ #   Stripe/PayPay（トレイト抽象化）
│   └── migrations/           #   PostgreSQL マイグレーション SQL
│
├── server/                   # Phase 1: FastAPI サーバー（非推奨・動作維持）
│   ├── main.py
│   ├── routers/              #   DID / Auth / Payment / Audit
│   ├── services/
│   ├── models/
│   ├── schemas/
│   └── crypto/
│
├── sdk/                      # Python SDK（変更なし）
│   ├── wallet.py             #   AgentWallet（メインクラス）
│   ├── tools.py              #   LangChain BaseTool 統合
│   ├── autogen_tools.py      #   AutoGen v0.4+ ツール
│   ├── openclaw_tools.py     #   OpenClaw アクション
│   └── client.py             #   HTTP クライアント
│
├── sdk-ts/                   # TypeScript SDK（変更なし）
│   └── src/
│       ├── wallet.ts
│       ├── client.ts
│       └── crypto.ts
│
├── mcp-server/               # MCP サーバー（変更なし）
│
├── docker/                   # ★ Phase 2: Docker 構成
│   ├── docker-compose.yml    #   PostgreSQL + Redis + Rustサーバー
│   ├── Dockerfile.server     #   マルチステージ Rust ビルド
│   └── init-db.sql
│
├── tests/                    # Python テストスイート
├── examples/                 # フレームワーク別サンプル
├── docs/                     # MkDocs ドキュメントサイト
└── .github/workflows/        # CI/CD パイプライン
```

---

## Quick Start / クイックスタート

### Option A: Rust Server (Phase 2) — Recommended

#### Prerequisites
- Rust 1.88+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Docker & Docker Compose（PostgreSQL + Redis 用）
- Stripe アカウント（テストモードの API キー）

```bash
git clone https://github.com/momo1235656/agenttrust-protocol.git
cd agenttrust-protocol

# Docker で PostgreSQL + Redis 起動
cd docker
cp ../server-rust/.env.example .env
# .env を編集: STRIPE_SECRET_KEY を設定

# 全スタック起動（PostgreSQL + Redis + Rustサーバー）
# ※ 初回ビルドは依存クレート(364パッケージ)のコンパイルのため10〜20分かかります
docker-compose up

# または Rust サーバーをローカルで起動
cd ../server-rust
cargo run
```

ヘルスチェック: http://localhost:8000/health

### Option B: Python Server (Phase 1) — Legacy

#### Prerequisites
- Python 3.11+
- Stripe アカウント

```bash
pip install -e ".[dev]"
cp .env.example .env
# .env: STRIPE_SECRET_KEY を設定

uvicorn server.main:app --reload --port 8000
```

Swagger UI: http://localhost:8000/docs

### Run Demo / デモ実行

```bash
# サーバー起動後（どちらのサーバーでも動作）
python demo.py
```

```
DID: did:key:z6MkhaXgBZDvot...
決済結果: {'transaction_id': 'tx_a1b2c3...', 'status': 'succeeded', 'amount': 5000, ...}
```

### Run Tests / テスト実行

```bash
# Python テスト (Phase 1)
pytest tests/ -v

# TypeScript テスト
cd sdk-ts && npm install && npm test

# Rust 単体テスト (Phase 2)
cd server-rust
cargo test test_crypto test_hashing test_circuit_breaker

# Rust 統合テスト (サーバー起動後)
TEST_SERVER_URL=http://localhost:8000 cargo test
```

---

## SDK Usage / SDK の使い方

### Python SDK

#### Basic Payment / 基本的な決済

```python
import asyncio
from sdk.wallet import AgentWallet

async def main():
    # 1. ウォレット作成（DID 自動生成）
    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(
        display_name="shopping-agent",
        max_limit=50000,          # 上限 5万円
    )
    print(f"Agent DID: {wallet.did}")

    # 2. 決済実行
    result = await wallet.pay(
        amount=3000,
        description="商品名: ノートPC ケース",
        idempotency_key="order-20260311-001",  # 二重決済防止
    )
    print(f"Transaction ID: {result['transaction_id']}")
    print(f"Status: {result['status']}")
    print(f"Audit Hash: {result['audit_hash']}")

    # 3. 監査ログ確認
    audit = await wallet.get_audit_log()
    print(f"Chain Valid: {audit['chain_valid']}")  # True

    await wallet.close()

asyncio.run(main())
```

#### Restore Existing Agent / 既存エージェントの復元

```python
import base64
from sdk.wallet import AgentWallet

wallet = AgentWallet(
    server_url="http://localhost:8000",
    did="did:key:z6Mk...",
    private_key=base64.b64decode("your-saved-private-key-base64"),
)
result = await wallet.pay(amount=1000, description="定期購入")
```

### TypeScript SDK

```typescript
import { AgentWallet } from 'agenttrust';

async function main() {
  const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
  await wallet.create({ displayName: 'ts-agent', maxLimit: 50000 });
  console.log(`DID: ${wallet.did}`);

  const result = await wallet.pay({
    amount: 3000,
    description: '商品の購入',
    idempotencyKey: 'order-20260311-001',
  });
  console.log(`Status: ${result.status}`);

  const audit = await wallet.getAuditLog();
  console.log(`Chain Valid: ${audit.chain_valid}`);
}

main().catch(console.error);
```

---

## Framework Integrations / フレームワーク統合

### LangChain

```python
from sdk.wallet import AgentWallet
from sdk.tools import PaymentTool

wallet = AgentWallet(server_url="http://localhost:8000")
await wallet.create(display_name="langchain-agent")

tool = PaymentTool(wallet=wallet)
tools = [tool]  # agent の tools リストに追加
```

### AutoGen (v0.4+)

```python
from sdk.autogen_tools import create_payment_tools

tools = create_payment_tools(wallet)

from autogen_agentchat.agents import AssistantAgent
agent = AssistantAgent(
    name="payment_agent",
    model_client=model_client,
    tools=tools,
)
await agent.run(task="5000円の商品を購入してください")
```

### OpenClaw

```python
from sdk.openclaw_tools import AgentTrustPaymentAction

action = AgentTrustPaymentAction(wallet=wallet)
result = await action.execute({"amount": 5000, "description": "商品購入"})
```

### MCP (Claude Desktop / Cursor)

`claude_desktop_config.json` に追加：

```json
{
  "mcpServers": {
    "agenttrust": {
      "command": "python",
      "args": ["-m", "mcp_server.server"],
      "cwd": "/path/to/agenttrust-protocol",
      "env": {
        "AGENTTRUST_SERVER_URL": "http://localhost:8000"
      }
    }
  }
}
```

---

## API Endpoints / API エンドポイント一覧

### Phase 1 Compatible (MVP) — 全SDK対応

| Method | Path | Description | 説明 |
|--------|------|-------------|------|
| `POST` | `/did/create` | Create agent DID | エージェント DID 作成 |
| `GET` | `/did/resolve/{did}` | Resolve DID Document | DID ドキュメント取得 |
| `POST` | `/did/verify` | Verify Ed25519 signature | 署名検証 |
| `POST` | `/auth/token` | Issue scoped JWT | スコープ付き JWT 発行 |
| `POST` | `/auth/verify-token` | Verify JWT | JWT 検証 |
| `POST` | `/payment/execute` | Execute payment | 決済実行（JWT 必須） |
| `GET` | `/payment/{id}` | Get payment status | 決済ステータス確認 |
| `GET` | `/audit/{did}` | Get audit hash chain | 監査ログ取得 |
| `POST` | `/audit/verify` | Verify chain integrity | チェーン整合性検証 |

### Phase 2 New Endpoints — Rust Server のみ

| Method | Path | Description | 説明 |
|--------|------|-------------|------|
| `POST` | `/oauth/authorize` | OAuth 2.0 Authorization Code | 認可コード発行 |
| `POST` | `/oauth/token` | Token issuance (4 grant types) | トークン発行（4種類） |
| `POST` | `/oauth/revoke` | Token revocation | トークン失効 |
| `GET` | `/oauth/jwks` | JWK Set | JWT検証用公開鍵セット |
| `POST` | `/oauth/register` | Register OAuth client | OAuthクライアント登録 |
| `POST` | `/approval/request` | Request human approval | 人間の承認リクエスト |
| `POST` | `/approval/{id}/approve` | Approve transaction | 決済承認 |
| `POST` | `/approval/{id}/reject` | Reject transaction | 決済拒否 |
| `POST` | `/payment/refund` | Refund payment | 返金 |
| `GET` | `/payment/methods` | List payment methods | 決済手段一覧 |
| `GET` | `/health` | Health check | ヘルスチェック |

---

## Error Codes / エラーコード

| Code | HTTP | Description / 説明 |
|------|------|-------------------|
| `DID_NOT_FOUND` | 404 | DID does not exist / DID が存在しない |
| `INVALID_SIGNATURE` | 401 | Ed25519 signature mismatch / 署名が不一致 |
| `TOKEN_EXPIRED` | 401 | JWT has expired / JWT の有効期限切れ |
| `TOKEN_INVALID` | 401 | JWT is malformed / JWT が無効 |
| `SCOPE_EXCEEDED` | 403 | Amount exceeds token limit / 金額がスコープ外 |
| `DUPLICATE_TRANSACTION` | 409 | Idempotency key already used / 冪等キー重複 |
| `PAYMENT_FAILED` | 502 | Payment provider error / 決済プロバイダーエラー |
| `CHAIN_INVALID` | 500 | Hash chain integrity failure / ハッシュチェーン破損 |
| `APPROVAL_REQUIRED` | 202 | High-value tx needs approval / 高額決済には承認が必要 |
| `AGENT_FROZEN` | 403 | Agent account frozen / エージェントが凍結済み |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests / レート制限超過 |

---

## Security Design / セキュリティ設計

**English:**
- Private keys never leave the SDK — only the public key is sent to the server
- JWT scopes enforce spending limits server-side (not just client-side)
- Idempotency keys prevent duplicate payments even under network retries
- High-value transactions (above `APPROVAL_REQUIRED_ABOVE`) require human approval via webhook
- Circuit breaker automatically blocks Stripe calls after 5 consecutive failures
- Redis rate limiting prevents abuse at both agent and IP level
- All secrets loaded from environment variables — nothing hardcoded
- Ed25519 private keys are zeroed in memory after use (`ZeroizeOnDrop`)

**日本語:**
- 秘密鍵はSDK外に出ません。サーバーには公開鍵のみ送信されます
- JWTスコープによる支払い上限はサーバー側で強制されます
- 冪等キーにより、ネットワーク再試行時でも二重決済を防止します
- 高額決済（`APPROVAL_REQUIRED_ABOVE`以上）はWebhook経由で人間の承認が必要です
- Stripeへの連続失敗5回でサーキットブレーカーが自動発動します
- RedisによるレートリミットでエージェントとIPの両レベルで乱用を防止します
- すべてのシークレットは環境変数から読み込みます
- Ed25519秘密鍵は使用後メモリからゼロ埋めされます（`ZeroizeOnDrop`）

---

## Roadmap / ロードマップ

| Phase | Status | Content |
|-------|--------|---------|
| **MVP** | ✅ Done | FastAPI server, Python SDK, SQLite, Stripe, hash chain audit |
| **Phase 1** | ✅ Done | TypeScript SDK, AutoGen, OpenClaw, MCP server, MkDocs, CI/CD |
| **Phase 2** | ✅ **Done** | **Rust/Axum server**, PostgreSQL, Redis, OAuth 2.0, Human-in-the-Loop, Circuit Breaker |
| **Phase 3** | 🔜 Planned | Production deployment (AWS/GCP), monitoring (Prometheus/Grafana) |
| **Phase 4** | 🔜 Planned | On-chain DID anchoring (Ethereum / Solana) |
| **Phase 5** | 🔜 Planned | `agenttrust-crypto` crate (PyO3 bindings for Python SDK) |

---

## Contributing / コントリビュート

```bash
# フォークしてブランチを作成
git checkout -b feature/your-feature

# 変更してテスト
pytest tests/ -v
cd sdk-ts && npm test
cd server-rust && cargo test

# プルリクエストを作成
gh pr create
```

---

## License / ライセンス

MIT License — see [LICENSE](LICENSE) for details.

---

## Links / リンク

- 📖 **Documentation**: https://momo1235656.github.io/agenttrust-protocol
- 🐛 **Issues**: https://github.com/momo1235656/agenttrust-protocol/issues
- 💬 **Discussions**: https://github.com/momo1235656/agenttrust-protocol/discussions
