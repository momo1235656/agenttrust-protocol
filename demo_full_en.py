"""demo_full.py — AgentTrust Protocol live demo (for screen recording)

Runs four scenarios in sequence:
  1. DID creation -> successful payment
  2. Over-limit payment -> rejected with SCOPE_EXCEEDED
  3. Audit chain verification (chain_valid: true)
  4. A2A escrow: initiate -> completion report -> release

Prerequisite: the server must be running via `docker-compose up` (http://localhost:8000)
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
    banner("Scenario 1/4: DID creation -> successful payment")

    wallet = AgentWallet(server_url=SERVER_URL)
    await wallet.create(display_name="demo-agent-A", max_limit=50000)
    print(f"DID: {wallet.did}")
    print(f"Spending limit: 50,000 JPY")

    time.sleep(1)
    print("\n>>> Executing a 5,000 JPY payment...")
    result = await wallet.pay(amount=5000, description="Test purchase")

    print(f"\n[OK] Payment succeeded")
    print(f"  status              : {result['status']}")
    print(f"  transaction_id       : {result['transaction_id']}")
    print(f"  stripe_payment_intent: {result['stripe_payment_intent_id']}")
    print(f"  audit_hash           : {result['audit_hash']}")

    return wallet


async def scenario_2_scope_exceeded(wallet: AgentWallet):
    banner("Scenario 2/4: Over-limit payment -> rejected with SCOPE_EXCEEDED")

    over_limit_amount = 999_999
    print(f"DID: {wallet.did}")
    print(f"Spending limit: 50,000 JPY")
    print(f"\n>>> Attempting a payment of {over_limit_amount:,} JPY, over the limit...")
    time.sleep(1)

    try:
        await wallet.pay(amount=over_limit_amount, description="Over-limit test")
        print("\n[UNEXPECTED] Payment succeeded when it should have been rejected")
    except httpx.HTTPStatusError as e:
        body = e.response.json()
        print(f"\n[REJECTED] HTTP {e.response.status_code}")
        print(f"  error.code   : {body['error']['code']}")
        print(f"  error.message: {body['error']['message']}")
        assert body["error"]["code"] == "SCOPE_EXCEEDED", "Unexpected error code"
        print("\n>>> Confirmed the spending limit is correctly enforced.")


async def scenario_3_audit_chain(wallet: AgentWallet):
    banner("Scenario 3/4: Audit chain verification")

    print(f"DID: {wallet.did}")
    print("\n>>> Verifying the audit hash chain via POST /audit/verify...")
    time.sleep(1)

    async with httpx.AsyncClient() as client:
        resp = await client.post(
            f"{SERVER_URL}/audit/verify",
            json={"agent_did": wallet.did},
        )
        resp.raise_for_status()
        result = resp.json()

    print(f"\n[Verification result]")
    print(f"  chain_valid   : {result['chain_valid']}")
    print(f"  total_entries : {result['total_entries']}")
    print(f"  verified_at   : {result['verified_at']}")
    assert result["chain_valid"] is True, "Audit chain is invalid"
    print("\n>>> Confirmed the audit hash chain is intact.")


async def scenario_4_a2a_escrow():
    banner("Scenario 4/4: A2A escrow (initiate -> completion report -> release)")

    # Create the sender and receiver agents
    sender = AgentWallet(server_url=SERVER_URL)
    await sender.create(display_name="demo-sender", max_limit=100000)
    receiver = AgentWallet(server_url=SERVER_URL)
    await receiver.create(display_name="demo-receiver", max_limit=100000)
    print(f"Sender DID  : {sender.did}")
    print(f"Receiver DID: {receiver.did}")

    async with httpx.AsyncClient(timeout=30.0) as client:
        # Trust scores must exist for both agents before an A2A transfer
        print("\n>>> Calculating trust scores for both agents...")
        for wallet in (sender, receiver):
            resp = await client.post(f"{SERVER_URL}/trust/{wallet.did}/recalculate")
            resp.raise_for_status()
            score = resp.json()["new_score"]
            print(f"  {wallet.did[:24]}... score={score}")

        # Initiate the A2A transfer (kicks off the 10-step saga + escrow funding)
        print("\n>>> Initiating the A2A transfer (funding escrow)...")
        time.sleep(1)
        resp = await client.post(
            f"{SERVER_URL}/a2a/transfer",
            json={
                "sender_did": sender.did,
                "receiver_did": receiver.did,
                "amount": 8000,
                "currency": "jpy",
                "description": "A2A demo: payment for service",
                "service_type": "demo-service",
                "timeout_minutes": 30,
            },
        )
        resp.raise_for_status()
        transfer = resp.json()
        transfer_id = transfer["transfer_id"]
        saga_id = transfer["saga_id"]
        print(f"\n[Transfer initiated]")
        print(f"  transfer_id: {transfer_id}")
        print(f"  saga_id    : {saga_id}")
        print(f"  status     : {transfer['status']}  (escrow: {transfer['escrow_status']})")
        print(f"  progress   : {transfer['steps']['completed']}/{transfer['steps']['total']} "
              f"(current: {transfer['steps']['current']})")

        # Receiver reports completion -> triggers automatic escrow release
        print("\n>>> Receiver reports service completion (triggers escrow release)...")
        time.sleep(1)
        resp = await client.post(
            f"{SERVER_URL}/saga/{saga_id}/complete",
            json={
                "reporter_did": receiver.did,
                "result_summary": "Service delivered",
            },
        )
        resp.raise_for_status()
        complete_result = resp.json()
        print(f"\n[Completion accepted]")
        print(f"  status: {complete_result['status']}")

        # Check the final state
        print("\n>>> Checking final state...")
        resp = await client.get(f"{SERVER_URL}/a2a/transfer/{transfer_id}")
        resp.raise_for_status()
        final = resp.json()
        print(f"\n[Final result]")
        print(f"  transfer.status : {final['status']}")
        print(f"  escrow.status   : {final['escrow']['status']}")
        print(f"  saga.status     : {final['saga']['status']}")
        assert final["escrow"]["status"] == "released", "Escrow was not released"
        print("\n>>> Confirmed the escrow funds were released to the receiver.")

    await sender.close()
    await receiver.close()


async def main():
    wallet = await scenario_1_payment_success()
    await scenario_2_scope_exceeded(wallet)
    await scenario_3_audit_chain(wallet)
    await scenario_4_a2a_escrow()
    await wallet.close()

    banner("All scenarios completed")


if __name__ == "__main__":
    asyncio.run(main())
