# OpenClaw 統合ガイド

```python
import asyncio
from sdk.wallet import AgentWallet
from sdk.openclaw_tools import AgentTrustPaymentAction, AgentTrustAuditAction

async def main():
    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="openclaw-agent")

    actions = [
        AgentTrustPaymentAction(wallet=wallet),
        AgentTrustAuditAction(wallet=wallet),
    ]

    # 直接実行テスト
    result = await actions[0].execute({
        "amount": 5000,
        "description": "テスト決済"
    })
    print(result)

asyncio.run(main())
```

## アクション一覧

| アクション | 説明 |
|-----------|------|
| `AgentTrustPaymentAction` | 決済実行 |
| `AgentTrustAuditAction` | 監査ログ確認 |
