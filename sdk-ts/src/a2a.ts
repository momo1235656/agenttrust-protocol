export interface A2ATransferRequest {
  senderDid: string;
  receiverDid: string;
  amount: number;
  description: string;
  serviceType?: string;
  timeoutMinutes?: number;
  currency?: string;
}

export interface A2ATransferResponse {
  transfer_id: string;
  saga_id: string;
  status: string;
  escrow_status: string;
  timeout_at: string | null;
  steps: { total: number; completed: number; current: string };
}

export class A2AClient {
  constructor(private readonly serverUrl: string) {}

  async initiate(req: A2ATransferRequest): Promise<A2ATransferResponse> {
    const res = await fetch(`${this.serverUrl}/a2a/transfer`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        sender_did: req.senderDid,
        receiver_did: req.receiverDid,
        amount: req.amount,
        currency: req.currency ?? 'jpy',
        description: req.description,
        service_type: req.serviceType,
        timeout_minutes: req.timeoutMinutes ?? 60,
      }),
    });
    if (!res.ok) throw new Error(`A2A initiate failed: ${await res.text()}`);
    return res.json();
  }

  async getStatus(transferId: string): Promise<Record<string, unknown>> {
    const res = await fetch(`${this.serverUrl}/a2a/transfer/${transferId}`);
    if (!res.ok) throw new Error(`Get transfer failed: ${await res.text()}`);
    return res.json();
  }

  async complete(sagaId: string, reporterDid: string, resultSummary = ''): Promise<Record<string, unknown>> {
    const res = await fetch(`${this.serverUrl}/saga/${sagaId}/complete`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ reporter_did: reporterDid, result_summary: resultSummary }),
    });
    if (!res.ok) throw new Error(`Complete saga failed: ${await res.text()}`);
    return res.json();
  }
}
