/**
 * AgentTrust APIサーバーとの通信を担当。
 * 全てのHTTPリクエスト/レスポンスをここに集約。
 */
import type {
  CreateDIDRequest,
  CreateDIDResponse,
  TokenResponse,
  ExecutePaymentRequest,
  PaymentResponse,
  AuditResponse,
  APIErrorResponse,
} from './types.js';

/** AgentTrust固有のエラークラス */
export class AgentTrustError extends Error {
  constructor(
    public readonly code: string,
    message: string,
    public readonly statusCode: number,
  ) {
    super(`[${code}] ${message}`);
    this.name = 'AgentTrustError';
  }
}

export class AgentTrustClient {
  private readonly baseUrl: string;

  constructor(serverUrl: string = 'http://localhost:8000') {
    this.baseUrl = serverUrl.replace(/\/$/, '');
  }

  /** POST /did/create */
  async createDID(req: CreateDIDRequest): Promise<CreateDIDResponse> {
    return this.post<CreateDIDResponse>('/did/create', req);
  }

  /** POST /did/verify */
  async verifyDID(
    did: string,
    message: string,
    signature: string,
  ): Promise<{ verified: boolean }> {
    return this.post('/did/verify', { did, message, signature });
  }

  /** POST /auth/token */
  async getToken(
    did: string,
    message: string,
    signature: string,
    scopes: string[],
  ): Promise<TokenResponse> {
    return this.post<TokenResponse>('/auth/token', {
      did,
      message,
      signature,
      requested_scopes: scopes,
    });
  }

  /** POST /payment/execute */
  async executePayment(
    token: string,
    req: ExecutePaymentRequest,
  ): Promise<PaymentResponse> {
    return this.post<PaymentResponse>('/payment/execute', req, {
      Authorization: `Bearer ${token}`,
    });
  }

  /** GET /payment/{id} */
  async getPayment(transactionId: string): Promise<PaymentResponse> {
    return this.get<PaymentResponse>(`/payment/${transactionId}`);
  }

  /** GET /audit/{agent_did} */
  async getAudit(agentDid: string): Promise<AuditResponse> {
    return this.get<AuditResponse>(`/audit/${encodeURIComponent(agentDid)}`);
  }

  private async post<T>(
    path: string,
    body: unknown,
    headers?: Record<string, string>,
  ): Promise<T> {
    const res = await fetch(`${this.baseUrl}${path}`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json', ...headers },
      body: JSON.stringify(body),
    });
    if (!res.ok) {
      const err = (await res.json()) as APIErrorResponse;
      throw new AgentTrustError(err.error.code, err.error.message, res.status);
    }
    return res.json() as Promise<T>;
  }

  private async get<T>(
    path: string,
    headers?: Record<string, string>,
  ): Promise<T> {
    const res = await fetch(`${this.baseUrl}${path}`, { headers });
    if (!res.ok) {
      const err = (await res.json()) as APIErrorResponse;
      throw new AgentTrustError(err.error.code, err.error.message, res.status);
    }
    return res.json() as Promise<T>;
  }
}
