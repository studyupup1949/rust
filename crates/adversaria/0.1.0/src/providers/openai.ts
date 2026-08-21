import type { Provider, ProviderResponse } from "../core/types.js";

export interface OpenAIProviderOptions {
  apiKey?: string;
  baseUrl?: string;
}

export class OpenAIProvider implements Provider {
  name = "openai" as const;
  private apiKey: string;
  private baseUrl: string;

  constructor(opts?: OpenAIProviderOptions) {
    this.apiKey = opts?.apiKey ?? process.env.OPENAI_API_KEY ?? "";
    this.baseUrl = opts?.baseUrl ?? "https://api.openai.com";
    if (!this.apiKey) throw new Error("OpenAI API key required (set OPENAI_API_KEY or config)");
  }

  async sendPrompt(model: string, prompt: string): Promise<ProviderResponse> {
    const res = await fetch(`${this.baseUrl}/v1/chat/completions`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${this.apiKey}`,
      },
      body: JSON.stringify({
        model,
        messages: [{ role: "user", content: prompt }],
        max_tokens: 1024,
        temperature: 0,
      }),
    });

    if (!res.ok) {
      const body = await res.text();
      throw new Error(`OpenAI API error ${res.status}: ${body}`);
    }

    const data = (await res.json()) as {
      choices: Array<{ message: { content: string } }>;
    };

    return {
      content: data.choices[0]?.message?.content ?? "",
      raw: data,
    };
  }
}
