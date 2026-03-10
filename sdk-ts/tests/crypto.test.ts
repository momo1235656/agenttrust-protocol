import { describe, it, expect } from 'vitest';
import { generateKeypair, signMessage, verifySignature, toBase64, fromBase64 } from '../src/crypto';

describe('crypto', () => {
  it('generates a valid Ed25519 keypair', async () => {
    const { privateKey, publicKey } = await generateKeypair();
    expect(privateKey).toBeInstanceOf(Uint8Array);
    expect(publicKey).toBeInstanceOf(Uint8Array);
    expect(privateKey.length).toBe(32);
    expect(publicKey.length).toBe(32);
  });

  it('signs and verifies a message', async () => {
    const { privateKey, publicKey } = await generateKeypair();
    const message = new TextEncoder().encode('test message');
    const signature = await signMessage(privateKey, message);
    const valid = await verifySignature(publicKey, message, signature);
    expect(valid).toBe(true);
  });

  it('rejects invalid signature', async () => {
    const { publicKey } = await generateKeypair();
    const message = new TextEncoder().encode('test message');
    const fakeSignature = new Uint8Array(64).fill(0);
    const valid = await verifySignature(publicKey, message, fakeSignature);
    expect(valid).toBe(false);
  });

  it('converts bytes to base64 and back', () => {
    const original = new Uint8Array([1, 2, 3, 4, 5, 255, 0, 128]);
    const b64 = toBase64(original);
    expect(typeof b64).toBe('string');
    const restored = fromBase64(b64);
    expect(restored).toEqual(original);
  });

  it('handles empty bytes', () => {
    const empty = new Uint8Array(0);
    const b64 = toBase64(empty);
    const restored = fromBase64(b64);
    expect(restored.length).toBe(0);
  });
});
