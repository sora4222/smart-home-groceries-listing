# Operating the app

Recurring workflows once the stack is up. Audience: software developers.
One-time provisioning is in [`human-setup.md`](human-setup.md); behaviour
and API details are in [`features/`](features/).

| Workflow | Surface |
|---|---|
| [Voice intake: add](#voice-intake-add) | Echo, then the web app |
| [Voice intake: remove, reduce, undo](#voice-intake-remove-reduce-undo) | Echo |
| [Triage review](#triage-review) | Web app `/triage` |
| [Woolworths shop](#woolworths-shop) | Web app, then woolworths.com.au in Chrome |
| [Change the delivery window](#change-the-delivery-window) | woolworths.com.au |
| [Coles shop](#coles-shop) | Web app, then coles.com.au in Chrome |
| [Undo a recorded purchase](#undo-a-recorded-purchase) | Web app |
| [Spending analysis](#spending-analysis) | Web app `/spending` |
| [After an update](#after-an-update) | Varies |
| [Troubleshooting](#troubleshooting) | — |

---

## Voice intake: add

"Alexa, ask grocery list to add 2 milk." creates a **pending** intake
request (`POST /api/intake/alexa`). Nothing reaches the list until someone
accepts it.

1. Open **Pending Requests**.
2. Correct the parsed name or quantity if needed, then **Accept**.

If the card is missing, triage held or rejected it; see
[Triage review](#triage-review).

---

## Voice intake: remove, reduce, undo

These apply immediately (no Pending Requests step), since they can only
shrink the order. Each one is recorded so it can be reverted. Spec:
[`features/FEATURE_VOICE.md`](features/FEATURE_VOICE.md#removing-reducing-and-undo-by-voice).

| Utterance ("Alexa, ask grocery list to …") | Effect |
|---|---|
| "remove milk" / "take milk off the shopping list" | Deletes the item |
| "remove two milk" | Decrements quantity by 2; deletes it at ≤ 0 |
| "reduce milk" | Decrements by 1 |
| "reduce milk by three" | Decrements by 3 |
| "undo" / "put it back" | Reverts the newest remove/reduce from the last 30 minutes |

- After a remove or reduce the session stays open, so a bare "undo" works
  straight away. Repeating "undo" steps further back.
- Matching is the item name ignoring case and whitespace, plus the simple
  plural/singular form ("eggs" ↔ "egg"). No fuzzy matching: an unmatched name
  changes nothing, and Alexa says so.
- Only `active` items change. Once the list is committed (**Ready to
  order**), Alexa refuses until the list is released in the web app.
- Pending intake requests are not on the list yet; reject those in
  **Pending Requests** instead.
- The web list does not live-update on voice changes; reload the page.

---

## Triage review

The intake classifier holds low-confidence items and rejects non-grocery
items (`FEATURE_TRIAGE.md`). The **Triage** nav badge counts held items.

1. Open **Triage → Held for review** and read each card's reason.
2. **Accept** moves the item to Pending Requests (it still needs a second
   **Accept** there). **Reject** discards it.
3. Optionally scan **Rejected** for false negatives.

---

## Woolworths shop

Use Chrome for both the web app and the store tab.

**Web app**
1. On **Grocery List**, pick a product for each item: **Compare prices →
   Choose**. Previously bought products are badged (**Bought once**,
   **Bought 3 times**) with the prices paid.
2. **Ready to order** commits the list. **Order** shows live prices and the
   total.
3. **Send to Woolworths**, choose **Delivery day** (default **Tomorrow**) and
   **Time of day** (default **Any time**), then **Send and open
   Woolworths**. This creates a trolley handoff and opens a store tab.

**Store tab**
4. Sign in if prompted, then run the **Fill Woolworths trolley**
   bookmarklet. First run: allow Chrome's local-network access prompt.
5. The bookmarklet reports products added and the reserved delivery window.
6. Review the trolley and pay on woolworths.com.au. The app never handles
   payment.

**Verify:** the **Send to Woolworths** panel lists each line as **Added**
with a **Delivery:** line, then shows **Saved as bought**.

Behaviour (`FEATURE_TROLLEY_HANDOFF.md`, `FEATURE_PURCHASE_HISTORY.md`):
- Products are added on top of the existing trolley contents.
- Lines the store can't deliver are removed again and shown as **Not
  added**; those items stay on the list.
- An already-reserved delivery window is kept if it fits the requested day
  and time of day.
- The handoff expires 30 minutes after **Send and open Woolworths**.
- A successful fill records a purchase order and moves the added items to
  `ordered`. It can be undone (below).

---

## Change the delivery window

On woolworths.com.au, open the delivery time in the header and pick another
slot before the store's cut-off. The app does not track this.

---

## Coles shop

Same as the Woolworths shop, with these differences:

**Web app**
1. **Send to Coles**, then **Send and open Coles**. There is no delivery
   choice: the Coles bookmarklet cannot reserve a window yet.

**Store tab**
2. Sign in if prompted, then run **Fill Coles trolley**. It checks you are
   signed in and a store is selected **before** claiming the handoff, so a
   refusal leaves the handoff waiting; fix it and run it again.
3. Reload the page and open the trolley to review it.
4. Pick a delivery window on coles.com.au, then pay there.

**Verify:** the **Send to Coles** panel lists each line as **Added**, then
**Saved as bought**.

The Coles trolley calls were not verified live before release
(`FEATURE_TROLLEY_HANDOFF.md`, "Coles"). If the first run fails, report the
exact message.

---

## Undo a recorded purchase

Deletes the purchase order (and its spending data) and restores each item
to the status it had before. The Woolworths trolley is not touched.

- Right after a fill: **Undo** in the **Send to Woolworths** (or **Send to
  Coles**) panel.
- Later: **Spending → Saved shops → Undo** on the order.

**Verify:** the items are back on **Grocery List**.

---

## Spending analysis

**Spending** filters by **Dates** (**This week**, **This month**, **This
quarter**, **All time**, **Choose days**), **Store**, **Category** and an
item-name search (**Find**). Views:

- **Over time**: totals bucketed by **Weeks**, **Months** or **Quarters**.
- **By item**: spend per item; **Prices paid** lists each unit price.
- **By store**: Woolworths vs Coles, delivery fees broken out.
- **By category**: e.g. Dairy & eggs, Bakery.

Totals include delivery fees. Categories are assigned when a purchase is
recorded; after a categoriser change, use **Sort categories again** on **By
category** to recompute them.

---

## After an update

| Change | Action |
|---|---|
| Bookmarklet source changed | Re-drag **Fill Woolworths trolley** from **Send to Woolworths** (and **Fill Coles trolley** from **Send to Coles**); delete the old bookmarks. |
| `STORE_TAB_SECRET` or the app origin changed | Re-drag both bookmarklets (they embed both values). |
| New migration | None; migrations run on backend start. |
| New `.env` key | `make setup-env`, fill any non-generated value, then `make down && make up`. |
| Alexa interaction model changed | Update the JSON in the Alexa console and rebuild ([`human-setup.md` §4](human-setup.md#4-alexa-skill)). |

---

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| "Open woolworths.com.au first" / "Open coles.com.au first" | Bookmarklet run outside that store's tab. |
| "Nothing to add" | No open handoff: press **Send and open Woolworths**, then run the bookmarklet within 30 minutes. |
| "The grocery app refused (HTTP 401)" | Stale bookmarklet secret; re-drag it. |
| "Fill trolley failed" (network error) | Backend down, or Chrome's local-network prompt was denied. |
| "No delivery time reserved" | Pick a slot manually on the store's site (always the case for Coles). |
| "Log in to Coles first" | Coles trolley read returned 401/403; sign in and re-run. Nothing was claimed. |
| "Choose your delivery address on Coles first" | No Coles store selected; set a delivery address and re-run. |
| "Coles is still loading" | Coles page config not ready; reload and re-run. |
| **Saved as bought** never appears | Backend unreachable when the report was posted; items stay on the list. |
| A line shows **Not added** | Not deliverable from your store right now; choose another product. |
| Triage says "The checker could not be reached" | Model server down: `make up` (Ollama: also `docker compose exec ollama ollama pull llama3.2`), then **Accept** in Triage. |
| Alexa: "I couldn't find … on the list" | No `active` item matched; check the exact name on **Grocery List**. |
| Alexa: "The list is locked for purchase" | The list is committed; release it in the web app first. |
| Alexa adds/removes nothing | [`human-setup.md` §7](human-setup.md#7-smoke-test-the-skill). |
