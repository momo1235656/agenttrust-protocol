# クイックスタート

5分でAgentTrustを動かします。

## 1. サーバーを起動

```bash
git clone https://github.com/momo1235656/agenttrust-protocol
cd agenttrust-protocol
pip install -e ".[dev]"
cp .env.example .env
# .env に STRIPE_SECRET_KEY=sk_test_... を設定
uvicorn server.main:app --reload --port 8000
```

## 2. SDKを使って決済

=== "Python"

    ```python
    import asyncio
    from sdk.wallet import AgentWallet

    async def main():
        # ウォレット作成（DID自動生成）
        wallet = AgentWallet(server_url="http://localhost:8000")
        await wallet.create(display_name="my-first-agent", max_limit=50000)
        print(f"DID: {wallet.did}")

        # 決済実行
        result = await wallet.pay(
            amount=5000,
            description="テスト商品の購入"
        )
        print(f"決済完了: {result['transaction_id']}")

        # 監査ログ確認
        audit = await wallet.get_audit_log()
        print(f"チェーン整合性: {audit['chain_valid']}")

    asyncio.run(main())
    ```

=== "TypeScript"

    ```typescript
    import { AgentWallet } from 'agenttrust';

    async function main() {
        const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
        await wallet.create({ displayName: 'my-first-agent', maxLimit: 50000 });
        console.log(`DID: ${wallet.did}`);

        const result = await wallet.pay({
            amount: 5000,
            description: 'テスト商品の購入',
        });
        console.log(`決済完了: ${result.transaction_id}`);

        const audit = await wallet.getAuditLog();
        console.log(`チェーン整合性: ${audit.chain_valid}`);
    }

    main().catch(console.error);
    ```

## 3. Swagger UIで確認

ブラウザで `http://localhost:8000/docs` を開くと、全APIエンドポイントを確認できます。
