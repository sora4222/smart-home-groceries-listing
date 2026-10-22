/** Which configuration keys each production service may receive. */
export interface ComponentSpec {
  keys?: string[];
  prefixes?: string[];
  rename?: Record<string, string>;
}
export const COMPONENTS: Record<string, ComponentSpec>;
export function component(name: string): ComponentSpec;
export function accepts(spec: ComponentSpec, key: string): boolean;
export function renamed(spec: ComponentSpec, values: Record<string, string>): Record<string, string>;
