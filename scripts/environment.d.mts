/** Configuration sources and the replaceable secret-provider boundary. */
export function readSecret(reference: string, inherited: NodeJS.ProcessEnv): string;
export function selectedEnvironmentFile(inherited?: NodeJS.ProcessEnv): { file: string; required: boolean };
export function loadEnvironment(options: {
  file: string | URL;
  required?: boolean;
  inherited?: NodeJS.ProcessEnv;
  accepts?: (key: string) => boolean;
  resolve?: (reference: string, inherited: NodeJS.ProcessEnv) => string;
  environmentId?: string;
  readEnvironment?: (environmentId: string, inherited: NodeJS.ProcessEnv) => Record<string, string>;
}): Record<string, string>;
