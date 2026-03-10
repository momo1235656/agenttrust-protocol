"""FastAPI application entry point."""
from contextlib import asynccontextmanager
from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from server.config import settings
from server.dependencies import init_db
from server.routers import did, auth, payment, audit


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifecycle handler - initialize DB on startup."""
    await init_db()
    yield


app = FastAPI(
    title="AgentTrust Protocol",
    description="Secure payment infrastructure for AI agents",
    version="0.1.0",
    lifespan=lifespan,
)

# CORS configuration
app.add_middleware(
    CORSMiddleware,
    allow_origins=settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Register routers
app.include_router(did.router)
app.include_router(auth.router)
app.include_router(payment.router)
app.include_router(audit.router)


@app.get("/health")
async def health_check():
    """Health check endpoint."""
    return {"status": "ok", "service": "AgentTrust Protocol"}
