# Using the grocery app

For jobs you do again and again. Setting up the first time:
[`human-setup.md`](human-setup.md).

| Job | Where | Time |
|---|---|---|
| [Add items by voice](#add-items-by-voice) | Echo, then the app | 1 min |
| [Check the Triage page](#check-the-triage-page) | The app | 1 min |
| [Do a Woolworths shop](#do-a-woolworths-shop) | The app, then Woolworths in Chrome | 10–15 min |
| [Change the delivery time](#change-the-delivery-time) | Woolworths in Chrome | 2 min |
| [Undo a shop saved by mistake](#undo-a-shop-saved-by-mistake) | The app | 1 min |
| [See your spending](#see-your-spending) | The app | 2 min |
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

## See who used the app

**Where:** the app. Do this if you think someone else is using the app.

- [ ] 1. Open **Logs**.
- [ ] 2. Read the list. The newest is at the top.
- [ ] 3. Each line shows the time, the page, and who it was.
- [ ] 4. Press **Load older** at the bottom to see more.

**Not signed in** is normal for Alexa and the health check. A lot of red
**401** or **404** lines from an address you do not know can mean a stranger
is trying the app. Ask Claude to look.

---

## Do a Woolworths shop

Two places, in this order: first **the app**, then **Woolworths in
Chrome**. Use Chrome for both, so the Woolworths tab opens next to the app.

### In the app

- [ ] 1. Open the **Grocery List** page.
- [ ] 2. For each item without a product: press **Compare prices**, then
  **Choose** on the product you want.
  - A product you bought before shows **Bought once** or **Bought 3
    times**. Press it to see the price you paid each time.
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
as **Added**, and a **Delivery:** line with the day and time. A few seconds
later it shows **Saved as bought**.

**Good to know**
- Products are **added on top** of what is already in your trolley.
- A product your Woolworths store cannot deliver is taken back out. The app
  shows it as **Not added**. Choose a different product for it next time.
- The bookmark keeps a delivery time you already picked, if it fits the day
  and time of day you chose.
- You have **30 minutes** to press the bookmark after **Send and open
  Woolworths**. After that, press **Send and open Woolworths** again.
- When the bookmark fills the trolley, the app saves the shop as bought.
  The added items leave your list. Items **Not added** stay on it.
- Pressed the bookmark by mistake? Press **Undo** in the **Send to
  Woolworths** panel. See [Undo a shop saved by
  mistake](#undo-a-shop-saved-by-mistake).

---

## Change the delivery time

**Where:** the Woolworths tab in Chrome.

- [ ] 1. At the top of the Woolworths page, press the delivery time.
- [ ] 2. Choose a new day and time.

You can do this any time before Woolworths' cut-off. The app does not need
to know.

---

## Undo a shop saved by mistake

**Where:** the app.

Do this if the app saved a shop you did not buy. Undo puts the items back on
your list, as they were. It also takes the shop out of your spending.

**Right after the bookmark:**

- [ ] 1. In the **Send to Woolworths** panel, press **Undo**.

**Later:**

- [ ] 1. Open **Spending** in the menu.
- [ ] 2. Find the shop under **Saved shops**. Each shows the day, store and
  total.
- [ ] 3. Press **Undo** next to it.

**It worked if:** the items are back on the **Grocery List** page.

Undo does not change your Woolworths trolley. Remove items there yourself if
you need to.

---

## See your spending

**Where:** the app.

- [ ] 1. Open **Spending** in the menu.
- [ ] 2. Choose the **Dates**: **This week**, **This month**, **This
  quarter**, **All time** or **Choose days**.
- [ ] 3. Optional: choose a **Store** or a **Category**.
- [ ] 4. Optional: type part of an item name, then press **Find**.
- [ ] 5. Press a view:
  - **Over time**: a line of your spending. Press **Weeks**, **Months** or
    **Quarters** to change the steps.
  - **By item**: what each item cost you. Press **Prices paid** to see each
    price.
  - **By store**: Woolworths and Coles side by side, with delivery shown
    apart.
  - **By category**: Dairy & eggs, Bakery, and the rest.

**It worked if:** **Latest shop** at the top shows your last shop.

**Good to know**
- Totals include delivery fees.
- Each item gets a category when the shop is saved. If the categories look
  wrong after an app update, press **Sort categories again** on **By
  category**.

---

## After an app update

Claude tells you when an update needs one of these. Each is quick.

| When | Do this |
|---|---|
| The bookmark's code changed | Open **Send to Woolworths** in the app. Drag **Fill Woolworths trolley** to the bookmarks bar again. Delete the old bookmark (right-click it → **Delete**). |
| `STORE_TAB_SECRET` or the app's address changed | Same as above: drag the bookmark again. |
| The database changed | Nothing. The app updates itself when it starts. |
| `.env` got a new setting | Run `make setup-env`. Then fill any value Claude names. Then `make down` and `make up`. |

---

## If something goes wrong

| What you see | What to do |
|---|---|
| "Open woolworths.com.au first" | Press the bookmark on the Woolworths tab, not the app. |
| "Nothing to add" | Press **Send and open Woolworths** in the app first. Then press the bookmark within 30 minutes. |
| "The grocery app refused (HTTP 401)" | The bookmark is old. Drag it again (see [After an app update](#after-an-app-update)). |
| "Fill trolley failed" with a network error | Check the app is running. Check you pressed **Allow** on Chrome's local network question. |
| "No delivery time reserved" | Pick a time on Woolworths yourself (see [Change the delivery time](#change-the-delivery-time)). |
| **Saved as bought** never shows | Check the app is running. The items stay on your list until it is saved. |
| A product says **Not added** | Woolworths cannot deliver it right now. Choose another product for that item. |
| An item says "The checker could not be reached" | The model is not running. In the terminal, run `make up`. If you use Ollama, also run `docker compose exec ollama ollama pull llama3.2`. Then press **Accept** on the item in **Triage**. |
| Alexa does not add anything | See [human-setup.md, part 7](human-setup.md#part-7--test-with-your-echo-about-5-minutes). |
| Something else | Ask Claude. Say what you pressed and what the screen said. |
