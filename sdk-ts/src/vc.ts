export class VCClient {
  constructor(private serverUrl: string, private agentDid: string) {}

  async issue(credentialType = 'AgentTrustScore', expirationDays = 30): Promise<Record<string, unknown>> {
    const r = await fetch(`${this.serverUrl}/vc/issue`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ agent_did: this.agentDid, credential_type: credentialType, expiration_days: expirationDays }),
    });
    if (!r.ok) throw new Error(await r.text());
    return r.json();
  }

  async verify(credential: Record<string, unknown>): Promise<Record<string, unknown>> {
    const r = await fetch(`${this.serverUrl}/vc/verify`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ verifiable_credential: credential }),
    });
    if (!r.ok) throw new Error(await r.text());
    return r.json();
  }
}
