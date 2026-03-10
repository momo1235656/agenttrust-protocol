# AgentTrust Protocol

**AI エージェントのための安全な決済インフラ**

AgentTrustは、AIエージェントが安全に決済を行うためのインフラです。

## なぜ AgentTrust が必要か

AIエージェントには「身分証明書」も「銀行口座」も「信用情報」もありません。
AgentTrustは以下を提供します：

- **DID（分散型ID）** — エージェントの身元証明
- **スコープ付きトークン** — 決済権限の精密な制御
- **ハッシュチェーン** — 改ざん不能な監査ログ

## 主な機能

| 機能 | 説明 |
|------|------|
| DID発行 | Ed25519鍵ペアでエージェントにDIDを付与 |
| スコープ付きJWT | 金額上限・カテゴリ制限付きの決済トークン |
| Stripe連携 | テスト/本番モードの決済実行 |
| 監査ログ | SHA-256ハッシュチェーンで取引を記録 |

## クイックスタート

=== "Python"

    ```python
    from sdk.wallet import AgentWallet

    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="my-agent")
    result = await wallet.pay(amount=5000, description="テスト購入")
    print(result)
    ```

=== "TypeScript"

    ```typescript
    import { AgentWallet } from 'agenttrust';

    const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
    await wallet.create({ displayName: 'my-agent' });
    const result = await wallet.pay({ amount: 5000, description: 'テスト購入' });
    console.log(result);
    ```

## 対応フレームワーク

- **LangChain** / **LangChain.js**
- **AutoGen** (v0.4+)
- **OpenClaw**
- **MCP** (Claude Desktop, Cursor等)
