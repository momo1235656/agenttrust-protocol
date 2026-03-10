# AgentTrust TypeScript SDK

TypeScript/Node.js向けAgentTrust Protocol SDK。AI エージェントに決済機能を追加します。

## インストール

```bash
npm install agenttrust
```

## クイックスタート

```typescript
import { AgentWallet } from 'agenttrust';

const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
await wallet.create({ displayName: 'my-agent', maxLimit: 50000 });

console.log(`DID: ${wallet.did}`);

const result = await wallet.pay({
  amount: 5000,
  description: 'テスト商品の購入',
});
console.log(`決済完了: ${result.transaction_id}`);
```

## 対応環境

- Node.js 18+
- Deno 1.38+
- Bun 1.0+

## API

### `new AgentWallet(config?)`

| パラメータ | 型 | 説明 |
|----------|-----|------|
| `config.serverUrl` | `string` | AgentTrustサーバーURL（デフォルト: `http://localhost:8000`） |
| `config.did` | `string` | 既存のDID（再利用時） |
| `config.privateKey` | `Uint8Array` | 既存の秘密鍵（再利用時） |

### `wallet.create(options?)`

新しいDIDを作成してサーバーに登録します。

### `wallet.pay(options)`

決済を実行します。

| パラメータ | 型 | 説明 |
|----------|-----|------|
| `options.amount` | `number` | 決済金額（円） |
| `options.description` | `string?` | 決済の説明 |
| `options.idempotencyKey` | `string?` | 冪等キー |

### `wallet.getAuditLog()`

監査ログを取得します。
