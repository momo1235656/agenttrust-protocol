# 監査ログ（ハッシュチェーン）

## 仕組み

各取引は前の取引のハッシュを含む「ハッシュチェーン」として記録されます。

```
取引 #0: hash = SHA256(0 + tx_001 + 3000 + succeeded + timestamp + "0"×64)
取引 #1: hash = SHA256(1 + tx_002 + 5000 + succeeded + timestamp + hash_0)
取引 #2: hash = SHA256(2 + tx_003 + 2000 + failed    + timestamp + hash_1)
```

## 改ざん検知

過去の取引を変更すると、それ以降の全ハッシュが変わり、検証で検知されます。

## 検証API

```bash
curl -X POST http://localhost:8000/audit/verify \
  -H "Content-Type: application/json" \
  -d '{"agent_did": "did:key:z6Mk..."}'
```

```json
{
  "chain_valid": true,
  "total_entries": 15,
  "verified_at": "2026-03-11T12:00:00Z"
}
```
