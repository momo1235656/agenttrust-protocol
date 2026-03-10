import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentWallet } from '../src/wallet';
import { AgentTrustError } from '../src/client';

// Mock fetch globally
const createMockDIDResponse = () => ({
  did: 'did:key:z6MkTestDID123456789',
  document: {
    '@context': ['https://www.w3.org/ns/did/v1'],
    id: 'did:key:z6MkTestDID123456789',
    authentication: [{ type: 'Ed25519VerificationKey2020', publicKeyBase64: 'dGVzdA==' }],
    service: [{ type: 'AgentPayment', maxTransactionLimit: 50000, allowedCategories: [] }],
  },
  private_key_base64: 'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=',
});

const createMockTokenResponse = () => ({
  access_token: 'eyJhbGciOiJFZERTQSJ9.test.signature',
  token_type: 'Bearer',
  expires_in: 1800,
  scopes: ['payment:execute', 'balance:read'],
});

const createMockPaymentResponse = () => ({
  transaction_id: 'tx_test123',
  status: 'succeeded',
  amount: 5000,
  currency: 'jpy',
  agent_did: 'did:key:z6MkTestDID123456789',
  stripe_payment_intent_id: 'pi_test',
  audit_hash: 'sha256:abc123',
  created_at: '2026-01-01T00:00:00Z',
});

describe('AgentWallet', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('creates a wallet with a new DID', async () => {
    let callCount = 0;
    vi.stubGlobal('fetch', vi.fn().mockImplementation(async (url: string) => {
      callCount++;
      if (url.includes('/did/create')) {
        return { ok: true, json: async () => createMockDIDResponse() };
      }
      return { ok: true, json: async () => ({}) };
    }));

    const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
    await wallet.create({ displayName: 'test-agent', maxLimit: 50000 });

    expect(wallet.did).toBe('did:key:z6MkTestDID123456789');
    expect(wallet.did).toMatch(/^did:key:z/);
  });

  it('did is null before create()', () => {
    const wallet = new AgentWallet();
    expect(wallet.did).toBeNull();
  });

  it('initializes with existing DID and privateKey', () => {
    const existingKey = new Uint8Array(32).fill(1);
    const wallet = new AgentWallet({
      did: 'did:key:zExisting',
      privateKey: existingKey,
    });
    expect(wallet.did).toBe('did:key:zExisting');
  });

  it('gets a token after create()', async () => {
    let requestCount = 0;
    vi.stubGlobal('fetch', vi.fn().mockImplementation(async (url: string) => {
      requestCount++;
      if (url.includes('/did/create')) {
        return { ok: true, json: async () => createMockDIDResponse() };
      }
      if (url.includes('/auth/token')) {
        return { ok: true, json: async () => createMockTokenResponse() };
      }
      return { ok: true, json: async () => ({}) };
    }));

    const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
    await wallet.create({ displayName: 'test-agent' });
    const token = await wallet.getToken();

    expect(typeof token).toBe('string');
    expect(token.length).toBeGreaterThan(0);
  });

  it('caches token within expiry window', async () => {
    const fetchMock = vi.fn().mockImplementation(async (url: string) => {
      if (url.includes('/did/create')) {
        return { ok: true, json: async () => createMockDIDResponse() };
      }
      if (url.includes('/auth/token')) {
        return { ok: true, json: async () => createMockTokenResponse() };
      }
      return { ok: true, json: async () => ({}) };
    });
    vi.stubGlobal('fetch', fetchMock);

    const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
    await wallet.create();

    const token1 = await wallet.getToken();
    const token2 = await wallet.getToken();

    expect(token1).toBe(token2);
    // /auth/token should only be called once due to caching
    const tokenCalls = fetchMock.mock.calls.filter((call: unknown[]) =>
      (call[0] as string).includes('/auth/token')
    );
    expect(tokenCalls.length).toBe(1);
  });

  it('executes payment successfully', async () => {
    vi.stubGlobal('fetch', vi.fn().mockImplementation(async (url: string) => {
      if (url.includes('/did/create')) {
        return { ok: true, json: async () => createMockDIDResponse() };
      }
      if (url.includes('/auth/token')) {
        return { ok: true, json: async () => createMockTokenResponse() };
      }
      if (url.includes('/payment/execute')) {
        return { ok: true, json: async () => createMockPaymentResponse() };
      }
      return { ok: true, json: async () => ({}) };
    }));

    const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
    await wallet.create({ maxLimit: 50000 });
    const result = await wallet.pay({ amount: 5000, description: 'テスト決済' });

    expect(result.status).toBe('succeeded');
    expect(result.amount).toBe(5000);
    expect(result.transaction_id).toBe('tx_test123');
  });

  it('throws AgentTrustError on invalid server URL', async () => {
    vi.stubGlobal('fetch', vi.fn().mockRejectedValue(new TypeError('Failed to fetch')));

    const wallet = new AgentWallet({ serverUrl: 'http://invalid-server:9999' });
    await expect(wallet.create()).rejects.toThrow();
  });
});
