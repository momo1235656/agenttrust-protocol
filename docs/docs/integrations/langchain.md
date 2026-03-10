# LangChain 統合ガイド

## Python (LangChain)

```python
import asyncio
from sdk.wallet import AgentWallet
from sdk.tools import PaymentTool

async def setup():
    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="langchain-agent", max_limit=100000)
    return PaymentTool(wallet=wallet)

tool = asyncio.run(setup())

# LangChainエージェントのツールリストに追加
tools = [tool]
```

## ツールの説明

LLMはツールの `description` を使って決済のタイミングを判断します：

> AIエージェントとして決済を実行します。金額（円）と説明を指定してください。
