"""Test configuration and fixtures."""
import asyncio
import pytest
import pytest_asyncio
from typing import AsyncGenerator

from sqlalchemy.ext.asyncio import AsyncSession, create_async_engine, async_sessionmaker
from httpx import AsyncClient, ASGITransport

from server.models import Base
from server.main import app
from server.dependencies import get_db


TEST_DATABASE_URL = "sqlite+aiosqlite:///:memory:"

test_engine = create_async_engine(TEST_DATABASE_URL, echo=False)
test_session_factory = async_sessionmaker(test_engine, expire_on_commit=False)


async def override_get_db() -> AsyncGenerator[AsyncSession, None]:
    """Override database dependency for tests."""
    async with test_session_factory() as session:
        try:
            yield session
        except Exception:
            await session.rollback()
            raise
        finally:
            await session.close()


app.dependency_overrides[get_db] = override_get_db


@pytest_asyncio.fixture(scope="session", autouse=True)
async def setup_test_db():
    """Create test database tables."""
    async with test_engine.begin() as conn:
        await conn.run_sync(Base.metadata.create_all)
    yield
    async with test_engine.begin() as conn:
        await conn.run_sync(Base.metadata.drop_all)


@pytest_asyncio.fixture
async def client() -> AsyncGenerator[AsyncClient, None]:
    """Provide an async test client."""
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as c:
        yield c


@pytest_asyncio.fixture
async def db_session() -> AsyncGenerator[AsyncSession, None]:
    """Provide a test database session."""
    async with test_session_factory() as session:
        yield session


@pytest_asyncio.fixture
async def created_did(client: AsyncClient):
    """Fixture that creates a DID for testing."""
    response = await client.post("/did/create", json={
        "display_name": "test-agent",
        "max_transaction_limit": 50000,
        "allowed_categories": ["electronics", "software"]
    })
    assert response.status_code == 201
    return response.json()


@pytest_asyncio.fixture
async def auth_token(client: AsyncClient, created_did: dict):
    """Fixture that creates a DID and gets an auth token."""
    import time
    import base64
    from server.crypto.signing import sign_message_b64

    did = created_did["did"]
    private_key = base64.b64decode(created_did["private_key_base64"])
    message = f"auth_request_{int(time.time())}"
    signature = sign_message_b64(private_key, message)

    response = await client.post("/auth/token", json={
        "did": did,
        "message": message,
        "signature": signature,
        "requested_scopes": ["payment:execute"],
    })
    assert response.status_code == 200
    data = response.json()
    return {"token": data["access_token"], "did": did, "private_key": private_key}
