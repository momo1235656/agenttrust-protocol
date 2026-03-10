/**
 * Ed25519鍵ペアの生成、署名、検証。
 * @noble/ed25519 を使用（純粋TypeScript実装、依存ゼロ）。
 * Python SDKのserver/crypto/と同じロジック。
 */
import * as ed from '@noble/ed25519';
import { sha512 } from '@noble/hashes/sha512';

// @noble/ed25519 v2 requires setting sha512 implementation
ed.etc.sha512Sync = (msg: Uint8Array) => sha512(msg);

/** Ed25519鍵ペアを生成する */
export async function generateKeypair(): Promise<{
  privateKey: Uint8Array;
  publicKey: Uint8Array;
}> {
  const privateKey = ed.utils.randomPrivateKey();
  const publicKey = await ed.getPublicKeyAsync(privateKey);
  return { privateKey, publicKey };
}

/** メッセージに署名する */
export async function signMessage(
  privateKey: Uint8Array,
  message: Uint8Array,
): Promise<Uint8Array> {
  return await ed.signAsync(message, privateKey);
}

/** 署名を検証する */
export async function verifySignature(
  publicKey: Uint8Array,
  message: Uint8Array,
  signature: Uint8Array,
): Promise<boolean> {
  try {
    return await ed.verifyAsync(signature, message, publicKey);
  } catch {
    return false;
  }
}

/** Uint8Array → Base64文字列（Node 18+/Deno/Bun/Browser対応） */
export function toBase64(bytes: Uint8Array): string {
  let binary = '';
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

/** Base64文字列 → Uint8Array（Node 18+/Deno/Bun/Browser対応） */
export function fromBase64(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
