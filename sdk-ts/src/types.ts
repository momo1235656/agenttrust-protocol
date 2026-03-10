/** DID作成リクエスト */
export interface CreateDIDRequest {
  display_name?: string;
  max_transaction_limit?: number;
  allowed_categories?: string[];
}

/** DID作成レスポンス */
export interface CreateDIDResponse {
  did: string;
  document: DIDDocument;
  private_key_base64: string;
}

/** DIDドキュメント */
export interface DIDDocument {
  "@context": string[];
  id: string;
  authentication: AuthenticationMethod[];
  service: ServiceEndpoint[];
}

/** 認証方式 */
export interface AuthenticationMethod {
  type: string;
  publicKeyBase64: string;
}

/** サービスエンドポイント */
export interface ServiceEndpoint {
  type: string;
  maxTransactionLimit: number;
  allowedCategories: string[];
}

/** トークン発行レスポンス */
export interface TokenResponse {
  access_token: string;
  token_type: string;
  expires_in: number;
  scopes: string[];
}

/** 決済実行リクエスト */
export interface ExecutePaymentRequest {
  amount: number;
  currency?: string;
  description?: string;
  idempotency_key?: string;
}

/** 決済実行レスポンス */
export interface PaymentResponse {
  transaction_id: string;
  status: string;
  amount: number;
  currency: string;
  agent_did: string;
  stripe_payment_intent_id: string;
  audit_hash: string;
  created_at: string;
}

/** 監査ログレスポンス */
export interface AuditResponse {
  agent_did: string;
  chain: AuditEntry[];
  chain_valid: boolean;
  total_transactions: number;
  success_rate: number;
}

/** 監査ログエントリ */
export interface AuditEntry {
  index: number;
  transaction_id: string;
  amount: number;
  status: string;
  timestamp: string;
  prev_hash: string;
  hash: string;
}

/** APIエラーレスポンス */
export interface APIErrorResponse {
  error: {
    code: string;
    message: string;
    details: Record<string, unknown>;
  };
}

/** ウォレットの設定 */
export interface WalletConfig {
  serverUrl?: string;
  did?: string;
  privateKey?: Uint8Array;
}
