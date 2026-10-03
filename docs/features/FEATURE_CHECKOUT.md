# Feature: Checkout (up to the payment page)

Status: **built as a confirmable step after the trolley fill.** The app never
places an order, stores card details, or enters payment.

## What happens
1. **Send to Woolworths / Coles** and the bookmarklet fill the household's
   own trolley (`FEATURE_TROLLEY_HANDOFF.md`).
2. When at least one product was added, the bookmarklet shows its result as a
   question: **Go to checkout now?** (`window.confirm`).
3. **OK** opens the store's checkout page in the same tab
   (`location.assign`). **Cancel** leaves the person on the trolley.
4. The person reviews the order, the delivery time and payment on the store's
   website and places the order there. The app stops at step 3.

| Store | Checkout path | Checked live |
|---|---|---|
| Woolworths | `/shop/checkout` | No (build workspace can't run the logged-in flow) |
| Coles | `/checkout` | No (build workspace can't reach coles.com.au) |

Nothing is offered when nothing was added (`checkout: null`).

## Why not the spec's server-side checkout
The spec's first choices — an official order API, or `chromiumoxide`
logging in with stored credentials — are not possible today:
- Neither store has a public order API.
- Woolworths login is passkeys + MFA inside its JavaScript; the server cannot
  log in (`docs/FEAT_WOOLWORTHS_ACCESS.md`). Coles is assumed the same.
So checkout runs in the household's own logged-in tab, like the fill. Store
credentials and rewards cards (`/settings/stores`) are therefore **not
built**: nothing would use them. Rewards cards are expected to apply when they
are linked to the store account the household is signed in to.

## Code
- `frontend/src/lib/store-tab/store-tab.ts` — `FillTrolleyResult.checkout`.
- `fill-woolworths-trolley.ts`, `fill-coles-trolley.ts` — the offer.
- `bookmarklet.ts` — shows the question and opens the page.

## Tests
`frontend/src/lib/__tests__/fill-woolworths-trolley.test.ts` (offered only
when something was added; OK opens `/shop/checkout`; the built bookmarklet
asks the question), `fill-coles-trolley.test.ts` (the Coles question).

## Not built
- Detecting a saved payment method before checkout.
- Order tracking (spec: low priority).
- Server-side checkout and stored store credentials (see above).
