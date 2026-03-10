/** TypeScript SDKの利用例 */
import { AgentWallet } from 'agenttrust';

async function main() {
  // 1. ウォレット作成（DID自動生成）
  const wallet = new AgentWallet({ serverUrl: 'http://localhost:8000' });
  await wallet.create({ displayName: 'ts-demo', maxLimit: 100000 });
  console.log(`DID: ${wallet.did}`);

  // 2. 決済実行
  const result = await wallet.pay({
    amount: 5000,
    description: 'TypeScript SDKからのテスト決済',
  });
  console.log('決済結果:', result);

  // 3. 監査ログ確認
  const audit = await wallet.getAuditLog();
  console.log('監査ログ:', {
    total: audit.total_transactions,
    valid: audit.chain_valid,
  });
}

main().catch(console.error);
