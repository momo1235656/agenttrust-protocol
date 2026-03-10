"""
AgentTrust MCP Server。
MCP対応のあらゆるエージェント（Claude Desktop, Cursor等）から
AgentTrustの決済機能を利用可能にする。

起動方法:
    python -m mcp_server.server

Claude Desktop設定例（claude_desktop_config.json）:
{
    "mcpServers": {
        "agenttrust": {
            "command": "python",
            "args": ["-m", "mcp_server.server"],
            "env": {
                "AGENTTRUST_SERVER_URL": "http://localhost:8000",
                "AGENTTRUST_DID": "did:key:z6Mk...",
                "AGENTTRUST_PRIVATE_KEY_BASE64": "..."
            }
        }
    }
}
"""
import os
import json
import asyncio
import base64

from mcp.server import Server
from mcp.server.stdio import stdio_server
from mcp.types import Tool, TextContent

from sdk.wallet import AgentWallet
from mcp_server.config import config

app = Server("agenttrust")

# グローバルウォレットインスタンス（遅延初期化）
_wallet: AgentWallet | None = None


async def get_wallet() -> AgentWallet:
    """ウォレットを取得する（初回呼び出し時に初期化）。

    環境変数 AGENTTRUST_DID と AGENTTRUST_PRIVATE_KEY_BASE64 が設定されていれば
    既存のDIDを使用し、なければ新しいDIDを作成する。

    Returns:
        AgentWallet: 初期化済みのウォレット
    """
    global _wallet
    if _wallet is None:
        if config.did and config.private_key_base64:
            private_key = base64.b64decode(config.private_key_base64)
            _wallet = AgentWallet(
                server_url=config.server_url,
                did=config.did,
                private_key=private_key,
            )
        else:
            _wallet = AgentWallet(server_url=config.server_url)
            await _wallet.create(display_name="mcp-agent")
    return _wallet


@app.list_tools()
async def list_tools() -> list[Tool]:
    """利用可能なツール一覧を返す。

    Returns:
        list[Tool]: 3つのMCPツール定義
    """
    return [
        Tool(
            name="payment_execute",
            description="AIエージェントとして決済を実行します。金額（円）と説明を指定してください。",
            inputSchema={
                "type": "object",
                "properties": {
                    "amount": {"type": "integer", "description": "決済金額（円）"},
                    "description": {"type": "string", "description": "決済の説明"},
                },
                "required": ["amount"],
            },
        ),
        Tool(
            name="balance_check",
            description="エージェントの取引履歴、成功率、監査ログの整合性を確認します。",
            inputSchema={
                "type": "object",
                "properties": {},
            },
        ),
        Tool(
            name="audit_verify",
            description="監査ログのハッシュチェーンが改ざんされていないことを検証します。",
            inputSchema={
                "type": "object",
                "properties": {},
            },
        ),
    ]


@app.call_tool()
async def call_tool(name: str, arguments: dict) -> list[TextContent]:
    """ツールを実行する。

    Args:
        name: ツール名
        arguments: ツールの引数

    Returns:
        list[TextContent]: ツールの実行結果
    """
    w = await get_wallet()

    if name == "payment_execute":
        amount = arguments.get("amount", 0)
        description = arguments.get("description", "")

        # fail-closed: 金額の範囲チェック（APIサーバーのチェックとは独立）
        if amount <= 0 or amount > config.max_amount:
            return [TextContent(
                type="text",
                text=f"エラー: 金額は{config.min_amount}円以上{config.max_amount:,}円以下で指定してください。指定された金額: {amount}円"
            )]

        result = await w.pay(amount=amount, description=description)
        return [TextContent(
            type="text",
            text=json.dumps({
                "status": "success",
                "transaction_id": result["transaction_id"],
                "amount": result["amount"],
                "currency": result.get("currency", "jpy"),
                "audit_hash": result.get("audit_hash", ""),
            }, ensure_ascii=False, indent=2)
        )]

    elif name == "balance_check":
        audit = await w.get_audit_log()
        return [TextContent(
            type="text",
            text=json.dumps({
                "agent_did": w.did,
                "total_transactions": audit["total_transactions"],
                "success_rate": audit["success_rate"],
                "chain_valid": audit["chain_valid"],
            }, ensure_ascii=False, indent=2)
        )]

    elif name == "audit_verify":
        audit = await w.get_audit_log()
        return [TextContent(
            type="text",
            text=(
                f"監査ログ検証結果: {'✅ 整合性OK' if audit['chain_valid'] else '❌ 改ざん検知'}\n"
                f"総エントリ数: {audit['total_transactions']}"
            )
        )]

    else:
        return [TextContent(type="text", text=f"エラー: 不明なツール '{name}'")]


async def main():
    """MCPサーバーをstdio transportで起動する。"""
    async with stdio_server() as (read_stream, write_stream):
        await app.run(read_stream, write_stream, app.create_initialization_options())


if __name__ == "__main__":
    asyncio.run(main())
