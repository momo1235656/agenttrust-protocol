"""Trust score client for AgentTrust Protocol."""
import httpx


class TrustScoreClient:
    def __init__(self, server_url: str, agent_did: str):
        self.server_url = server_url.rstrip("/")
        self.agent_did = agent_did
        from urllib.parse import quote
        self._encoded_did = quote(agent_did, safe="")

    async def get_score(self) -> dict:
        async with httpx.AsyncClient() as client:
            r = await client.get(f"{self.server_url}/trust/{self._encoded_did}/score")
            r.raise_for_status()
            return r.json()

    async def get_history(self, from_date: str = None, to_date: str = None) -> dict:
        params = {}
        if from_date:
            params["from"] = from_date
        if to_date:
            params["to"] = to_date
        async with httpx.AsyncClient() as client:
            r = await client.get(
                f"{self.server_url}/trust/{self._encoded_did}/history",
                params=params,
            )
            r.raise_for_status()
            return r.json()

    async def recalculate(self) -> dict:
        async with httpx.AsyncClient() as client:
            r = await client.post(
                f"{self.server_url}/trust/{self._encoded_did}/recalculate"
            )
            r.raise_for_status()
            return r.json()
