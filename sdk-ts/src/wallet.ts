/**
 * TypeScript版 AgentWallet。
 * Python SDKのsdk/wallet.pyと同じ機能を提供。
 */
import { AgentTrustClient } from './client.js';
import { signMessage, toBase64, fromBase64 } from './crypto.js';
import type { WalletConfig, PaymentResponse, AuditResponse } from './types.js';

export class AgentWallet {
  private readonly client: AgentTrustClient;
  private _did: string | null = null;
  private _privateKey: Uint8Array | null = null;
  private _accessToken: string | null = null;
  private _tokenExpiresAt: number = 0;

  constructor(config: WalletConfig = {}) {
    this.client = new AgentTrustClient(config.serverUrl);
    if (config.did && config.privateKey) {
      this._did = config.did;
      this._privateKey = config.privateKey;
    }
  }

  /** エージェントのDID */
  get did(): string | null {
    return this._did;
  }

  /**
   * 新しいDIDを作成してサーバーに登録する。
   *
   * @param options - 作成オプション
   */
  async create(options: {
    displayName?: string;
    maxLimit?: number;
    allowedCategories?: string[];
  } = {}): Promise<void> {
    const res = await this.client.createDID({
      display_name: options.displayName,
      max_transaction_limit: options.maxLimit ?? 100000,
      allowed_categories: options.allowedCategories ?? [],
    });
    this._did = res.did;
    this._privateKey = fromBase64(res.private_key_base64);
  }

  /**
   * アクセストークンを取得する（有効期限内のトークンはキャッシュを返す）。
   *
   * @param scopes - 要求するスコープ
   * @returns JWTアクセストークン
   */
  async getToken(scopes: string[] = ['payment:execute', 'balance:read']): Promise<string> {
    // 有効期限内のトークンがあればキャッシュを返す（60秒のバッファ）
    if (this._accessToken && Date.now() < this._tokenExpiresAt - 60_000) {
      return this._accessToken;
    }

    if (!this._did || !this._privateKey) {
      throw new Error('Wallet not initialized. Call create() first.');
    }

    const message = `auth_request_${Date.now()}`;
    const msgBytes = new TextEncoder().encode(message);
    const signature = await signMessage(this._privateKey, msgBytes);

    const res = await this.client.getToken(
      this._did,
      toBase64(msgBytes),
      toBase64(signature),
      scopes,
    );

    this._accessToken = res.access_token;
    this._tokenExpiresAt = Date.now() + res.expires_in * 1000;
    return this._accessToken;
  }

  /**
   * 決済を実行する。
   *
   * @param options - 決済オプション
   * @returns 決済結果
   */
  async pay(options: {
    amount: number;
    description?: string;
    idempotencyKey?: string;
  }): Promise<PaymentResponse> {
    const token = await this.getToken();
    return this.client.executePayment(token, {
      amount: options.amount,
      currency: 'jpy',
      description: options.description ?? '',
      idempotency_key: options.idempotencyKey ?? `${this._did}-${Date.now()}`,
    });
  }

  /**
   * 監査ログを取得する。
   *
   * @returns 監査チェーンデータ
   */
  async getAuditLog(): Promise<AuditResponse> {
    if (!this._did) throw new Error('Wallet not initialized.');
    return this.client.getAudit(this._did);
  }
}
