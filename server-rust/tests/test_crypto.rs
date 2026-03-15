/// Unit tests for cryptographic operations.

#[cfg(test)]
mod tests {
    use agenttrust_server::crypto::keys::{derive_did, resolve_public_key, AgentKeyPair};
    use agenttrust_server::crypto::signing::verify_signature_b64;
    use base64::Engine;

    #[test]
    fn test_generate_keypair() {
        let keypair = AgentKeyPair::generate();
        assert!(keypair.did.starts_with("did:key:z"));
        assert_eq!(keypair.public_key_bytes().len(), 32);
    }

    #[test]
    fn test_did_derivation_consistency() {
        let keypair = AgentKeyPair::generate();
        let did1 = keypair.did.clone();
        let did2 = derive_did(&keypair.verifying_key);
        assert_eq!(did1, did2);
    }

    #[test]
    fn test_did_resolve_roundtrip() {
        let keypair = AgentKeyPair::generate();
        let recovered = resolve_public_key(&keypair.did).expect("Should resolve DID");
        assert_eq!(recovered.as_bytes(), keypair.verifying_key.as_bytes());
    }

    #[test]
    fn test_sign_and_verify() {
        let b64 = base64::engine::general_purpose::STANDARD;
        let keypair = AgentKeyPair::generate();
        let message = b"hello, AgentTrust!";
        let message_b64 = b64.encode(message);

        let sig = keypair.sign(message);
        let sig_b64 = b64.encode(sig.to_bytes());

        verify_signature_b64(&keypair.verifying_key, &message_b64, &sig_b64)
            .expect("Signature should be valid");
    }

    #[test]
    fn test_verify_wrong_message_fails() {
        let b64 = base64::engine::general_purpose::STANDARD;
        let keypair = AgentKeyPair::generate();
        let message = b"correct message";
        let wrong_message = b"wrong message";

        let sig = keypair.sign(message);
        let sig_b64 = b64.encode(sig.to_bytes());
        let wrong_message_b64 = b64.encode(wrong_message);

        let result = verify_signature_b64(&keypair.verifying_key, &wrong_message_b64, &sig_b64);
        assert!(result.is_err());
    }

    #[test]
    fn test_private_key_base64_is_44_chars() {
        let keypair = AgentKeyPair::generate();
        // 32 bytes → 44 chars in standard base64 (with padding)
        assert_eq!(keypair.private_key_base64().len(), 44);
    }

    #[test]
    fn test_public_key_base64_is_44_chars() {
        let keypair = AgentKeyPair::generate();
        assert_eq!(keypair.public_key_base64().len(), 44);
    }
}
