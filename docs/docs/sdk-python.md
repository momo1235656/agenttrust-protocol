# Python SDK

## インストール

```bash
pip install -e ".[dev]"
```

## AgentWallet

```python
from sdk.wallet import AgentWallet

# 新規エージェント
wallet = AgentWallet(server_url="http://localhost:8000")
await wallet.create(display_name="my-agent", max_limit=50000)

# 既存エージェント
import base64
wallet = AgentWallet(
    server_url="http://localhost:8000",
    did="did:key:z6Mk...",
    private_key=base64.b64decode("your-private-key-base64"),
)
```

## 決済

```python
result = await wallet.pay(
    amount=5000,
    description="商品名",
    idempotency_key="unique-key-001",  # 省略可（自動生成）
)
print(result["transaction_id"])
```

## LangChain統合

```python
from sdk.tools import PaymentTool

tool = PaymentTool(wallet=wallet)
# LangChainのagentのtool listに追加するだけ
```

## AutoGen統合

```python
from sdk.autogen_tools import create_payment_tools

tools = create_payment_tools(wallet)
# AssistantAgentのtools引数に渡す
```

## OpenClaw統合

```python
from sdk.openclaw_tools import AgentTrustPaymentAction, AgentTrustAuditAction

actions = [
    AgentTrustPaymentAction(wallet=wallet),
    AgentTrustAuditAction(wallet=wallet),
]
```
