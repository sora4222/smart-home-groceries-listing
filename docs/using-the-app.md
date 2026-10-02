# Using the grocery app

For jobs you do again and again. Setting up the first time:
[`human-setup.md`](human-setup.md).

| Job | Where | Time |
|---|---|---|
| [Add items by voice](#add-items-by-voice) | Echo, then the app | 1 min |
| [Check the Triage page](#check-the-triage-page) | The app | 1 min |
| [Do a Woolworths shop](#do-a-woolworths-shop) | The app, then Woolworths in Chrome | 10–15 min |
| [Change the delivery time](#change-the-delivery-time) | Woolworths in Chrome | 2 min |
| [Do a Coles shop](#do-a-coles-shop) | The app, then Coles in Chrome | 10–15 min |
| [After an app update](#after-an-app-update) | Depends on the update | 2 min |
| [If something goes wrong](#if-something-goes-wrong) | — | — |

---

## Add items by voice

**Where:** next to your Echo, then the app.

- [ ] 1. Say: *"Alexa, ask grocery list to add 2 milk."*
- [ ] 2. In the app, open **Pending Requests**.
- [ ] 3. Fix the name or number if Alexa heard it wrong.
- [ ] 4. Press **Accept**.

**It worked if:** the item is on the **Grocery List** page.

**Not in Pending Requests?** The checker may have stopped it. See the next
part.

---

## Check the Triage page

**Where:** the app. Do this when **Triage** in the menu shows a red number,
or when an item you said is not in **Pending Requests**.

The checker asks "would a supermarket sell this?". It stops items it says
no to, and items it is not sure about.

- [ ] 1. Open **Triage**.
- [ ] 2. Look at **Held for review**. These need you.
- [ ] 3. Read the reason on each card.
- [ ] 4. Press **Accept** if you want the item. It moves to **Pending
  Requests**.
- [ ] 5. Press **Reject** if you do not want it.
- [ ] 6. Optional: look at **Rejected** too, in case the checker was wrong.

**It worked if:** the red number on **Triage** is gone.

**Accept does not add it to the list yet.** Open **Pending Requests** and
press **Accept** there too.

---

## Do a Woolworths shop

Two places, in this order: first **the app**, then **Woolworths in
Chrome**. Use Chrome for both, so the Woolworths tab opens next to the app.

### In the app

- [ ] 1. Open the **Grocery List** page.
- [ ] 2. For each item without a product: press **Compare prices**, then
  **Choose** on the product you want.
- [ ] 3. Press **Ready to order**.
- [ ] 4. Optional: open **Order** to see today's prices and the total.
- [ ] 5. Press **Send to Woolworths**.
- [ ] 6. Pick a **Delivery day**. **Tomorrow** is already picked.
- [ ] 7. Pick a **Time of day**. **Any time** is already picked.
- [ ] 8. Press **Send and open Woolworths**. A Woolworths tab opens.

### In the Woolworths tab

- [ ] 9. Log in if Woolworths asks (passkey or 1Password).
- [ ] 10. Press the **Fill Woolworths trolley** bookmark.
  - First time only: if Chrome asks to reach devices on your local network,
    press **Allow**.
- [ ] 11. Read the message. It says how many products were added and the
  delivery time. Press **OK**.
- [ ] 12. Check the trolley.
- [ ] 13. Optional: change the delivery time (see below).
- [ ] 14. Pay on Woolworths. The app never pays for you.

**It worked if:** the app's **Send to Woolworths** panel shows each product
as **Added**, and a **Delivery:** line with the day and time.

**Good to know**
- Products are **added on top** of what is already in your trolley.
- A product your Woolworths store cannot deliver is taken back out. The app
  shows it as **Not added**. Choose a different product for it next time.
- The bookmark keeps a delivery time you already picked, if it fits the day
  and time of day you chose.
- You have **30 minutes** to press the bookmark after **Send and open
  Woolworths**. After that, press **Send and open Woolworths** again.

---

## Change the delivery time

**Where:** the Woolworths tab in Chrome.

- [ ] 1. At the top of the Woolworths page, press the delivery time.
- [ ] 2. Choose a new day and time.

You can do this any time before Woolworths' cut-off. The app does not need
to know.

---

## Do a Coles shop

Two places, in this order: first **the app**, then **Coles in Chrome**.
Use Chrome for both, so the Coles tab opens next to the app.

### In the app

- [ ] 1. Open the **Grocery List** page.
- [ ] 2. For each item without a product: press **Compare prices**, then
  **Choose** on the product you want.
- [ ] 3. Press **Ready to order**.
- [ ] 4. Optional: open **Order** to see today's prices and the total.
- [ ] 5. Press **Send to Coles**.
- [ ] 6. Press **Send and open Coles**. A Coles tab opens.

### In the Coles tab

- [ ] 7. Log in if Coles asks. Chrome fills in your saved login.
- [ ] 8. Press the **Fill Coles trolley** bookmark.
  - First time only: if Chrome asks to reach devices on your local network,
    press **Allow**.
- [ ] 9. Read the message. It says how many products were added. Press **OK**.
- [ ] 10. Reload the Coles page. Open the trolley (top right).
- [ ] 11. Check the trolley.
- [ ] 12. Choose a delivery time on Coles. The app cannot do this for Coles yet.
- [ ] 13. Pay on Coles. The app never pays for you.

**It worked if:** the app's **Send to Coles** panel shows each product as
**Added**.

**Good to know**
- Products are **added on top** of what is already in your trolley.
- You have **30 minutes** to press the bookmark after **Send and open
  Coles**. After that, press **Send and open Coles** again.
- If the bookmark says "Log in" or "Choose your delivery address", nothing
  was used up. Do that on Coles, then press the bookmark again.

---

## After an app update

Claude tells you when an update needs one of these. Each is quick.

| When | Do this |
|---|---|
| The bookmark's code changed | Open **Send to Woolworths** in the app. Drag **Fill Woolworths trolley** to the bookmarks bar again. Delete the old bookmark (right-click it → **Delete**). For Coles, do the same with **Send to Coles** and **Fill Coles trolley**. |
| `STORE_TAB_SECRET` or the app's address changed | Same as above: drag both bookmarks again. |
| The database changed | Nothing. The app updates itself when it starts. |
| `.env` got a new setting | Run `make setup-env`. Then fill any value Claude names. Then `make down` and `make up`. |

---

## If something goes wrong

| What you see | What to do |
|---|---|
| "Open woolworths.com.au first" or "Open coles.com.au first" | Press the bookmark on that store's tab, not the app. |
| "Nothing to add" | Press **Send and open Woolworths** (or **Coles**) in the app first. Then press the bookmark within 30 minutes. |
| "Log in to Coles first" | Log in on the Coles tab. Then press the bookmark again. |
| "Choose your delivery address on Coles first" | At the top of the Coles page, choose **Delivery** to your address. Then press the bookmark again. |
| "Coles is still loading" | Reload the Coles page. Wait for it to finish. Then press the bookmark again. |
| A Coles product says **Not added** | Coles cannot sell it at your store right now. Choose another product for that item. |
| "The grocery app refused (HTTP 401)" | The bookmark is old. Drag it again (see [After an app update](#after-an-app-update)). |
| "Fill trolley failed" with a network error | Check the app is running. Check you pressed **Allow** on Chrome's local network question. |
| "No delivery time reserved" | Pick a time on Woolworths yourself (see [Change the delivery time](#change-the-delivery-time)). |
| A product says **Not added** | Woolworths cannot deliver it right now. Choose another product for that item. |
| An item says "The checker could not be reached" | The model is not running. In the terminal, run `make up`. If you use Ollama, also run `docker compose exec ollama ollama pull llama3.2`. Then press **Accept** on the item in **Triage**. |
| Alexa does not add anything | See [human-setup.md, part 7](human-setup.md#part-7--test-with-your-echo-about-5-minutes). |
| Something else | Ask Claude. Say what you pressed and what the screen said. |
