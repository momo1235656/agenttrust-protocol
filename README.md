# AgentTrust Protocol

<img width="1353" height="399" alt="スクリーンショット 2026-03-12 3 03 03" src="https://github.com/user-attachments/assets/88a06134-2a3b-4ea0-8b14-e469a0d67ef8" />

**Secure Payment Infrastructure for AI Agents**
**AI エージェントのための安全な決済インフラ**

[![Python Tests](https://github.com/momo1235656/agenttrust-protocol/actions/workflows/test-python.yml/badge.svg)](https://github.com/momo1235656/agenttrust-protocol/actions/workflows/test-python.yml)
[![TypeScript Tests](https://github.com/momo1235656/agenttrust-protocol/actions/workflows/test-typescript.yml/badge.svg)](https://github.com/momo1235656/agenttrust-protocol/actions/workflows/test-typescript.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

---

## What is AgentTrust? / AgentTrust とは？

**English:**
AI agents are increasingly capable of taking real-world actions — including making purchases and trading services with each other. But they have no passport, no bank account, and no credit history. AgentTrust solves this by giving every AI agent a cryptographic identity (DID), a permission-scoped access token (JWT), a tamper-proof transaction log (hash chain), and now a full **Agent-to-Agent (A2A) payment layer** backed by escrow, saga orchestration, and flow control.

Think of it as **OAuth + identity + audit logging + A2A escrow payments, purpose-built for AI agents**.

**日本語:**
AI エージェントは購入・予約・契約だけでなく、**エージェント同士でサービスを取引する**ようになっています。しかし、エージェントには「身分証明書」も「銀行口座」も「信用情報」もありません。AgentTrust はこの問題を解決します。すべての AI エージェントに、暗号学的な ID（DID）、権限スコープ付きアクセストークン（JWT）、改ざん不能な取引記録（ハッシュチェーン）、そして**エスクロー決済・Saga オーケストレーション・フロー制御を備えた A2A 決済レイヤー**を付与します。

一言でいえば、**「AI エージェント専用の OAuth + ID + 監査ログ + A2A エスクロー決済基盤」**です。

---

## Core Concepts / コアコンセプト

### 🪪 DID — Agent Identity / エージェントの身元証明

**English:**
Each agent is assigned a [W3C DID](https://www.w3.org/TR/did-core/) (`did:key` method) derived from an Ed25519 public key. Only the holder of the private key can prove ownership of the DID. No central authority issues or revokes it.

**日本語:**
各エージェントには Ed25519 公開鍵から導出された W3C 準拠の DID（`did:key`方式）が付与されます。秘密鍵を持つ者だけが DID の所有権を証明できます。中央機関による発行・失効は不要です。

```
did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
         ^
         Ed25519公開鍵（multicodec + base58btc エンコード）
```

### 🔑 Scoped JWT — Fine-grained Authorization / きめ細かい権限制御

**English:**
After proving DID ownership via Ed25519 signature, an agent receives a JWT that embeds its spending limits and allowed categories. The payment API enforces these limits server-side — the agent literally cannot overspend.

**日本語:**
Ed25519 署名で DID の所有権を証明したエージェントは、支払い上限と許可カテゴリが組み込まれた JWT を受け取ります。決済 API はこの制限をサーバー側で強制します。エージェントは物理的に限度を超えた支払いができません。

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
すべての取引は追記専用の SHA-256 ハッシュチェーンに記録されます。各エントリには前のエントリのハッシュが含まれるため、過去の記録を改ざん・削除すると検知されます。

```
取引 #0: hash = SHA256(0 | tx_001 | 3000 | succeeded | timestamp | "000...0")
取引 #1: hash = SHA256(1 | tx_002 | 5000 | succeeded | timestamp | hash_0)
取引 #2: hash = SHA256(2 | tx_003 | 2000 | failed    | timestamp | hash_1)
                                                                    ^^^^^^
                                                                    前のハッシュへの参照
```

---

## Architecture / アーキテクチャ

### Phase 4 (Current) — A2A Payments, Escrow, Saga, Flow Control, Kafka, gRPC

```
┌──────────────────────────────────────────────────────────────────┐
│            AI Agent A (sender)    AI Agent B (receiver)          │
│            LangChain / AutoGen / OpenClaw / MCP                  │
└─────────────┬─────────────────────────────────┬──────────────────┘
              │ SDK                              │ SDK / gRPC
   ┌──────────┼──────────────┐      ┌───────────┴──────────────┐
   │  Python SDK  TS SDK  MCP│      │  gRPC Client (port 50052)│
   └──────────┼──────────────┘      └───────────┬──────────────┘
              │ HTTP REST API（100% 後方互換）   │ gRPC
┌─────────────▼─────────────────────────────────▼──────────────────┐
│          AgentTrust Server — Rust / Axum 0.7 (Phase 4)           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐           │
│  │  DID API │ │ Auth API │ │  Pay API │ │ Audit API │           │
│  └──────────┘ └──────────┘ └──────────┘ └───────────┘           │
│  ┌──────────┐ ┌──────────────┐ ┌──────────────────────────────┐ │
│  │ OAuth2.0 │ │ Approval API │ │ Trust Score / VC / Fraud     │ │
│  └──────────┘ └──────────────┘ └──────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │                  ★ Phase 4: A2A Payment Layer              │  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌────────────────────┐ │  │
│  │  │  A2A Service │ │   Escrow     │ │  Saga Orchestrator │ │  │
│  │  │  (10-step)   │ │  (funded /   │ │  (10 steps +       │ │  │
│  │  │              │ │   released / │ │   backward         │ │  │
│  │  │              │ │   refunded / │ │   compensation)    │ │  │
│  │  │              │ │   disputed)  │ │                    │ │  │
│  │  └──────────────┘ └──────────────┘ └────────────────────┘ │  │
│  │  ┌──────────────────────┐ ┌──────────────────────────────┐ │  │
│  │  │  Flow Controller     │ │  gRPC Server (port 50052)    │ │  │
│  │  │  (rate / pair /      │ │  InitiateTransfer            │ │  │
│  │  │   chain depth / BFS) │ │  WatchTransfer (streaming)   │ │  │
│  │  └──────────────────────┘ └──────────────────────────────┘ │  │
│  └────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────┐  ┌──────────────────────────────┐  │
│  │  PostgreSQL 16           │  │  Apache Kafka                │  │
│  │  (+ a2a_transfers,       │  │  a2a.transfer / escrow /     │  │
│  │   escrows, sagas,        │  │  saga / fraud / trust topics │  │
│  │   saga_steps,            │  │  (best-effort, optional)     │  │
│  │   flow_policies)         │  └──────────────────────────────┘  │
│  └──────────────────────────┘                                     │
│       ┌──────────┐   ┌──────────────────────────────────────┐    │
│       │  Redis 7 │   │  Scheduler (tokio::time::interval)   │    │
│       │ (flow    │   │  Escrow timeout / Saga timeout        │    │
│       │  counters│   │  → 自動補償トランザクション (60s毎)  │    │
│       │  + rate) │   └──────────────────────────────────────┘    │
│       └──────────┘                                                │
└───────────────────────────────────────────────────────────────────┘
```

---

## Tech Stack / 技術スタック

### Phase 4 — Rust Server (Current)

| Layer / 層            | Technology / 技術               | Reason / 採用理由                                               |
| --------------------- | ------------------------------- | --------------------------------------------------------------- |
| API Server            | **Axum 0.7 + tokio**            | 1,000+ req/s, async                                             |
| Cryptography          | **ed25519-dalek** (pure Rust)   | No OpenSSL dependency                                           |
| JWT                   | **jsonwebtoken** (EdDSA)        | Ed25519-signed tokens                                           |
| Database              | **sqlx + PostgreSQL 16**        | Async, connection pool                                          |
| Cache / Rate Limit    | **Redis 7**                     | Sliding window rate limiting + Flow counters                    |
| Payment               | **reqwest** → Stripe REST API   | No heavy SDK dependency                                         |
| OAuth 2.0             | Custom implementation           | RFC 6749 compliant                                              |
| Human-in-Loop         | Webhook + approval table        | Async approval flow                                             |
| Circuit Breaker       | In-memory (atomic counters)     | Zero-latency state check                                        |
| Trust Score Engine    | Weighted formula (5 metrics)    | 取引履歴から信頼スコア(0–100)を算出                             |
| VC Issuer             | W3C VC Data Model 2.0           | Ed25519 署名付き検証可能クレデンシャル発行                      |
| Fraud Detection       | Rule engine (5 rules)           | リアルタイム不正検知・リスクスコア算出                          |
| DID Registry (L2)     | Solidity + Foundry              | Polygon Amoy オンチェーン DID 管理                              |
| ML Service            | Python gRPC                     | 不正検知ルールエンジン（Rust 実装のミラー）                     |
| **A2A Payment**       | **Stripe Connect + Saga**       | エージェント間の直接送金（10 ステップ Saga）                    |
| **Escrow Service**    | **PostgreSQL + Stripe Connect** | 取引完了まで資金を仮預かり（funded/released/refunded/disputed） |
| **Saga Orchestrator** | **tokio async tasks**           | 分散トランザクション管理・逆順補償トランザクション              |
| **Flow Controller**   | **Redis counters + BFS**        | レート制限・同一ペア制限・チェーン深度・循環検知                |
| **Kafka Events**      | **rdkafka 0.36**                | イベント駆動アーキテクチャ（best-effort、オプション）           |
| **gRPC Server**       | **tonic 0.11 + prost 0.12**     | port 50052 でエージェント間高速通信                             |
| **Scheduler**         | **tokio::time::interval**       | エスクロー/Saga タイムアウト自動補償（60 秒毎）                 |

### Phase 1 — Python Server (Legacy, still functional)

| Layer / 層   | Technology / 技術           |
| ------------ | --------------------------- |
| API Server   | FastAPI + Uvicorn           |
| Cryptography | PyNaCl (Ed25519)            |
| JWT          | PyJWT + cryptography        |
| Database     | SQLite + SQLAlchemy (async) |
| Payment      | Stripe Python SDK           |

### SDKs (Unchanged across phases)

| SDK            | Technology             |
| -------------- | ---------------------- |
| Python SDK     | Pure Python + httpx    |
| TypeScript SDK | @noble/ed25519 + fetch |
| MCP Server     | Anthropic MCP SDK      |

---

## Project Structure / プロジェクト構造

```
agenttrust-protocol/
│
├── server-rust/              # ★ Phase 4: Rust/Axum サーバー
│   ├── Cargo.toml
│   ├── build.rs              # ★ Phase 4: tonic proto コンパイル
│   ├── proto/
│   │   ├── payment.proto     # ★ Phase 4: gRPC サービス定義
│   │   └── events.proto      # ★ Phase 4: Kafka イベント定義
│   ├── src/
│   │   ├── main.rs           #   エントリーポイント（gRPC + Scheduler 起動）
│   │   ├── config.rs         #   環境変数設定
│   │   ├── error.rs          #   統一エラー型
│   │   ├── state.rs          #   AppState (DB/Redis/Kafka/CircuitBreaker)
│   │   ├── crypto/           #   Ed25519、SHA-256、JWT（純粋Rust）
│   │   ├── events/           # ★ Phase 4: Kafka イベント型
│   │   │   └── types.rs      #   AgentTrustEvent 共通構造
│   │   ├── grpc/             # ★ Phase 4: gRPC サーバー（port 50052）
│   │   │   ├── server.rs     #   tonic サーバー起動
│   │   │   └── payment_grpc.rs # InitiateTransfer / WatchTransfer streaming
│   │   ├── scheduler/        # ★ Phase 4: バックグラウンドジョブ
│   │   │   ├── escrow_timeout.rs  # エスクロー期限切れ自動返金
│   │   │   └── saga_timeout.rs    # Saga タイムアウト自動補償
│   │   ├── routes/           #   Axum HTTPハンドラ（38エンドポイント）
│   │   │   ├── trust.rs      #   Phase 3: 信頼スコア API
│   │   │   ├── vc.rs         #   Phase 3: 検証可能クレデンシャル API
│   │   │   ├── fraud.rs      #   Phase 3: 不正検知 API
│   │   │   ├── a2a.rs        # ★ Phase 4: A2A 送金 API
│   │   │   ├── escrow.rs     # ★ Phase 4: エスクロー API
│   │   │   ├── saga.rs       # ★ Phase 4: Saga ステータス・完了・補償
│   │   │   └── flow.rs       # ★ Phase 4: フロー制御ポリシー API
│   │   ├── services/
│   │   │   ├── trust_service.rs  # Phase 3: スコア算出ロジック
│   │   │   ├── vc_service.rs     # Phase 3: VC 発行・検証・失効
│   │   │   ├── fraud_service.rs  # Phase 3: 不正ルールエンジン
│   │   │   ├── kafka_service.rs  # ★ Phase 4: Kafka Producer（best-effort）
│   │   │   ├── flow_service.rs   # ★ Phase 4: フロー制御・BFS チェーン深度
│   │   │   ├── escrow_service.rs # ★ Phase 4: エスクロー CRUD
│   │   │   ├── saga_service.rs   # ★ Phase 4: Saga ステップ管理・補償
│   │   │   └── a2a_service.rs    # ★ Phase 4: A2A 送金オーケストレーション
│   │   ├── models/
│   │   │   ├── trust_score.rs / verifiable_credential.rs / fraud_alert.rs
│   │   │   ├── a2a_transfer.rs   # ★ Phase 4
│   │   │   ├── escrow.rs         # ★ Phase 4
│   │   │   ├── saga.rs           # ★ Phase 4（Saga + SagaStep）
│   │   │   └── flow_policy.rs    # ★ Phase 4
│   │   ├── middleware/        #   JWT認証・レート制限・CB
│   │   └── payment_providers/ #   Stripe/PayPay（トレイト抽象化）
│   └── migrations/           #   PostgreSQL マイグレーション SQL (014まで)
│       ├── 010_create_a2a_transfers.sql
│       ├── 011_create_escrows.sql
│       ├── 012_create_sagas.sql
│       ├── 013_create_saga_steps.sql
│       └── 014_create_flow_policies.sql
│
├── contracts/                # Phase 3: Solidity スマートコントラクト
│   ├── src/DIDRegistry.sol   #   オンチェーン DID レジストリ
│   ├── test/DIDRegistry.t.sol
│   └── script/Deploy.s.sol
│
├── ml-service/               # Phase 3: Python gRPC 不正検知サービス
│
├── server/                   # Phase 1: FastAPI サーバー（非推奨・動作維持）
│
├── sdk/                      # Python SDK
│   ├── wallet.py / tools.py / autogen_tools.py / openclaw_tools.py
│   ├── client.py / trust.py / vc.py
│   └── a2a.py                # ★ Phase 4: A2AClient
│
├── sdk-ts/                   # TypeScript SDK
│   └── src/
│       ├── wallet.ts / client.ts / crypto.ts / trust.ts / vc.ts
│       └── a2a.ts            # ★ Phase 4: A2AClient
│
├── mcp-server/               # MCP サーバー（変更なし）
│
├── docker/                   # ★ Phase 4: Docker 構成
│   ├── docker-compose.yml    #   PostgreSQL + Redis + Rust + ML + Anvil
│   │                         #   + Zookeeper + Kafka（Phase 4追加）
│   ├── Dockerfile.server     #   マルチステージ Rust ビルド
│   └── init-db.sql
│
├── tests/                    # Python テストスイート
├── examples/                 # フレームワーク別サンプル
├── docs/                     # MkDocs ドキュメントサイト
└── .github/workflows/        # CI/CD パイプライン
```

---

## Environment Variables / 環境変数

サーバー起動前に `server-rust/.env.example` を `.env` にコピーし、以下の変数を設定してください。

| Variable / 変数名                  | Required | Default                              | Description / 説明                                                              |
| ---------------------------------- | -------- | ------------------------------------ | ------------------------------------------------------------------------------- |
| `DATABASE_URL`                     | ✅       | —                                    | PostgreSQL 接続 URL                                                             |
| `STRIPE_SECRET_KEY`                | ✅       | —                                    | Stripe シークレットキー (`sk_test_...` / `sk_live_...`)                         |
| `REDIS_URL`                        | ❌       | `redis://localhost:6379`             | Redis 接続 URL                                                                  |
| `JWT_SERVER_PRIVATE_KEY`           | ❌       | (自動生成・起動時に stderr 出力)     | Ed25519 秘密鍵（Base64 32 bytes）。未設定時は起動毎に新しい鍵が生成されます     |
| `JWT_SERVER_PUBLIC_KEY`            | ❌       | (自動生成・起動時に stderr 出力)     | Ed25519 公開鍵（Base64 32 bytes）。永続化する場合は明示的に設定してください     |
| `KAFKA_BROKERS`                    | ❌       | (無効)                               | Kafka ブローカーアドレス。未設定の場合 Kafka は無効化されます                   |
| `APPROVAL_REQUIRED_ABOVE`          | ❌       | `30000`                              | この金額（最小通貨単位）を超えた決済は Human-in-the-Loop 承認が必要             |
| `APPROVAL_WEBHOOK_DEFAULT_URL`     | ❌       | —                                    | 承認リクエスト通知先 Webhook URL                                                |
| `APPROVAL_EXPIRY_HOURS`            | ❌       | `24`                                 | 承認待ちリクエストの有効期限（時間）                                            |
| `STRIPE_WEBHOOK_SECRET`            | ❌       | —                                    | Stripe Webhook 署名検証シークレット (`whsec_...`)                               |
| `CORS_ORIGINS`                     | ❌       | `http://localhost:3000,...`          | CORS 許可オリジン（カンマ区切り）                                               |
| `DID_STORE_PATH`                   | ❌       | `./data/dids`                        | DID ドキュメントストアのパス                                                    |
| `RATE_LIMIT_PER_AGENT_PER_MINUTE`  | ❌       | `60`                                 | エージェントあたりの毎分リクエスト上限                                          |
| `RATE_LIMIT_PER_IP_PER_MINUTE`     | ❌       | `120`                                | IP あたりの毎分リクエスト上限                                                   |
| `PORT`                             | ❌       | `8000`                               | HTTP サーバーポート                                                             |
| `RUST_LOG`                         | ❌       | `info`                               | ログレベル (`trace`, `debug`, `info`, `warn`, `error`)                          |

> **JWT 鍵の自動生成について**: `JWT_SERVER_PRIVATE_KEY` が未設定の場合、サーバー起動時にエフェメラルな Ed25519 鍵ペアを自動生成し、Base64 エンコードされた値を stderr に出力します。開発環境ではそのまま使用可能ですが、**本番環境または再起動をまたいで JWT を有効にしたい場合は、出力された値を `.env` に設定してください**（未設定のまま再起動すると全 JWT が無効化されます）。

```env
# server-rust/.env (最小構成)
DATABASE_URL=postgres://postgres:postgres@localhost:5432/agenttrust
STRIPE_SECRET_KEY=<your_stripe_test_secret_key>

# 推奨: 初回起動後に stderr 出力された鍵を設定
JWT_SERVER_PRIVATE_KEY=
JWT_SERVER_PUBLIC_KEY=

# Kafka を有効にする場合
# KAFKA_BROKERS=kafka:9092
```

---

## Quick Start / クイックスタート

### Option A: Rust Server (Phase 4) — Recommended

#### Prerequisites

- Rust 1.88+ (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Docker & Docker Compose（PostgreSQL + Redis + Kafka 用）
- Stripe アカウント（テストモードの API キー）
- Protocol Buffers compiler（gRPC ビルド用）: `brew install protobuf` / `apt install protobuf-compiler`
- cmake（rdkafka ビルド用）: `brew install cmake` / `apt install cmake`

```bash
git clone https://github.com/momo1235656/agenttrust-protocol.git
cd agenttrust-protocol

# Docker で全スタック起動（PostgreSQL + Redis + Kafka + Zookeeper + Rustサーバー）
cd docker
cp ../server-rust/.env.example .env
# .env を編集: STRIPE_SECRET_KEY を設定

# ※ 初回ビルドは依存クレート(400+パッケージ)のコンパイルのため15〜25分かかります
docker-compose up
```

- REST API: http://localhost:8000/health
- gRPC: localhost:50052

#### ローカル起動（Docker なし）

```bash
cd server-rust
cp .env.example .env
# .env を編集: DATABASE_URL / STRIPE_SECRET_KEY を設定

# 1. データベースマイグレーション
#    sqlx-cli が必要: cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx database create
sqlx migrate run

# 2. サーバー起動（初回のみ JWT 鍵を自動生成して stderr 出力）
cargo run
# → Generated JWT_SERVER_PRIVATE_KEY=AAAA...
# → Generated JWT_SERVER_PUBLIC_KEY=BBBB...
# 出力された値を .env に設定すると、再起動後も JWT が有効になります
```

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

# Rust 単体テスト (Phase 2–4)
cd server-rust
cargo test test_crypto test_hashing test_circuit_breaker

# Rust 統合テスト (サーバー起動後)
TEST_SERVER_URL=http://localhost:8000 cargo test

# DID Registry スマートコントラクト テスト (Phase 3, Foundry 必須)
cd contracts && forge test -v

# ML Service テスト (Phase 3)
cd ml-service && pip install pytest && pytest tests/ -v

# gRPC 動作確認 (Phase 4, grpcurl 必須: brew install grpcurl)
grpcurl -plaintext -proto server-rust/proto/payment.proto \
  localhost:50052 list agenttrust.payment.AgentPaymentService
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
import { AgentWallet } from "agenttrust";

async function main() {
  const wallet = new AgentWallet({ serverUrl: "http://localhost:8000" });
  await wallet.create({ displayName: "ts-agent", maxLimit: 50000 });
  console.log(`DID: ${wallet.did}`);

  const result = await wallet.pay({
    amount: 3000,
    description: "商品の購入",
    idempotencyKey: "order-20260311-001",
  });
  console.log(`Status: ${result.status}`);

  const audit = await wallet.getAuditLog();
  console.log(`Chain Valid: ${audit.chain_valid}`);
}

main().catch(console.error);
```

### Agent-to-Agent Payment / エージェント間決済 (Phase 4)

```python
import asyncio
from sdk.a2a import A2AClient

async def main():
    client = A2AClient(server_url="http://localhost:8000")

    # Agent A → Agent B にデータ分析を依頼（エスクロー経由）
    result = await client.initiate(
        sender_did="did:key:z6MkAgent_A...",
        receiver_did="did:key:z6MkAgent_B...",
        amount=15000,
        description="データ分析サービスの依頼",
        service_type="data_analysis",
        timeout_minutes=60,
    )
    print(f"Transfer ID: {result['transfer_id']}")
    print(f"Saga ID:     {result['saga_id']}")
    print(f"Status:      {result['status']}")       # "service_pending"
    print(f"Escrow:      {result['escrow_status']}") # "funded"

    # Agent B が作業完了を報告 → エスクロー自動解放 → 双方スコア更新
    await client.complete(
        saga_id=result['saga_id'],
        reporter_did="did:key:z6MkAgent_B...",
        result_summary="分析レポートを生成しました",
    )

asyncio.run(main())
```

---

## Trust Score Formula / 信頼スコア算出式

信頼スコアは 0〜100 の整数値で、以下の 5 つのメトリクスから算出されます。

```
trust_score = 50 + floor(raw_score × 50)   ← 最小 50、最大 100

raw_score =
    success_rate          × 0.30   # 決済成功率
  + (1 − dispute_rate)   × 0.25   # 紛争なし率
  + min(tx_count/1000,1) × 0.15   # 取引量スコア
  + min(counterparties/50,1) × 0.15  # 取引先多様性スコア
  + min(account_age_days/365,1) × 0.15  # アカウント年齢スコア
```

| Metric / メトリクス       | Weight | Description / 説明                              |
| ------------------------- | ------ | ----------------------------------------------- |
| `success_rate`            | 30%    | 成功取引数 / 総取引数（取引なし時は 1.0）        |
| `1 − dispute_rate`        | 25%    | 紛争なし率（現在は常に 1.0 — Known Limitations 参照） |
| `volume_score`            | 15%    | 総取引数が 1,000 件で満点                        |
| `diversity_score`         | 15%    | ユニーク取引先数が 50 で満点                     |
| `age_score`               | 15%    | アカウント登録から 365 日で満点                  |

**Risk Level Mapping** (VC / A2A 送金での信頼スコア解釈):

| Score | Risk Level | A2A 参加可否 |
| ----- | ---------- | ------------ |
| 80–100 | `low`     | ✅ 許可      |
| 60–79  | `medium`  | ✅ 許可      |
| 40–59  | `high`    | ✅ 許可      |
| 0–39   | `critical`| ❌ 拒否（最低閾値 30 未満）|

---

## A2A Saga — 10-Step Transaction Flow / 10 ステップ Saga フロー

A2A 送金は分散トランザクション（Saga パターン）として管理されます。各ステップは前のステップが成功してから実行され、失敗時は**逆順に補償トランザクション**が実行されます。

| Step | Name                 | Action / アクション                                      | Compensation / 補償アクション |
| ---- | -------------------- | -------------------------------------------------------- | ----------------------------- |
| 0    | `flow_check`         | レート制限・ペア制限・チェーン深度・循環検知             | (なし)                        |
| 1    | `did_verify_both`    | 送信者・受信者の DID 解決・存在確認                      | (なし)                        |
| 2    | `trust_mutual_check` | 双方の信頼スコアが最低閾値（30）以上か確認               | (なし)                        |
| 3    | `fraud_check`        | 不正検知ルールエンジン実行（block なら中断）             | (なし)                        |
| 4    | `escrow_fund`        | Stripe Connect でエスクロー口座に資金を拘束              | エスクロー返金（送信者へ）    |
| 5    | `notify_receiver`    | 受信者エージェントにサービス依頼を通知                   | 通知キャンセル                |
| 6    | `await_completion`   | ⏸ **受信者のサービス完了報告を待機**（タイムアウト付き） | エスクロー返金（送信者へ）    |
| 7    | `escrow_release`     | サービス確認後、エスクロー資金を受信者へ解放             | (不可逆 — 人手介入)           |
| 8    | `audit_record`       | 双方の監査ログ（ハッシュチェーン）に取引を記録           | 監査ログへのエラー記録        |
| 9    | `trust_update`       | 双方の信頼スコアを再計算・更新                           | (なし)                        |

> ステップ 6 はサービス完了を待機する**非同期チェックポイント**です。受信者が `/saga/:id/complete` を呼び出すか、タイムアウト（デフォルト 60 分）で自動補償が実行されます。

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

### Phase 1 Compatible (MVP) — 全 SDK 対応

| Method | Path                 | Description              | 説明                  |
| ------ | -------------------- | ------------------------ | --------------------- |
| `POST` | `/did/create`        | Create agent DID         | エージェント DID 作成 |
| `GET`  | `/did/resolve/{did}` | Resolve DID Document     | DID ドキュメント取得  |
| `POST` | `/did/verify`        | Verify Ed25519 signature | 署名検証              |
| `POST` | `/auth/token`        | Issue scoped JWT         | スコープ付き JWT 発行 |
| `POST` | `/auth/verify-token` | Verify JWT               | JWT 検証              |
| `POST` | `/payment/execute`   | Execute payment          | 決済実行（JWT 必須）  |
| `GET`  | `/payment/{id}`      | Get payment status       | 決済ステータス確認    |
| `GET`  | `/audit/{did}`       | Get audit hash chain     | 監査ログ取得          |
| `POST` | `/audit/verify`      | Verify chain integrity   | チェーン整合性検証    |

### Phase 2 Endpoints — Rust Server のみ

| Method | Path                     | Description                    | 説明                   |
| ------ | ------------------------ | ------------------------------ | ---------------------- |
| `POST` | `/oauth/authorize`       | OAuth 2.0 Authorization Code   | 認可コード発行         |
| `POST` | `/oauth/token`           | Token issuance (4 grant types) | トークン発行（4 種類） |
| `POST` | `/oauth/revoke`          | Token revocation               | トークン失効           |
| `GET`  | `/oauth/jwks`            | JWK Set                        | JWT 検証用公開鍵セット |
| `POST` | `/oauth/register`        | Register OAuth client          | OAuth クライアント登録 |
| `POST` | `/approval/request`      | Request human approval         | 人間の承認リクエスト   |
| `POST` | `/approval/{id}/approve` | Approve transaction            | 決済承認               |
| `POST` | `/approval/{id}/reject`  | Reject transaction             | 決済拒否               |
| `POST` | `/payment/refund`        | Refund payment                 | 返金                   |
| `GET`  | `/payment/methods`       | List payment methods           | 決済手段一覧           |
| `GET`  | `/health`                | Health check                   | ヘルスチェック         |

### Phase 3 Endpoints — Trust / VC / Fraud

| Method | Path                      | Description                  | 説明                            |
| ------ | ------------------------- | ---------------------------- | ------------------------------- |
| `GET`  | `/trust/:did/score`       | Get latest trust score       | 最新信頼スコア取得              |
| `GET`  | `/trust/:did/history`     | Get score history            | スコア履歴取得                  |
| `POST` | `/trust/:did/recalculate` | Recalculate trust score      | スコア再計算・保存              |
| `POST` | `/vc/issue`               | Issue Verifiable Credential  | VC 発行（Ed25519 署名）         |
| `POST` | `/vc/verify`              | Verify Verifiable Credential | VC 検証（署名・有効期限・失効） |
| `POST` | `/vc/revoke`              | Revoke Verifiable Credential | VC 失効                         |
| `POST` | `/fraud/check`            | Check transaction for fraud  | トランザクション不正チェック    |
| `GET`  | `/fraud/:did/alerts`      | Get fraud alerts             | 不正アラート一覧取得            |

### Phase 4 New Endpoints — A2A / Escrow / Saga / Flow

| Method | Path                   | Description               | 説明                                   |
| ------ | ---------------------- | ------------------------- | -------------------------------------- |
| `POST` | `/a2a/transfer`        | Initiate A2A transfer     | A2A 送金開始（10 ステップ Saga）       |
| `GET`  | `/a2a/transfer/:id`    | Get A2A transfer status   | 送金状態・Saga・エスクロー確認         |
| `POST` | `/escrow/:id/release`  | Release escrow            | エスクロー解放（受信者へ送金）         |
| `POST` | `/escrow/:id/refund`   | Refund escrow             | エスクロー返金（送信者へ返却）         |
| `POST` | `/escrow/:id/dispute`  | Dispute escrow            | 紛争申立（エスクロー凍結）             |
| `GET`  | `/saga/:id/status`     | Get saga status           | Saga 全ステップ状態取得                |
| `POST` | `/saga/:id/complete`   | Report service completion | 受信者がサービス完了を報告             |
| `POST` | `/saga/:id/compensate` | Manual compensation       | 手動補償実行（管理者用）               |
| `POST` | `/flow/configure`      | Configure flow policy     | フロー制御ポリシー設定                 |
| `GET`  | `/flow/:did/health`    | Get flow health           | レート・アクティブ Saga 数・健全性確認 |

### Phase 4 gRPC Endpoints — port 50052

| RPC                | Type             | Description              | 説明                       |
| ------------------ | ---------------- | ------------------------ | -------------------------- |
| `InitiateTransfer` | Unary            | Start A2A transfer       | A2A 送金開始               |
| `CompleteService`  | Unary            | Report completion        | サービス完了報告           |
| `WatchTransfer`    | Server Streaming | Real-time status updates | リアルタイムステータス更新 |
| `DisputeEscrow`    | Unary            | Dispute escrow           | 紛争申立                   |

---

## Error Codes / エラーコード

| Code                       | HTTP | Description / 説明                                                                  |
| -------------------------- | ---- | ----------------------------------------------------------------------------------- |
| `DID_NOT_FOUND`            | 404  | DID does not exist / DID が存在しない                                               |
| `INVALID_SIGNATURE`        | 401  | Ed25519 signature mismatch / 署名が不一致                                           |
| `TOKEN_EXPIRED`            | 401  | JWT has expired / JWT の有効期限切れ                                                |
| `TOKEN_INVALID`            | 401  | JWT is malformed / JWT が無効                                                       |
| `SCOPE_EXCEEDED`           | 403  | Amount exceeds token limit / 金額がスコープ外                                       |
| `DUPLICATE_TRANSACTION`    | 409  | Idempotency key already used / 冪等キー重複                                         |
| `PAYMENT_FAILED`           | 502  | Payment provider error / 決済プロバイダーエラー                                     |
| `CHAIN_INVALID`            | 500  | Hash chain integrity failure / ハッシュチェーン破損                                 |
| `APPROVAL_REQUIRED`        | 202  | High-value tx needs approval / 高額決済には承認が必要                               |
| `AGENT_FROZEN`             | 403  | Agent account frozen / エージェントが凍結済み                                       |
| `RATE_LIMIT_EXCEEDED`      | 429  | Too many requests / レート制限超過                                                  |
| `FLOW_VIOLATION`           | 429  | Flow control limit exceeded / フロー制御違反（レート・ペア・深度）                  |
| `ESCROW_ERROR`             | 400  | Escrow operation failed / エスクロー操作エラー                                      |
| `SAGA_COMPENSATION_FAILED` | 500  | Saga compensation failed — manual intervention required / Saga 補償失敗・要手動介入 |

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
- **A2A escrow holds funds until service is confirmed** — neither party can unilaterally claim funds
- **Saga compensation runs in reverse order** — every step that has side effects defines a rollback action
- **Flow Controller is fail-closed** — if the controller is unavailable, transactions are denied (not allowed)
- **Chain depth BFS** detects and blocks circular payment loops (A→B→C→A)
- **Kafka events are best-effort** — Kafka failure never blocks a payment transaction

**日本語:**

- 秘密鍵は SDK 外に出ません。サーバーには公開鍵のみ送信されます
- JWT スコープによる支払い上限はサーバー側で強制されます
- 冪等キーにより、ネットワーク再試行時でも二重決済を防止します
- 高額決済（`APPROVAL_REQUIRED_ABOVE`以上）は Webhook 経由で人間の承認が必要です
- Stripe への連続失敗 5 回でサーキットブレーカーが自動発動します
- Redis によるレートリミットでエージェントと IP の両レベルで乱用を防止します
- すべてのシークレットは環境変数から読み込みます
- Ed25519 秘密鍵は使用後メモリからゼロ埋めされます（`ZeroizeOnDrop`）
- **A2A エスクローはサービス確認まで資金を仮預かり** — 一方的な資金引き出しは不可
- **Saga の補償は逆順実行** — 副作用のある全ステップが巻き戻しアクションを定義
- **フロー制御はフェイルクローズ** — コントローラー障害時は取引を拒否（通過させない）
- **BFS 循環検知**で A→B→C→A のような決済ループを検知・遮断します
- **Kafka イベントはベストエフォート** — Kafka 障害が決済トランザクションをブロックすることはありません

---

## Roadmap / ロードマップ

| Phase       | Status      | Content                                                                                                                                                                                                                               |
| ----------- | ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **MVP**     | ✅ Done     | FastAPI server, Python SDK, SQLite, Stripe, hash chain audit                                                                                                                                                                          |
| **Phase 1** | ✅ Done     | TypeScript SDK, AutoGen, OpenClaw, MCP server, MkDocs, CI/CD                                                                                                                                                                          |
| **Phase 2** | ✅ Done     | Rust/Axum server, PostgreSQL, Redis, OAuth 2.0, Human-in-the-Loop, Circuit Breaker                                                                                                                                                    |
| **Phase 3** | ✅ Done     | Trust Score Engine, W3C VC Issuer (Ed25519), Fraud Detection (rule engine + gRPC ML), DID Registry on L2 (Solidity/Foundry/Polygon Amoy)                                                                                              |
| **Phase 4** | ✅ **Done** | **A2A Payments** (10-step Saga), **Escrow Service** (Stripe Connect), **Saga Orchestrator** (backward compensation), **Flow Controller** (rate/pair/chain-depth/BFS), **Kafka** event stream, **gRPC Server** (tonic 0.11, streaming) |
| **Phase 5** | 🔜 Planned  | Production deployment (AWS/GCP), monitoring (Prometheus/Grafana), `agenttrust-crypto` crate (PyO3 bindings)                                                                                                                           |

---

## Known Limitations / 既知の制限事項

現在の実装における既知の制限事項です。Phase 5 以降で順次対応予定です。

| Item / 項目                        | Status / 状況 | Detail / 詳細                                                                                                                                                            |
| ---------------------------------- | ------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `dispute_rate` in trust score      | ⚠️ 暫定実装   | 信頼スコアの dispute_rate は現在常に `0.0` が返されます。紛争履歴を反映した実装は Phase 5 で対応予定。                                                                   |
| Kafka トピック                     | ⚠️ 簡略化     | アーキテクチャ図では複数トピックを示していますが、現実装では `agent.trust.events` 単一トピックを使用しています。                                                          |
| gRPC TLS                           | ⚠️ 非対応     | 現在の gRPC サーバー（port 50052）は TLS なし（plaintext）で起動します。本番環境では TLS 設定が必須です。                                                                 |
| Stripe Connect 本番連携            | ⚠️ 未完成     | エスクローの Stripe Connect 実装は構造が定義されていますが、実際の Connect アカウント間送金は未実装です。                                                                 |
| Unit / Integration Tests (Phase 4) | ⚠️ 未整備     | Phase 4 コンポーネント（A2A / Escrow / Saga / Flow）の自動テストは未実装です。                                                                                            |
| JWT 鍵のエフェメラル動作           | ℹ️ 注意       | `JWT_SERVER_PRIVATE_KEY` 未設定時、起動毎に新しい鍵ペアを生成します。**再起動すると全 JWT が無効化されます**。永続化するには stderr 出力された値を `.env` に設定してください。 |

---

## Troubleshooting / トラブルシューティング

| Symptom / 症状                                        | Cause / 原因                               | Fix / 解決策                                                                 |
| ----------------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------- |
| `error: could not find system library 'protobuf'`     | `protoc` 未インストール                    | `brew install protobuf` / `apt install protobuf-compiler`                    |
| `cmake: command not found` (rdkafka ビルドエラー)     | `cmake` 未インストール                     | `brew install cmake` / `apt install cmake`                                   |
| `DATABASE_URL must be set`                            | `.env` が未作成または未設定               | `cp .env.example .env` して `DATABASE_URL` を設定                            |
| `migration failed: relation "..." does not exist`     | マイグレーション未実行                     | `sqlx migrate run`（`sqlx-cli` 要インストール）                              |
| JWT が再起動後に無効になる                            | 鍵未設定でエフェメラル動作中              | 起動時 stderr の `JWT_SERVER_PRIVATE_KEY=...` を `.env` に設定               |
| `FLOW_VIOLATION 429` が常に返る                       | Redis 未起動またはフローポリシーが厳しい  | Redis 起動確認 / `POST /flow/configure` でレート制限を緩和                   |
| gRPC 接続拒否（port 50052）                           | gRPC サーバーが TLS を要求               | `grpcurl -plaintext ...`（`-plaintext` フラグが必要）                        |
| 初回 Docker ビルドが途中で停止                        | メモリ/ディスク不足（Rust ビルドは重い）  | Docker Desktop のメモリを 8GB 以上に設定 / ディスク空き容量を確認            |
| `cargo test` で DB 関連テストが失敗                   | テスト用 DB が未起動                       | `TEST_SERVER_URL=http://localhost:8000 cargo test`（サーバー起動後に実行）   |

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

## Disclaimer / 免責事項

**English:**

AgentTrust Protocol is a **research and prototype project** intended for educational and experimental purposes. By using this software, you agree to the following:

1. **Not Production-Ready Financial Software** — This project has not been audited for production use. Do not use it to process real financial transactions without conducting your own comprehensive security review.
2. **No Financial Liability** — The authors and contributors accept no responsibility for any financial losses, failed transactions, double charges, or payment errors arising from the use of this software.
3. **Unaudited Smart Contracts** — The Solidity smart contracts (`DIDRegistry.sol`) have not been formally audited. Deploying unaudited contracts to mainnet or any network holding real funds is strongly discouraged.
4. **Experimental AI Agent Payments** — Granting AI agents the ability to execute real-world payments carries inherent risk. You are solely responsible for configuring appropriate spending limits and human-in-the-loop controls.
5. **No Regulatory Compliance Guarantee** — This software does not guarantee compliance with financial regulations (PCI-DSS, GDPR, FATF, etc.) in your jurisdiction. Consult a legal professional before deploying in any regulated environment.
6. **Cryptographic Keys** — Loss of an Ed25519 private key results in permanent loss of the associated agent identity. There is no key recovery mechanism.
7. **Third-Party Services** — This project integrates with Stripe, Polygon, and other third-party services under their respective terms of service. The authors are not affiliated with these providers.
8. **Use at Your Own Risk** — This software is provided "AS IS," without warranty of any kind. See the [MIT License](LICENSE) for full terms.

**日本語:**

AgentTrust Protocol は**教育・実験目的の研究プロトタイプ**です。本ソフトウェアを使用することで、以下の事項に同意したものとみなします。

1. **本番環境向け金融ソフトウェアではありません** — 本プロジェクトは本番利用のための監査を受けていません。独自のセキュリティレビューを実施せずに実際の金融取引に使用しないでください。
2. **金融上の損失に関する免責** — 本ソフトウェアの使用によって生じた金銭的損失、取引の失敗、二重請求、決済エラーなどについて、著作者および貢献者は一切の責任を負いません。
3. **未監査スマートコントラクト** — Solidity スマートコントラクト（`DIDRegistry.sol`）は正式なセキュリティ監査を受けていません。実資金を扱うネットワークへの未監査コントラクトのデプロイは強く非推奨とします。
4. **AI エージェント決済の固有リスク** — AI エージェントに実際の決済権限を付与することには固有のリスクが伴います。適切な支払い上限と Human-in-the-Loop 制御の設定は利用者の責任において行ってください。
5. **法規制コンプライアンスの非保証** — 本ソフトウェアは、お使いの地域における金融規制（PCI-DSS・GDPR・FATF 等）への準拠を保証しません。規制対象の環境でのデプロイ前に法律の専門家にご相談ください。
6. **暗号鍵の管理** — Ed25519 秘密鍵を紛失した場合、対応するエージェント ID は永久に失われます。鍵の復元機能はありません。
7. **サードパーティサービス** — 本プロジェクトは Stripe・Polygon 等のサードパーティサービスをそれぞれの利用規約に基づき利用します。著作者はこれらのプロバイダーとは無関係です。
8. **自己責任での使用** — 本ソフトウェアは「現状のまま（AS IS）」提供され、いかなる種類の保証もありません。詳細は[MIT ライセンス](LICENSE)をご参照ください。

---

## License / ライセンス

MIT License — see [LICENSE](LICENSE) for details.

---

## Links / リンク

- 📖 **Documentation**: https://momo1235656.github.io/agenttrust-protocol/
- 🐛 **Issues**: https://github.com/momo1235656/agenttrust-protocol/issues
- 💬 **Discussions**: https://github.com/momo1235656/agenttrust-protocol/discussions
