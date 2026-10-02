# Feature: Spending Analysis

Status: **implemented.** The **Spending** page (`/analysis`) shows where the
grocery money went, from saved purchases (`FEATURE_PURCHASE_HISTORY.md`).
Steps for people: `docs/using-the-app.md` ("See your spending").

## What the page shows
- **Latest shop** — the newest saved shop, whatever the filters say.
- **Filters**, one row: **Dates** (This week · This month · This quarter ·
  All time · Choose days), **Store**, **Category**, item name + **Find**.
  Filters live in the page address, so a view can be bookmarked.
- **Views** (tabs):
  - **Over time** — line chart, **Weeks / Months / Quarters**; empty
    buckets are 0; totals include delivery.
  - **By item** — spend per item (grouped by item key), biggest first;
    **Prices paid** opens each price, at any store.
  - **By store** — every asked store, even at $0; items and delivery apart.
  - **By category** — biggest first; **Sort categories again** re-runs the
    category rules on every purchase.
- **Saved shops** — the newest shops, each with **Undo**.

## Rules
- Days are the household's calendar days in the browser's time zone
  (`tz` param, IANA name; unknown → 422). `from` > `to` → 422.
- A week starts Monday; quarters start Jan, Apr, Jul, Oct.
- At most 600 buckets in **Over time**.
- No suggestions or "you could save" advice — facts only.

## Endpoints (Clerk)

| Method | Path | Answer |
|---|---|---|
| `GET` | `/api/spending?from&to&store&item&category&period&tz` | `latest_order`, `categories`, items/delivery totals, `over_time`, `by_item`, `by_store`, `by_category` |
| `GET` | `/api/spending/item-prices?name=` | every purchase of that item, oldest first |

## Code
- Backend: `services/spending/` (`mod.rs`, `period.rs`, `range.rs`,
  `breakdown.rs` + tests), `routes/spending.rs`,
  `models/schemas/spending.rs`, `routes/extract.rs` `ValidatedQuery`.
- Frontend: `routes/analysis.tsx`, `components/analysis/`,
  `lib/api/spending.ts`, `lib/date-range.ts`, `lib/analysis-search.ts`,
  `lib/period-label.ts`.

## Tests
- `backend/tests/spending.rs`; `services/spending/*` unit tests.
- `frontend/src/components/analysis/__tests__/`, `lib/__tests__/`
  (date-range, analysis-search, spending-query, period-label).
- `frontend/e2e/spending.spec.ts` — fill, see it on Spending, Undo.
