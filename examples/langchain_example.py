"""LangChainでAgentTrustを使うサンプル"""
import asyncio
from sdk.wallet import AgentWallet
from sdk.tools import PaymentTool


async def main():
    # 1. ウォレット作成
    wallet = AgentWallet(server_url="http://localhost:8000")
    await wallet.create(display_name="langchain-demo", max_limit=100000)
    print(f"DID: {wallet.did}")

    # 2. LangChainツールとして使用
    tool = PaymentTool(wallet=wallet)
    result = await tool._arun(amount=3000, description="LangChainからのテスト決済")
    print(result)

    # LangChainエージェントに組み込む場合:
    # from langchain.agents import create_tool_calling_agent, AgentExecutor
    # from langchain_openai import ChatOpenAI
    #
    # llm = ChatOpenAI(model="gpt-4o")
    # tools = [tool]
    # agent = create_tool_calling_agent(llm, tools, prompt)
    # executor = AgentExecutor(agent=agent, tools=tools)
    # executor.invoke({"input": "3000円の商品を購入してください"})


if __name__ == "__main__":
    asyncio.run(main())
