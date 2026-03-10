/**
 * LangChain.js対応の決済ツール（オプション）。
 * LangChain.jsがインストールされている場合のみ動作する。
 */
import type { AgentWallet } from '../wallet.js';

/** LangChain.js DynamicStructuredTool互換の決済ツール定義 */
export interface PaymentToolDefinition {
  name: string;
  description: string;
  schema: {
    type: 'object';
    properties: {
      amount: { type: 'number'; description: string };
      description: { type: 'string'; description: string };
    };
    required: ['amount'];
  };
  func: (input: { amount: number; description?: string }) => Promise<string>;
}

/**
 * LangChain.js用の決済ツール定義を作成する。
 *
 * @param wallet - 初期化済みのAgentWalletインスタンス
 * @returns LangChain.js DynamicStructuredTool互換のツール定義
 */
export function createPaymentTool(wallet: AgentWallet): PaymentToolDefinition {
  return {
    name: 'agenttrust_payment',
    description: 'AIエージェントとして決済を実行します。金額（円）と説明を指定してください。',
    schema: {
      type: 'object' as const,
      properties: {
        amount: { type: 'number' as const, description: '決済金額（円）' },
        description: { type: 'string' as const, description: '決済の説明' },
      },
      required: ['amount'] as const,
    },
    func: async (input: { amount: number; description?: string }) => {
      const result = await wallet.pay({
        amount: input.amount,
        description: input.description ?? '',
      });
      return `決済完了: ${result.transaction_id}, ${result.amount}円, ステータス: ${result.status}`;
    },
  };
}

/** PaymentTool alias for named export compatibility */
export const PaymentTool = createPaymentTool;
