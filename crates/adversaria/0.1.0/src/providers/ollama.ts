import type { Provider, ProviderResponse } from "../core/types.js";

export interface OllamaProviderOptions {
  baseUrl?: string;
}

export class OllamaProvider implements Provider {
  name = "ollama" as const;
  private baseUrl: string;

  constructor(opts?: OllamaProviderOptions) {
    this.baseUrl = opts?.baseUrl ?? process.env.OLLAMA_BASE_URL ?? "http://localhost:11434";
  }

  async sendPrompt(model: string, prompt: string): Promise<ProviderResponse> {
    const res = await fetch(`${this.baseUrl}/api/chat`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        model,
        messages: [{ role: "user", content: prompt }],
        stream: false,
      }),
    });

    if (!res.ok) {
      const body = await res.text();
      throw new Error(`Ollama API error ${res.status}: ${body}`);
    }

    const data = (await res.json()) as {
      message: { content: string };
    };

    return {
      content: data.message?.content ?? "",
      raw: data,
    };
  }
}
