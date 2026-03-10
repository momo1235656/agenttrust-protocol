# MCP 統合ガイド

AgentTrust MCPサーバーを使うと、Claude Desktop、Cursor等のMCP対応エージェントから直接決済機能を利用できます。

## セットアップ

### 1. AgentTrustサーバーを起動

```bash
uvicorn server.main:app --port 8000
```

### 2. Claude Desktop設定

`~/Library/Application Support/Claude/claude_desktop_config.json` を編集:

```json
{
  "mcpServers": {
    "agenttrust": {
      "command": "python",
      "args": ["-m", "mcp_server.server"],
      "cwd": "/path/to/agenttrust-protocol",
      "env": {
        "AGENTTRUST_SERVER_URL": "http://localhost:8000"
      }
    }
  }
}
```

### 3. Claude Desktopを再起動

Claude Desktopを再起動すると、チャットで「5000円の決済をして」等のリクエストが可能になります。

## 利用可能なツール

| ツール名 | 説明 |
|---------|------|
| `payment_execute` | 決済実行（金額: 1円〜100万円） |
| `balance_check` | 取引履歴・成功率確認 |
| `audit_verify` | ハッシュチェーン整合性検証 |

## セキュリティ

- 金額は1円以上100万円以下のみ受け付け
- DIDと秘密鍵は環境変数で管理
