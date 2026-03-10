# TypeScript SDK

## インストール

```bash
npm install agenttrust
```

## AgentWallet

```typescript
import { AgentWallet } from 'agenttrust';

// 新規エージェント
const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
await wallet.create({ displayName: 'my-agent', maxLimit: 50000 });

// 既存エージェント
const wallet = new AgentWallet({
  serverUrl: 'http://localhost:8000',
  did: 'did:key:z6Mk...',
  privateKey: Uint8Array.from(Buffer.from('your-private-key-base64', 'base64')),
});
```

## 決済

```typescript
const result = await wallet.pay({
  amount: 5000,
  description: '商品名',
  idempotencyKey: 'unique-key-001', // 省略可（自動生成）
});
console.log(result.transaction_id);
```

## 対応環境

- **Node.js** 18+: `import { AgentWallet } from 'agenttrust'`
- **Deno**: `import { AgentWallet } from 'npm:agenttrust'`
- **Bun**: `import { AgentWallet } from 'agenttrust'`

## エラーハンドリング

```typescript
import { AgentTrustError } from 'agenttrust';

try {
  await wallet.pay({ amount: 999999 });
} catch (e) {
  if (e instanceof AgentTrustError) {
    console.error(`エラーコード: ${e.code}`); // "SCOPE_EXCEEDED"
    console.error(`HTTPステータス: ${e.statusCode}`); // 403
  }
}
```
