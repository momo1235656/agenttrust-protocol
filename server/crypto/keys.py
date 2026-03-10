"""
Ed25519 keypair generation and DID derivation.
This module is designed to be replaceable with a Rust implementation via PyO3.
"""
from nacl.signing import SigningKey
import base58


def generate_keypair() -> tuple[bytes, bytes]:
    """Generate an Ed25519 private/public key pair.

    Returns:
        tuple[bytes, bytes]: (private_key, public_key) both as raw bytes
    """
    signing_key = SigningKey.generate()
    private_key = bytes(signing_key)
    public_key = bytes(signing_key.verify_key)
    return private_key, public_key


def derive_did(public_key: bytes) -> str:
    """Derive a did:key identifier from an Ed25519 public key.

    Uses multicodec ed25519-pub prefix (0xed01) and multibase base58btc encoding (z prefix).

    Args:
        public_key: Raw Ed25519 public key bytes (32 bytes)

    Returns:
        str: DID string in format "did:key:z{base58encoded}"
    """
    # multicodec ed25519-pub prefix: 0xed01
    multicodec_key = b'\xed\x01' + public_key
    encoded = base58.b58encode(multicodec_key).decode()
    return f"did:key:z{encoded}"


def public_key_to_base64(public_key: bytes) -> str:
    """Encode public key bytes to base64 string.

    Args:
        public_key: Raw public key bytes

    Returns:
        str: Base64-encoded public key
    """
    import base64
    return base64.b64encode(public_key).decode()


def base64_to_public_key(b64_key: str) -> bytes:
    """Decode base64 string to public key bytes.

    Args:
        b64_key: Base64-encoded public key string

    Returns:
        bytes: Raw public key bytes
    """
    import base64
    return base64.b64decode(b64_key)
