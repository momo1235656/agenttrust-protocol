import httpx
from typing import Optional


class A2AClient:
    def __init__(self, server_url: str, timeout: float = 30.0):
        self._base = server_url.rstrip("/")
        self._client = httpx.AsyncClient(timeout=timeout)

    async def initiate(
        self,
        sender_did: str,
        receiver_did: str,
        amount: int,
        description: str,
        service_type: Optional[str] = None,
        timeout_minutes: int = 60,
        currency: str = "jpy",
    ) -> dict:
        resp = await self._client.post(
            f"{self._base}/a2a/transfer",
            json={
                "sender_did": sender_did,
                "receiver_did": receiver_did,
                "amount": amount,
                "currency": currency,
                "description": description,
                "service_type": service_type,
                "timeout_minutes": timeout_minutes,
            },
        )
        resp.raise_for_status()
        return resp.json()

    async def get_status(self, transfer_id: str) -> dict:
        resp = await self._client.get(f"{self._base}/a2a/transfer/{transfer_id}")
        resp.raise_for_status()
        return resp.json()

    async def complete(self, saga_id: str, reporter_did: str, result_summary: str = "") -> dict:
        resp = await self._client.post(
            f"{self._base}/saga/{saga_id}/complete",
            json={"reporter_did": reporter_did, "result_summary": result_summary},
        )
        resp.raise_for_status()
        return resp.json()

    async def close(self):
        await self._client.aclose()
