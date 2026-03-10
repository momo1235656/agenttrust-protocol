"""demo.py — AgentTrust MVP 動作確認"""
import asyncio
from sdk.wallet import AgentWallet

try:
    from sdk.tools import PaymentTool, LANGCHAIN_AVAILABLE
except ImportError:
    LANGCHAIN_AVAILABLE = False


async def main():
    # 1. ウォレット作成（DID自動生成）
    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="demo-agent", max_limit=50000)
    print(f"DID: {wallet.did}")

    # 2. 決済実行
    result = await wallet.pay(amount=5000, description="テスト商品の購入")
    print(f"決済結果: {result}")

    # 3. LangChainツールとしての利用
    if LANGCHAIN_AVAILABLE:
        tool = PaymentTool(wallet=wallet)
        result = await tool._arun(amount=3000, description="もう一つの商品")
        print(f"ツール結果: {result}")
    else:
        print("LangChain未インストールのためツールテストをスキップ")

    await wallet.close()


if __name__ == "__main__":
    asyncio.run(main())
