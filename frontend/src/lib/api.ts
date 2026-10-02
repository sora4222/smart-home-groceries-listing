/**
 * Typed fetch wrappers for the Rust/Axum backend. Keep `lib/api/` the only
 * place that knows the backend's URL shape — components call these, never
 * `fetch` directly.
 *
 * One module per API domain under `lib/api/`; this file gathers them into the
 * `api` object and re-exports their types, so callers import from
 * `#/lib/api` without caring which file a type lives in.
 */
import { accessLogsApi } from "#/lib/api/access-logs";
import { groceryApi } from "#/lib/api/grocery";
import { itemRulesApi } from "#/lib/api/item-rules";
import { orderReviewApi } from "#/lib/api/order-review";
import { productsApi } from "#/lib/api/products";
import { purchasesApi } from "#/lib/api/purchases";
import { selectionsApi } from "#/lib/api/selections";
import { spendingApi } from "#/lib/api/spending";
import { triageApi } from "#/lib/api/triage";
import { trolleyHandoffsApi } from "#/lib/api/trolley-handoffs";
import { voiceApi } from "#/lib/api/voice";

export * from "#/lib/api/access-logs";
export { ApiError } from "#/lib/api/client";
export * from "#/lib/api/grocery";
export * from "#/lib/api/item-rules";
export * from "#/lib/api/order-review";
export * from "#/lib/api/products";
export * from "#/lib/api/purchases";
export * from "#/lib/api/selections";
export * from "#/lib/api/spending";
export * from "#/lib/api/triage";
export * from "#/lib/api/trolley-handoffs";
export * from "#/lib/api/voice";

export const api = {
	voice: voiceApi,
	triage: triageApi,
	grocery: groceryApi,
	itemRules: itemRulesApi,
	products: productsApi,
	selections: selectionsApi,
	trolleyHandoffs: trolleyHandoffsApi,
	orderReview: orderReviewApi,
	purchases: purchasesApi,
	spending: spendingApi,
	accessLogs: accessLogsApi,
};
