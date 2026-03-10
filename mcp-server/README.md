# AgentTrust MCP Server

AgentTrustの決済機能をMCP対応エージェント（Claude Desktop, Cursor等）から利用するためのMCPサーバー。

## インストール

```bash
pip install -e ".[dev]"
pip install mcp
```

## 起動方法

```bash
python -m mcp_server.server
```

## Claude Desktop設定

`claude_desktop_config.json` に以下を追加:

```json
{
  "mcpServers": {
    "agenttrust": {
      "command": "python",
      "args": ["-m", "mcp_server.server"],
      "env": {
        "AGENTTRUST_SERVER_URL": "http://localhost:8000"
      }
    }
  }
}
```

既存のDIDを使う場合:
```json
{
  "mcpServers": {
    "agenttrust": {
      "command": "python",
      "args": ["-m", "mcp_server.server"],
      "env": {
        "AGENTTRUST_SERVER_URL": "http://localhost:8000",
        "AGENTTRUST_DID": "did:key:z6Mk...",
        "AGENTTRUST_PRIVATE_KEY_BASE64": "your-base64-private-key"
      }
    }
  }
}
```

## 利用可能なツール

| ツール名 | 説明 |
|---------|------|
| `payment_execute` | 決済を実行（金額・説明を指定） |
| `balance_check` | 取引履歴・成功率・監査整合性を確認 |
| `audit_verify` | ハッシュチェーンの改ざん検証 |

## セキュリティ

- 金額は1円以上100万円以下のみ受け付け（fail-closed）
- DIDと秘密鍵は環境変数で管理（コードにハードコードしない）
