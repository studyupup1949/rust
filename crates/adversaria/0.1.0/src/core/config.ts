import { readFile } from "node:fs/promises";
import { resolve } from "node:path";
import yaml from "js-yaml";
import { z } from "zod";
import { AdversariaConfig } from "./types.js";

const configSchema = z.object({
  provider: z.object({
    name: z.string(),
    model: z.string(),
  }),
  providers: z
    .object({
      openai: z.object({ apiKey: z.string().optional(), baseUrl: z.string().optional() }).optional(),
      anthropic: z.object({ apiKey: z.string().optional(), baseUrl: z.string().optional() }).optional(),
      ollama: z.object({ baseUrl: z.string().optional() }).optional(),
    })
    .catchall(z.record(z.any()))
    .optional(),
  suites: z.array(z.string()).optional(),
  payloads: z.record(z.string()).optional(),
  plugins: z.array(z.string()).optional(),
  reportDir: z.string().optional(),
});

const DEFAULT_CONFIG: AdversariaConfig = {
  provider: { name: "openai", model: "gpt-4o-mini" },
  suites: ["prompt-injection", "jailbreaks", "role-confusion", "data-exfiltration"],
  reportDir: "./reports",
};

function expandEnv(value: string): string {
  return value.replace(/\$\{([^}]+)\}/g, (_, key) => process.env[key] ?? "");
}

function expandEnvDeep<T>(obj: T): T {
  if (typeof obj === "string") return expandEnv(obj) as T;
  if (Array.isArray(obj)) return obj.map((v) => expandEnvDeep(v)) as T;
  if (obj && typeof obj === "object") {
    const out: Record<string, unknown> = {};
    for (const [k, v] of Object.entries(obj)) out[k] = expandEnvDeep(v);
    return out as T;
  }
  return obj;
}

export async function loadConfig(pathLike?: string): Promise<AdversariaConfig> {
  const path = resolve(pathLike ?? "adversaria.config.yaml");
  try {
    const raw = await readFile(path, "utf8");
    const parsed = path.endsWith(".json") ? JSON.parse(raw) : yaml.load(raw);
    const expanded = expandEnvDeep(parsed);
    const config = configSchema.parse(expanded) as AdversariaConfig;
    return { ...DEFAULT_CONFIG, ...config };
  } catch (err) {
    if (pathLike) throw err;
    return DEFAULT_CONFIG;
  }
}
