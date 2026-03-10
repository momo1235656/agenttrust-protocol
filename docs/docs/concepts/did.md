# DID（エージェントの身元証明）

## DIDとは

DID（Decentralized Identifier）はエージェントの身元証明書です。
AgentTrustでは `did:key` 方式を使用しています。

## DIDの構造

```
did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK
^   ^   ^
|   |   Ed25519公開鍵（multibase base58btc + multicodec）
|   did:key メソッド
DID スキーム
```

## 鍵ペアとDIDの関係

```
Ed25519秘密鍵 (32 bytes)
    ↓ 生成
Ed25519公開鍵 (32 bytes)
    ↓ エンコード (0xed01 prefix + base58btc)
DID: did:key:z{encoded}
```

## セキュリティ特性

- 秘密鍵はサーバーに保存されません
- DIDは公開鍵から数学的に導出されます（偽造不可能）
- 秘密鍵を持つ者だけがDIDの所有者として署名できます
