"""
Ed25519 message signing and verification.
This module is designed to be replaceable with a Rust implementation via PyO3.
"""
from nacl.signing import SigningKey, VerifyKey
from nacl.exceptions import BadSignatureError
import base64


def sign_message(private_key: bytes, message: bytes) -> bytes:
    """Sign a message with an Ed25519 private key.

    Args:
        private_key: Raw Ed25519 private key bytes (32 bytes)
        message: Message bytes to sign

    Returns:
        bytes: Ed25519 signature bytes (64 bytes)
    """
    signing_key = SigningKey(private_key)
    signed = signing_key.sign(message)
    return signed.signature


def verify_signature(public_key: bytes, message: bytes, signature: bytes) -> bool:
    """Verify an Ed25519 signature.

    Args:
        public_key: Raw Ed25519 public key bytes (32 bytes)
        message: Original message bytes
        signature: Ed25519 signature bytes (64 bytes)

    Returns:
        bool: True if signature is valid, False otherwise
    """
    try:
        verify_key = VerifyKey(public_key)
        verify_key.verify(message, signature)
        return True
    except BadSignatureError:
        return False


def sign_message_b64(private_key: bytes, message: str) -> str:
    """Sign a message string and return base64-encoded signature.

    Args:
        private_key: Raw Ed25519 private key bytes
        message: Message string to sign

    Returns:
        str: Base64-encoded signature
    """
    sig = sign_message(private_key, message.encode())
    return base64.b64encode(sig).decode()


def verify_signature_b64(public_key: bytes, message: str, signature_b64: str) -> bool:
    """Verify a base64-encoded signature against a message string.

    Args:
        public_key: Raw Ed25519 public key bytes
        message: Original message string
        signature_b64: Base64-encoded signature string

    Returns:
        bool: True if signature is valid, False otherwise
    """
    try:
        sig = base64.b64decode(signature_b64)
        msg = message.encode()
        return verify_signature(public_key, msg, sig)
    except Exception:
        return False
