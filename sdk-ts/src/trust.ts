export class TrustScoreClient {
  constructor(private serverUrl: string, private agentDid: string) {}

  private get encodedDid(): string {
    return encodeURIComponent(this.agentDid);
  }

  async getScore(): Promise<Record<string, unknown>> {
    const r = await fetch(`${this.serverUrl}/trust/${this.encodedDid}/score`);
    if (!r.ok) throw new Error(await r.text());
    return r.json();
  }

  async getHistory(params?: { from?: string; to?: string }): Promise<Record<string, unknown>> {
    const url = new URL(`${this.serverUrl}/trust/${this.encodedDid}/history`);
    if (params?.from) url.searchParams.set('from', params.from);
    if (params?.to) url.searchParams.set('to', params.to);
    const r = await fetch(url.toString());
    if (!r.ok) throw new Error(await r.text());
    return r.json();
  }

  async recalculate(): Promise<Record<string, unknown>> {
    const r = await fetch(`${this.serverUrl}/trust/${this.encodedDid}/recalculate`, { method: 'POST' });
    if (!r.ok) throw new Error(await r.text());
    return r.json();
  }
}
