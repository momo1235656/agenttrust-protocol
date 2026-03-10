"""MCPサーバーの設定。"""
import os


class MCPConfig:
    """環境変数からMCPサーバーの設定を読み込む。"""

    server_url: str = os.environ.get("AGENTTRUST_SERVER_URL", "http://localhost:8000")
    did: str | None = os.environ.get("AGENTTRUST_DID")
    private_key_base64: str | None = os.environ.get("AGENTTRUST_PRIVATE_KEY_BASE64")

    # 金額の上下限（fail-closed原則）
    min_amount: int = 1
    max_amount: int = 1_000_000


config = MCPConfig()
