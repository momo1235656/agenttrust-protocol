"""demo_full_ja.py — AgentTrust Protocol 動作確認デモ（画面録画用）

4つのシナリオを順番に実行します:
  1. DID生成 → 決済成功
  2. 上限超過 → SCOPE_EXCEEDED で拒否される瞬間
  3. 監査チェーン検証（chain_valid: true）
  4. A2Aエスクロー: 開始 → 完了報告 → 解放

実行前提: docker-compose up でサーバーが起動していること（http://localhost:8000）
"""
import asyncio
import time
import httpx
from sdk.wallet import AgentWallet

SERVER_URL = "http://localhost:8000"


def banner(title: str):
    print()
    print("=" * 70)
    print(f"  {title}")
    print("=" * 70)


async def scenario_1_payment_success():
    banner("シナリオ 1/4: DID生成 → 決済成功")

    wallet = AgentWallet(server_url=SERVER_URL)
    await wallet.create(display_name="demo-agent-A", max_limit=50000)
    print(f"DID: {wallet.did}")
    print(f"上限額: 50,000 JPY")

    time.sleep(1)
    print("\n>>> 5,000円の決済を実行します...")
    result = await wallet.pay(amount=5000, description="テスト商品の購入")

    print(f"\n[OK] 決済成功")
    print(f"  status              : {result['status']}")
    print(f"  transaction_id       : {result['transaction_id']}")
    print(f"  stripe_payment_intent: {result['stripe_payment_intent_id']}")
    print(f"  audit_hash           : {result['audit_hash']}")

    return wallet


async def scenario_2_scope_exceeded(wallet: AgentWallet):
    banner("シナリオ 2/4: 上限超過 → SCOPE_EXCEEDED で拒否される瞬間")

    over_limit_amount = 999_999
    print(f"DID: {wallet.did}")
    print(f"上限額: 50,000 JPY")
    print(f"\n>>> 上限を超える {over_limit_amount:,}円の決済を試みます...")
    time.sleep(1)

    try:
        await wallet.pay(amount=over_limit_amount, description="上限超過テスト")
        print("\n[NG] 予期せず決済が成功してしまいました（想定外）")
    except httpx.HTTPStatusError as e:
        body = e.response.json()
        print(f"\n[拒否されました] HTTP {e.response.status_code}")
        print(f"  error.code   : {body['error']['code']}")
        print(f"  error.message: {body['error']['message']}")
        assert body["error"]["code"] == "SCOPE_EXCEEDED", "期待したエラーコードと異なります"
        print("\n>>> 上限制御が正しく機能していることを確認しました。")


async def scenario_3_audit_chain(wallet: AgentWallet):
    banner("シナリオ 3/4: 監査チェーン検証")

    print(f"DID: {wallet.did}")
    print("\n>>> POST /audit/verify で監査ハッシュチェーンを検証します...")
    time.sleep(1)

    async with httpx.AsyncClient() as client:
        resp = await client.post(
            f"{SERVER_URL}/audit/verify",
            json={"agent_did": wallet.did},
        )
        resp.raise_for_status()
        result = resp.json()

    print(f"\n[検証結果]")
    print(f"  chain_valid   : {result['chain_valid']}")
    print(f"  total_entries : {result['total_entries']}")
    print(f"  verified_at   : {result['verified_at']}")
    assert result["chain_valid"] is True, "監査チェーンが不正です"
    print("\n>>> 監査ハッシュチェーンの整合性を確認しました。")


async def scenario_4_a2a_escrow():
    banner("シナリオ 4/4: A2Aエスクロー（開始 → 完了報告 → 解放）")

    # 送金元・受取先の2エージェントを作成
    sender = AgentWallet(server_url=SERVER_URL)
    await sender.create(display_name="demo-sender", max_limit=100000)
    receiver = AgentWallet(server_url=SERVER_URL)
    await receiver.create(display_name="demo-receiver", max_limit=100000)
    print(f"送金元 DID: {sender.did}")
    print(f"受取先 DID: {receiver.did}")

    async with httpx.AsyncClient(timeout=30.0) as client:
        # 信頼スコアを算出（A2A送金には両者のスコアが必要）
        print("\n>>> 両エージェントの信頼スコアを算出します...")
        for wallet in (sender, receiver):
            resp = await client.post(f"{SERVER_URL}/trust/{wallet.did}/recalculate")
            resp.raise_for_status()
            score = resp.json()["new_score"]
            print(f"  {wallet.did[:24]}... score={score}")

        # A2A送金開始（10ステップSagaの起動 + エスクロー資金拘束）
        print("\n>>> A2A送金を開始します（エスクロー資金拘束）...")
        time.sleep(1)
        resp = await client.post(
            f"{SERVER_URL}/a2a/transfer",
            json={
                "sender_did": sender.did,
                "receiver_did": receiver.did,
                "amount": 8000,
                "currency": "jpy",
                "description": "A2Aデモ: サービス提供の対価",
                "service_type": "demo-service",
                "timeout_minutes": 30,
            },
        )
        resp.raise_for_status()
        transfer = resp.json()
        transfer_id = transfer["transfer_id"]
        saga_id = transfer["saga_id"]
        print(f"\n[開始完了]")
        print(f"  transfer_id: {transfer_id}")
        print(f"  saga_id    : {saga_id}")
        print(f"  status     : {transfer['status']}  (escrow: {transfer['escrow_status']})")
        print(f"  進捗       : {transfer['steps']['completed']}/{transfer['steps']['total']} "
              f"(現在: {transfer['steps']['current']})")

        # 受取先がサービス完了を報告 → エスクロー解放が自動実行される
        print("\n>>> 受取先がサービス完了を報告します（エスクロー解放がトリガーされます）...")
        time.sleep(1)
        resp = await client.post(
            f"{SERVER_URL}/saga/{saga_id}/complete",
            json={
                "reporter_did": receiver.did,
                "result_summary": "サービス提供完了",
            },
        )
        resp.raise_for_status()
        complete_result = resp.json()
        print(f"\n[完了報告受理]")
        print(f"  status: {complete_result['status']}")

        # 最終状態を確認
        print("\n>>> 最終状態を確認します...")
        resp = await client.get(f"{SERVER_URL}/a2a/transfer/{transfer_id}")
        resp.raise_for_status()
        final = resp.json()
        print(f"\n[最終結果]")
        print(f"  transfer.status : {final['status']}")
        print(f"  escrow.status   : {final['escrow']['status']}")
        print(f"  saga.status     : {final['saga']['status']}")
        assert final["escrow"]["status"] == "released", "エスクローが解放されていません"
        print("\n>>> エスクロー資金が受取先に解放されたことを確認しました。")

    await sender.close()
    await receiver.close()


async def main():
    wallet = await scenario_1_payment_success()
    await scenario_2_scope_exceeded(wallet)
    await scenario_3_audit_chain(wallet)
    await scenario_4_a2a_escrow()
    await wallet.close()

    banner("全シナリオ完了")


if __name__ == "__main__":
    asyncio.run(main())
