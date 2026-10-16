/** Configuration sources and the replaceable secret-provider boundary. */
export function readSecret(reference: string, inherited: NodeJS.ProcessEnv): string;
export function loadEnvironment(options: {
  file: string | URL;
  required?: boolean;
  inherited?: NodeJS.ProcessEnv;
  accepts?: (key: string) => boolean;
  resolve?: (reference: string, inherited: NodeJS.ProcessEnv) => string;
}): Record<string, string>;
