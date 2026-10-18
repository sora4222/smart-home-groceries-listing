/**
 * Decide whether `make install` must provide a local Ollama, and which model.
 *
 *   grocery-env json triage | node triage-setup.mjs   →  "ollama llama3.2" | "none"
 *
 * Only Ollama is installed automatically. llama.cpp, vLLM and OpenAI are
 * pointed at with INTAKE_LLM_BASE_URL (docs/production.md).
 */
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

const DOCKER_HOSTS = new Set(['ollama', 'llamacpp', 'vllm', 'host.docker.internal']);
const LOCAL_HOSTS = new Set(['localhost', '127.0.0.1', '[::1]', '::1']);
const DEFAULT_MODEL = 'llama3.2';

/** Returns `{ ollama: false }` or `{ ollama: true, model }`; throws on a Compose hostname. */
export function triagePlan({ INTAKE_LLM_PROVIDER: provider = 'ollama', INTAKE_LLM_BASE_URL: base = '', INTAKE_LLM_MODEL: model = '' }) {
  if (base) {
    const host = new URL(base).hostname;
    if (DOCKER_HOSTS.has(host)) {
      throw new Error(`INTAKE_LLM_BASE_URL names "${host}", a Docker Compose host; use http://127.0.0.1:<port>/v1`);
    }
    if (provider === 'ollama' && !LOCAL_HOSTS.has(host)) return { ollama: false };
  }
  if (provider !== 'ollama') return { ollama: false };
  return { ollama: true, model: model || DEFAULT_MODEL };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const plan = triagePlan(JSON.parse(readFileSync(0, 'utf8')));
    process.stdout.write(plan.ollama ? `ollama ${plan.model}\n` : 'none\n');
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
