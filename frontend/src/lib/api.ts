/**
 * Typed fetch wrappers for the Rust/Axum backend. Keep `lib/api/` the only
 * place that knows the backend's URL shape — components call these, never
 * `fetch` directly.
 *
 * One module per API domain under `lib/api/`; this file gathers them into the
 * `api` object and re-exports their types, so callers import from
 * `#/lib/api` without caring which file a type lives in.
 */
import { groceryApi } from "#/lib/api/grocery";
import { itemRulesApi } from "#/lib/api/item-rules";
import { productsApi } from "#/lib/api/products";
import { selectionsApi } from "#/lib/api/selections";
import { trolleyHandoffsApi } from "#/lib/api/trolley-handoffs";
import { voiceApi } from "#/lib/api/voice";

export { ApiError } from "#/lib/api/client";
export * from "#/lib/api/grocery";
export * from "#/lib/api/item-rules";
export * from "#/lib/api/products";
export * from "#/lib/api/selections";
export * from "#/lib/api/trolley-handoffs";
export * from "#/lib/api/voice";

export const api = {
	voice: voiceApi,
	grocery: groceryApi,
	itemRules: itemRulesApi,
	products: productsApi,
	selections: selectionsApi,
	trolleyHandoffs: trolleyHandoffsApi,
};
