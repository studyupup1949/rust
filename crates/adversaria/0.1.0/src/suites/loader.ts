import { readFile, readdir } from "node:fs/promises";
import { resolve, join, extname } from "node:path";
import { fileURLToPath } from "node:url";
import yaml from "js-yaml";
import type { Suite, Payload } from "../core/types.js";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const BUILTIN_DIR = resolve(__dirname, "payloads");

interface RawSuite {
  id: string;
  name: string;
  category: string;
  payloads: Array<{ id: string; name: string; prompt: string; tags?: string[] }>;
}

function parseSuiteFile(raw: unknown): Suite {
  const data = raw as RawSuite;
  const payloads: Payload[] = data.payloads.map((p) => ({
    id: p.id,
    name: p.name,
    prompt: p.prompt,
    tags: p.tags,
  }));
  return { id: data.id, name: data.name, category: data.category, payloads };
}

export async function loadSuiteFromFile(filePath: string): Promise<Suite> {
  const content = await readFile(filePath, "utf8");
  const ext = extname(filePath);
  const raw = ext === ".json" ? JSON.parse(content) : yaml.load(content);
  return parseSuiteFile(raw);
}

export async function loadBuiltinSuites(filter?: string[]): Promise<Suite[]> {
  const files = await readdir(BUILTIN_DIR);
  const yamlFiles = files.filter((f) => f.endsWith(".yaml") || f.endsWith(".yml"));

  const suites: Suite[] = [];
  for (const file of yamlFiles) {
    const suite = await loadSuiteFromFile(join(BUILTIN_DIR, file));
    if (!filter || filter.includes(suite.id) || filter.includes(suite.category)) {
      suites.push(suite);
    }
  }
  return suites;
}

export async function loadSuites(
  filter?: string[],
  overrides?: Record<string, string>,
): Promise<Suite[]> {
  const suites = await loadBuiltinSuites(filter);

  if (overrides) {
    for (const [suiteId, path] of Object.entries(overrides)) {
      const idx = suites.findIndex((s) => s.id === suiteId);
      const custom = await loadSuiteFromFile(resolve(path));
      if (idx >= 0) {
        suites[idx] = custom;
      } else {
        suites.push(custom);
      }
    }
  }

  return suites;
}
