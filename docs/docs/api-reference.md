# API リファレンス

全9エンドポイントのリファレンスです。ベースURL: `http://localhost:8000`

## DID

### POST /did/create

新しいエージェントDIDを作成します。

```bash
curl -X POST http://localhost:8000/did/create \
  -H "Content-Type: application/json" \
  -d '{
    "display_name": "my-agent",
    "max_transaction_limit": 50000,
    "allowed_categories": ["electronics", "software"]
  }'
```

**レスポンス (201)**:
```json
{
  "did": "did:key:z6Mkf5rGN8Hv...",
  "document": { "...": "..." },
  "private_key_base64": "..."
}
```

### GET /did/resolve/{did}

DIDドキュメントを取得します。

```bash
curl http://localhost:8000/did/resolve/did:key:z6Mkf5rGN8Hv...
```

### POST /did/verify

署名を検証します。

```bash
curl -X POST http://localhost:8000/did/verify \
  -H "Content-Type: application/json" \
  -d '{
    "did": "did:key:z6Mkf5rGN8Hv...",
    "message": "base64encodedmessage",
    "signature": "base64eddsignature"
  }'
```

## 認証

### POST /auth/token

スコープ付きJWTを発行します。

```bash
curl -X POST http://localhost:8000/auth/token \
  -H "Content-Type: application/json" \
  -d '{
    "did": "did:key:z6Mkf5rGN8Hv...",
    "message": "auth_request_1710000000",
    "signature": "base64eddsignature",
    "requested_scopes": ["payment:execute"]
  }'
```

### POST /auth/verify-token

JWTを検証します。

## 決済

### POST /payment/execute

決済を実行します。

```bash
curl -X POST http://localhost:8000/payment/execute \
  -H "Authorization: Bearer eyJhbGci..." \
  -H "Content-Type: application/json" \
  -d '{
    "amount": 5000,
    "currency": "jpy",
    "description": "テスト商品の購入",
    "idempotency_key": "unique-key-001"
  }'
```

### GET /payment/{transaction_id}

決済ステータスを確認します。

## 監査

### GET /audit/{agent_did}

監査ハッシュチェーンを取得します。

### POST /audit/verify

ハッシュチェーンの整合性を検証します。

## エラーコード

| コード | HTTP | 説明 |
|--------|------|------|
| `DID_NOT_FOUND` | 404 | DIDが存在しない |
| `INVALID_SIGNATURE` | 401 | 署名が無効 |
| `TOKEN_EXPIRED` | 401 | JWTの有効期限切れ |
| `TOKEN_INVALID` | 401 | JWTが無効 |
| `SCOPE_EXCEEDED` | 403 | 金額がスコープ外 |
| `DUPLICATE_TRANSACTION` | 409 | 冪等キーが重複 |
| `PAYMENT_FAILED` | 502 | Stripe APIエラー |
