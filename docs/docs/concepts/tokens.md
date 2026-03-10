# トークンとスコープ

## スコープ付きJWT

AgentTrustのJWTには決済権限が埋め込まれています。

```json
{
  "sub": "did:key:z6Mk...",
  "scopes": ["payment:execute", "balance:read"],
  "max_amount": 50000,
  "currency": "jpy",
  "allowed_categories": ["electronics"],
  "iat": 1710000000,
  "exp": 1710001800
}
```

## スコープ一覧

| スコープ | 説明 |
|---------|------|
| `payment:execute` | 決済の実行 |
| `payment:read` | 決済履歴の参照 |
| `balance:read` | 残高・監査ログの参照 |
| `audit:read` | 監査ログの参照 |

## トークンの有効期限

- デフォルト: **30分**
- SDK側でキャッシュし、期限前60秒で自動更新
