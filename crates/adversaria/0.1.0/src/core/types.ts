export type ProviderName = "openai" | "anthropic" | "ollama" | string;

export type SuiteCategory =
  | "prompt-injection"
  | "jailbreaks"
  | "role-confusion"
  | "data-exfiltration"
  | string;

export interface Payload {
  id: string;
  name: string;
  prompt: string;
  tags?: string[];
}

export interface Suite {
  id: string;
  name: string;
  category: SuiteCategory;
  payloads: Payload[];
}

export interface ProviderResponse {
  content: string;
  raw?: unknown;
}

export interface Provider {
  name: ProviderName;
  sendPrompt(model: string, prompt: string): Promise<ProviderResponse>;
}

export interface AdversariaConfig {
  provider: {
    name: ProviderName;
    model: string;
  };
  providers?: {
    openai?: { apiKey?: string; baseUrl?: string };
    anthropic?: { apiKey?: string; baseUrl?: string };
    ollama?: { baseUrl?: string };
    [key: string]: { [k: string]: unknown } | undefined;
  };
  suites?: string[];
  payloads?: Record<string, string>;
  plugins?: string[];
  reportDir?: string;
}

export interface AttackTrace {
  suiteId: string;
  suiteCategory: SuiteCategory;
  payloadId: string;
  payloadName: string;
  prompt: string;
  response: string;
  riskScore: number;
  verdict: "pass" | "fail";
  indicators: string[];
}

export interface ReportSummary {
  category: SuiteCategory;
  total: number;
  failures: number;
  avgRisk: number;
}

export interface Report {
  version: string;
  runId: string;
  timestamp: string;
  provider: ProviderName;
  model: string;
  riskScore: number;
  summaryByCategory: ReportSummary[];
  traces: AttackTrace[];
}
