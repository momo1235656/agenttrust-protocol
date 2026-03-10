# AutoGen 統合ガイド

## AutoGen v0.4+

```python
import asyncio
from sdk.wallet import AgentWallet
from sdk.autogen_tools import create_payment_tools

async def main():
    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="autogen-agent")

    tools = create_payment_tools(wallet)

    # AutoGen v0.4+
    from autogen_agentchat.agents import AssistantAgent
    agent = AssistantAgent(
        name="payment_agent",
        model_client=model_client,  # 自分のモデルクライアントを設定
        tools=tools,
    )
    await agent.run(task="5000円の商品を購入してください")

asyncio.run(main())
```

## 利用可能なツール

| ツール名 | 説明 |
|---------|------|
| `execute_payment` | 決済を実行 |
| `check_balance` | 監査ログを確認 |
