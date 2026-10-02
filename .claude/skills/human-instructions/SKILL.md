---
name: human-instructions
description: Use when telling the person (Jesse) to do something by hand — sign up, copy a value, change a setting on a website, log in, press a button in the app or a browser, allow a prompt, plug in or speak to a device — in docs/ or in chat. Not for telling them to run tests, builds, lint or other developer commands.
---

# Writing instructions a person must follow

Jesse finds complex wording hard. Low attention, working memory and
reading speed are part of it. Every instruction a person has to carry out
must be **easy to read, in the right order, and grouped by place**, so he can
finish one place before moving to the next.

## When this applies
- Anything only a person can do: accounts, passwords, passkeys, payments,
  pasting values into `.env`, website settings (Clerk, Cloudflare, Amazon,
  Woolworths), dragging a bookmark, allowing a browser prompt, speaking to
  an Echo, checking a page.
- Both places: the docs (`docs/human-setup.md`, `docs/using-the-app.md`) and
  a chat reply that ends with "you need to…".

**Not** for "run the tests", "run `make lint`", "rebuild" and other
developer checks — write those the normal way.

## Where it goes
| What | File |
|---|---|
| Done once to set the app up | `docs/human-setup.md` |
| Done again and again (a weekly shop, changing a delivery time) | `docs/using-the-app.md` |
| Done once because of one change (drag the bookmark again) | Chat **and** the "After an update" list in `docs/using-the-app.md` |
| Why it works that way, findings, API details | The feature doc (`docs/features/…`) — link to the human doc, never repeat the steps |

Before writing, **read the human doc it belongs in**. Put the new task where
the person already is — inside the part for that website or that screen —
not at the end. If a new step needs a page that an earlier step already
opened, add it to that earlier part and say "keep this tab open".

## Order the tasks
1. List every task, and for each: **where** it happens (home server
   terminal, a website, the app, Chrome on Woolworths, the Echo) and what it
   **needs first** (a value from another place, the app running).
2. Group tasks by place. Finish a place before leaving it.
3. Put a task that needs a value **after** the task that makes the value.
   Example: the Alexa address comes from Cloudflare, so Cloudflare comes
   before Amazon.
4. Avoid restarts: collect every value before the first start.
5. When a later task needs a page that is already open, say so at the end
   of the earlier part: "Keep this tab open. You come back to it in part 5."
6. Optional tasks go last, labelled **Optional**.

## Write each part like this
```markdown
## Part 3 — Cloudflare website (about 15 minutes)

**Where:** cloudflare.com, in your browser.
**You need:** `.env` open in your text editor.
**You get:** the tunnel token, and your Alexa address.

- [ ] 1. Go to [dash.cloudflare.com](https://dash.cloudflare.com) and log in.
- [ ] 2. Press **Networking**, then **Tunnels**.
- [ ] 3. Press **Create Tunnel**.

| Copy this | Paste it into `.env` after |
|---|---|
| The long text starting `eyJ` | `CLOUDFLARE_TUNNEL_TOKEN=` |

**It worked if:** the tunnel is listed.
**Keep this tab open.** You come back to it in part 5.
```

## Wording rules
- **One action per step.** Start with a verb: Press, Copy, Paste, Type,
  Open, Choose, Say.
- **Short sentences**, about 15 words or fewer. Everyday words.
- **Name buttons exactly as the screen does**, in bold: **Create Tunnel**.
  Check the real name (read the app code, or the site's current docs) — menus
  change. If unsure, say "it may be called X or Y".
- **Say where at the top** of every part, and what the person needs and gets.
- **Numbered checkboxes** (`- [ ] 1.`), so he can stop and find his place.
- **A time guess** per part, and "You can stop here" between parts.
- **A copy → paste table** whenever a value moves between places.
- **Code blocks only for text to copy exactly.** One command per block.
- **"It worked if:"** after each part — what he should see.
- **"If it goes wrong:"** at most three short fixes, the most likely first.
- **No jargon.** If a technical word is needed, explain it once in the
  "Words" list at the top of the doc. Avoid: e.g., i.e., N/A, etc., "simply",
  "just", double negatives, passive voice.
- **Why** gets one short line ("Why: so Amazon can reach your server.") or
  goes in the feature doc.
- **Warnings** in bold, just before the step they apply to, not at the end.
- Never put a secret value in a doc or a chat message. Name the setting.

## In a chat reply
- End with a short **"You need to do"** list, numbered, grouped by place,
  using the same rules. Link the doc part instead of repeating long steps.
- Ask the person to do only what an agent cannot do (logins, passkeys,
  payments, physical devices, approvals).

## Before you finish
- [ ] Every human task from this change is in the right doc and part.
- [ ] Tasks are grouped by place and ordered by what they need.
- [ ] No step has two actions. No sentence needs reading twice.
- [ ] Button and menu names match the screen.
- [ ] Feature docs link to the human doc instead of repeating steps.
