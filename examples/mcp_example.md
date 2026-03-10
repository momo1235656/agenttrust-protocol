# MCP統合ガイド

## Claude Desktopで使う場合

### 1. AgentTrustサーバーを起動

```bash
uvicorn server.main:app --port 8000
```

### 2. 必要なパッケージをインストール

```bash
pip install mcp
```

### 3. `claude_desktop_config.json` に以下を追加

macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`

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

### 4. 既存のDIDを使う場合

```json
{
  "mcpServers": {
    "agenttrust": {
      "command": "python",
      "args": ["-m", "mcp_server.server"],
      "cwd": "/path/to/agenttrust-protocol",
      "env": {
        "AGENTTRUST_SERVER_URL": "http://localhost:8000",
        "AGENTTRUST_DID": "did:key:z6Mk...",
        "AGENTTRUST_PRIVATE_KEY_BASE64": "your-base64-encoded-private-key"
      }
    }
  }
}
```

### 5. Claude Desktopを再起動

Claude Desktopを再起動すると、以下のような指示で決済が実行できます：

- 「5000円の決済をして」
- 「取引履歴を見せて」
- 「監査ログを確認して」

## Cursorで使う場合

Cursorの設定ファイル（`.cursor/mcp.json`）に同様の設定を追加してください。

## 利用可能なMCPツール

| ツール名 | 説明 | 引数 |
|---------|------|------|
| `payment_execute` | 決済実行 | `amount` (必須), `description` |
| `balance_check` | 取引履歴確認 | なし |
| `audit_verify` | ハッシュチェーン検証 | なし |
