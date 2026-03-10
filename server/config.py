"""Application configuration loaded from environment variables."""
import os
import json
import base64
from pathlib import Path
from dotenv import load_dotenv
from nacl.signing import SigningKey

load_dotenv()

class Settings:
    stripe_secret_key: str = os.getenv("STRIPE_SECRET_KEY", "sk_test_dummy")
    database_url: str = os.getenv("DATABASE_URL", "sqlite+aiosqlite:///./data/agenttrust.db")
    did_store_path: str = os.getenv("DID_STORE_PATH", "./data/dids")
    cors_origins: list[str] = json.loads(os.getenv("CORS_ORIGINS", '["*"]'))

    # JWT signing keys (Ed25519) - server's own keys for signing tokens
    _server_private_key: bytes | None = None
    _server_public_key: bytes | None = None

    def get_server_keypair(self) -> tuple[bytes, bytes]:
        """Get or generate server's Ed25519 keypair for JWT signing."""
        if self._server_private_key is None:
            priv_b64 = os.getenv("JWT_SERVER_PRIVATE_KEY", "").strip()
            pub_b64 = os.getenv("JWT_SERVER_PUBLIC_KEY", "").strip()
            if priv_b64 and pub_b64:
                self._server_private_key = base64.b64decode(priv_b64)
                self._server_public_key = base64.b64decode(pub_b64)
            else:
                # Generate new keypair
                sk = SigningKey.generate()
                self._server_private_key = bytes(sk)
                self._server_public_key = bytes(sk.verify_key)
        return self._server_private_key, self._server_public_key


settings = Settings()
