import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AgentTrustClient, AgentTrustError } from '../src/client';

describe('AgentTrustClient', () => {
  beforeEach(() => {
    vi.restoreAllMocks();
  });

  it('throws AgentTrustError on non-ok response', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      json: async () => ({
        error: { code: 'DID_NOT_FOUND', message: 'DID not found', details: {} }
      }),
    });
    vi.stubGlobal('fetch', mockFetch);

    const client = new AgentTrustClient('http://localhost:8000');
    await expect(client.getAudit('did:key:zNonExistent')).rejects.toThrow(AgentTrustError);
  });

  it('AgentTrustError has code and statusCode', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 401,
      json: async () => ({
        error: { code: 'TOKEN_INVALID', message: 'Invalid token', details: {} }
      }),
    });
    vi.stubGlobal('fetch', mockFetch);

    const client = new AgentTrustClient('http://localhost:8000');
    try {
      await client.getAudit('did:key:z123');
    } catch (e) {
      expect(e).toBeInstanceOf(AgentTrustError);
      const err = e as AgentTrustError;
      expect(err.code).toBe('TOKEN_INVALID');
      expect(err.statusCode).toBe(401);
    }
  });

  it('sends correct headers for payment', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({
        transaction_id: 'tx_test',
        status: 'succeeded',
        amount: 5000,
        currency: 'jpy',
        agent_did: 'did:key:z123',
        stripe_payment_intent_id: 'pi_test',
        audit_hash: 'sha256:abc',
        created_at: '2026-01-01T00:00:00Z',
      }),
    });
    vi.stubGlobal('fetch', mockFetch);

    const client = new AgentTrustClient('http://localhost:8000');
    await client.executePayment('test-token', { amount: 5000, currency: 'jpy' });

    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:8000/payment/execute',
      expect.objectContaining({
        headers: expect.objectContaining({ Authorization: 'Bearer test-token' }),
      })
    );
  });
});
