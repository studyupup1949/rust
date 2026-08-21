import type { Provider, ProviderResponse } from "../core/types.js";

export interface AnthropicProviderOptions {
  apiKey?: string;
  baseUrl?: string;
}

export class AnthropicProvider implements Provider {
  name = "anthropic" as const;
  private apiKey: string;
  private baseUrl: string;

  constructor(opts?: AnthropicProviderOptions) {
    this.apiKey = opts?.apiKey ?? process.env.ANTHROPIC_API_KEY ?? "";
    this.baseUrl = opts?.baseUrl ?? "https://api.anthropic.com";
    if (!this.apiKey) throw new Error("Anthropic API key required (set ANTHROPIC_API_KEY or config)");
  }

  async sendPrompt(model: string, prompt: string): Promise<ProviderResponse> {
    const res = await fetch(`${this.baseUrl}/v1/messages`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "x-api-key": this.apiKey,
        "anthropic-version": "2023-06-01",
      },
      body: JSON.stringify({
        model,
        messages: [{ role: "user", content: prompt }],
        max_tokens: 1024,
      }),
    });

    if (!res.ok) {
      const body = await res.text();
      throw new Error(`Anthropic API error ${res.status}: ${body}`);
    }

    const data = (await res.json()) as {
      content: Array<{ type: string; text: string }>;
    };

    const text = data.content
      .filter((c) => c.type === "text")
      .map((c) => c.text)
      .join("");

    return { content: text, raw: data };
  }
}
