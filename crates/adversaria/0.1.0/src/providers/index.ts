import type { Provider, AdversariaConfig } from "../core/types.js";
import { OpenAIProvider } from "./openai.js";
import { AnthropicProvider } from "./anthropic.js";
import { OllamaProvider } from "./ollama.js";

export { OpenAIProvider } from "./openai.js";
export { AnthropicProvider } from "./anthropic.js";
export { OllamaProvider } from "./ollama.js";

export function createProvider(config: AdversariaConfig): Provider {
  const name = config.provider.name;
  const providerConf = config.providers?.[name] ?? {};

  switch (name) {
    case "openai":
      return new OpenAIProvider(providerConf as { apiKey?: string; baseUrl?: string });
    case "anthropic":
      return new AnthropicProvider(providerConf as { apiKey?: string; baseUrl?: string });
    case "ollama":
      return new OllamaProvider(providerConf as { baseUrl?: string });
    default:
      throw new Error(
        `Unknown provider "${name}". Built-in providers: openai, anthropic, ollama`,
      );
  }
}
