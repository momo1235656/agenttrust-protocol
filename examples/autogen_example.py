"""AutoGenでAgentTrustを使うサンプル"""
import asyncio
from sdk.wallet import AgentWallet
from sdk.autogen_tools import create_payment_tools


async def main():
    # 1. ウォレット作成
    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="autogen-demo", max_limit=100000)
    print(f"DID: {wallet.did}")

    # 2. ツール作成
    tools = create_payment_tools(wallet)
    print(f"登録されたツール数: {len(tools)}")

    # 3. ツール単体テスト
    result = await tools[0](amount=5000, description="AutoGenからのテスト決済")
    print(result)

    # AutoGen v0.4+ に組み込む場合:
    # from autogen_agentchat.agents import AssistantAgent
    # from autogen_ext.models import OpenAIChatCompletionClient
    #
    # model_client = OpenAIChatCompletionClient(model="gpt-4o")
    # agent = AssistantAgent(
    #     name="payment_agent",
    #     model_client=model_client,
    #     tools=tools,
    # )
    # await agent.run(task="5000円の商品を購入してください")


if __name__ == "__main__":
    asyncio.run(main())
