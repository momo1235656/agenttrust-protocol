"""Verifiable Credential client for AgentTrust Protocol."""
import httpx


class VCClient:
    def __init__(self, server_url: str, agent_did: str):
        self.server_url = server_url.rstrip("/")
        self.agent_did = agent_did

    async def issue(self, credential_type: str = "AgentTrustScore", expiration_days: int = 30) -> dict:
        async with httpx.AsyncClient() as client:
            r = await client.post(
                f"{self.server_url}/vc/issue",
                json={
                    "agent_did": self.agent_did,
                    "credential_type": credential_type,
                    "expiration_days": expiration_days,
                },
            )
            r.raise_for_status()
            return r.json()

    async def verify(self, credential: dict) -> dict:
        async with httpx.AsyncClient() as client:
            r = await client.post(
                f"{self.server_url}/vc/verify",
                json={"verifiable_credential": credential},
            )
            r.raise_for_status()
            return r.json()
