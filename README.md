# AgentTrust Protocol

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

```
┌─────────────────────────────────────────────────────────┐
│                    AI Agent (your code)                  │
│           LangChain / AutoGen / OpenClaw / MCP           │
└──────────────────────┬──────────────────────────────────┘
                       │ SDK
          ┌────────────┼────────────┐
          │            │            │
    Python SDK   TypeScript SDK   MCP Server
    (sdk/)       (sdk-ts/)        (mcp-server/)
          │            │            │
          └────────────┼────────────┘
                       │ HTTP REST API
┌──────────────────────▼──────────────────────────────────┐
│               AgentTrust Server (FastAPI)                │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌───────────┐  │
│  │  DID API │ │ Auth API │ │  Pay API │ │ Audit API │  │
│  └──────────┘ └──────────┘ └──────────┘ └───────────┘  │
│  ┌──────────────────────┐  ┌──────────────────────────┐ │
│  │  SQLite (agents,     │  │  Ed25519 Crypto          │ │
│  │  transactions,       │  │  SHA-256 Hash Chain      │ │
│  │  audit_logs)         │  │  Stripe Integration      │ │
│  └──────────────────────┘  └──────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

---

## Tech Stack / 技術スタック

| Layer / 層 | Technology / 技術 | Reason / 採用理由 |
|-----------|-------------------|-----------------|
| API Server | FastAPI + Uvicorn | Async, auto Swagger UI |
| Cryptography | PyNaCl (Ed25519) | Industry-standard signatures |
| JWT | PyJWT + cryptography | Ed25519-signed tokens |
| Database | SQLite + SQLAlchemy (async) | Zero-config, production-ready schema |
| Payment | Stripe Python SDK | Test mode available |
| Python SDK | Pure Python + httpx | No heavy dependencies |
| TypeScript SDK | @noble/ed25519 + fetch | Zero dependencies, runs everywhere |
| MCP Server | Anthropic MCP SDK | Claude Desktop / Cursor integration |
| Tests | pytest-asyncio + vitest | Full async test coverage |

---

## Project Structure / プロジェクト構造

```
agenttrust-protocol/
│
├── server/                   # FastAPI サーバー（変更頻度: 低）
│   ├── main.py               # アプリ定義・ルーター登録
│   ├── config.py             # 環境変数設定
│   ├── dependencies.py       # DB セッション DI
│   ├── routers/              # API エンドポイント
│   │   ├── did.py            #   POST /did/create, GET /did/resolve/{did}
│   │   ├── auth.py           #   POST /auth/token
│   │   ├── payment.py        #   POST /payment/execute
│   │   └── audit.py          #   GET /audit/{did}
│   ├── services/             # ビジネスロジック
│   │   ├── did_service.py
│   │   ├── auth_service.py
│   │   ├── payment_service.py
│   │   └── audit_service.py
│   ├── models/               # SQLAlchemy モデル
│   ├── schemas/              # Pydantic スキーマ
│   └── crypto/               # 暗号処理（将来 Rust 移行予定）
│       ├── keys.py           #   Ed25519 鍵ペア生成・DID 導出
│       ├── signing.py        #   署名・検証
│       └── hashing.py        #   SHA-256 ハッシュチェーン
│
├── sdk/                      # Python SDK
│   ├── wallet.py             # AgentWallet（メインクラス）
│   ├── tools.py              # LangChain BaseTool 統合
│   ├── autogen_tools.py      # AutoGen v0.4+ ツール
│   ├── openclaw_tools.py     # OpenClaw アクション
│   └── client.py             # HTTP クライアント
│
├── sdk-ts/                   # TypeScript SDK
│   ├── src/
│   │   ├── wallet.ts         # AgentWallet（TS 版）
│   │   ├── client.ts         # fetch ベース HTTP クライアント
│   │   ├── crypto.ts         # @noble/ed25519 ラッパー
│   │   └── types.ts          # 全型定義
│   └── tests/                # vitest テスト
│
├── mcp-server/               # MCP サーバー
│   └── server.py             # Claude Desktop / Cursor 対応
│
├── tests/                    # Python テストスイート
├── examples/                 # フレームワーク別サンプル
├── docs/                     # MkDocs ドキュメントサイト
├── .github/workflows/        # CI/CD パイプライン
├── data/dids/                # DID ドキュメント JSON ストア
└── demo.py                   # E2E デモスクリプト
```

---

## Quick Start / クイックスタート

### Prerequisites / 前提条件

- Python 3.11+
- Node.js 18+（TypeScript SDK を使う場合）
- Stripe アカウント（テストモードの API キー）

### 1. Clone & Install / クローンとインストール

```bash
git clone https://github.com/momo1235656/agenttrust-protocol.git
cd agenttrust-protocol

# Python 依存関係
pip install -e ".[dev]"
```

### 2. Configure / 環境変数の設定

```bash
cp .env.example .env
```

`.env` を編集：

```env
# Stripe テストキー（sk_test_ で始まるもの）
STRIPE_SECRET_KEY=sk_test_your_key_here

# データベースパス（デフォルトのままで OK）
DATABASE_URL=sqlite+aiosqlite:///./data/agenttrust.db
```

### 3. Start Server / サーバー起動

```bash
uvicorn server.main:app --reload --port 8000
```

Swagger UI: http://localhost:8000/docs

### 4. Run Demo / デモ実行

```bash
# 別ターミナルで
python demo.py
```

```
DID: did:key:z6MkhaXgBZDvot...
決済結果: {'transaction_id': 'tx_a1b2c3...', 'status': 'succeeded', 'amount': 5000, ...}
ツール結果: 決済完了: tx_d4e5f6..., 3000円, ステータス: succeeded
```

### 5. Run Tests / テスト実行

```bash
# Python テスト
pytest tests/ -v

# TypeScript テスト
cd sdk-ts && npm install && npm test
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

# DID と秘密鍵を保存しておけば、ウォレットを再作成できる
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
  // 1. ウォレット作成
  const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
  await wallet.create({ displayName: 'ts-agent', maxLimit: 50000 });
  console.log(`DID: ${wallet.did}`);

  // 2. 決済実行
  const result = await wallet.pay({
    amount: 3000,
    description: '商品の購入',
    idempotencyKey: 'order-20260311-001',
  });
  console.log(`Status: ${result.status}`);

  // 3. 監査ログ確認
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

# LangChain ツールとして登録するだけ
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

Claude Desktop を再起動すると「5000円の決済をして」等の指示で決済が実行されます。

---

## API Endpoints / API エンドポイント一覧

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
| `GET` | `/health` | Health check | ヘルスチェック |

Interactive docs / インタラクティブドキュメント: **http://localhost:8000/docs**

---

## Error Codes / エラーコード

| Code | HTTP | Description / 説明 |
|------|------|-------------------|
| `DID_NOT_FOUND` | 404 | DID does not exist / DID が存在しない |
| `INVALID_SIGNATURE` | 401 | Ed25519 signature mismatch / 署名が不一致 |
| `TOKEN_EXPIRED` | 401 | JWT has expired / JWT の有効期限切れ |
| `TOKEN_INVALID` | 401 | JWT is malformed / JWT が無効 |
| `SCOPE_EXCEEDED` | 403 | Amount exceeds token limit / 金額がスコープ外 |
| `DUPLICATE_TRANSACTION` | 409 | Idempotency key already used / 冪等キーが重複 |
| `PAYMENT_FAILED` | 502 | Stripe returned an error / Stripe API エラー |
| `CHAIN_INVALID` | 500 | Hash chain integrity failure / ハッシュチェーン破損 |

---

## Security Design / セキュリティ設計

**English:**
- Private keys never leave the SDK — only the public key is sent to the server
- JWT scopes enforce spending limits server-side (not just client-side)
- Idempotency keys prevent duplicate payments even under network retries
- The MCP server applies an additional amount cap (¥1–¥1,000,000) independent of the JWT
- All secrets are loaded from environment variables — nothing is hardcoded

**日本語:**
- 秘密鍵はSDK外に出ません。サーバーには公開鍵のみ送信されます
- JWTスコープによる支払い上限はサーバー側で強制されます（クライアント側だけでなく）
- 冪等キーにより、ネットワーク再試行時でも二重決済を防止します
- MCPサーバーはJWTとは独立して金額上限（1円〜100万円）をチェックします（fail-closed原則）
- すべてのシークレットは環境変数から読み込みます。コードへのハードコードはありません

---

## Roadmap / ロードマップ

| Phase | Status | Content |
|-------|--------|---------|
| **MVP** | ✅ Done | FastAPI server, Python SDK, SQLite, Stripe, hash chain audit |
| **Phase 1** | ✅ Done | TypeScript SDK, AutoGen, OpenClaw, MCP server, MkDocs, CI/CD |
| **Phase 2** | 🔜 Planned | Rust crypto module (PyO3), 10x performance improvement |
| **Phase 3** | 🔜 Planned | OAuth 2.0 Authorization Code flow |
| **Phase 4** | 🔜 Planned | On-chain DID anchoring (Ethereum / Solana) |
| **Phase 5** | 🔜 Planned | Multi-currency, PayPay / Apple Pay support |

---

## Contributing / コントリビュート

```bash
# フォークしてブランチを作成
git checkout -b feature/your-feature

# 変更してテスト
pytest tests/ -v
cd sdk-ts && npm test

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
