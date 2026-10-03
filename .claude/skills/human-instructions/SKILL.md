---
name: human-instructions
description: Use when documenting something a person must do by hand — sign up, copy a value into .env, change a setting in a vendor console, log in, run a bookmarklet, allow a browser prompt, speak to an Echo — in docs/ or in chat. Not for telling them to run tests, builds, lint or other developer commands.
---

# Writing steps a person must carry out

Two audiences, two registers:

| Where | Reader | Register |
|---|---|---|
| `docs/` (`human-setup.md`, `using-the-app.md`, `features/`) | A software developer | Developer terminology, concise |
| A chat reply to Jesse | Jesse | Short, simple sentences |

Jesse decided (2026-10-02) that repository docs are written for software
developers, in their terminology. Chat replies stay plain because Jesse
finds long or complex text hard to read and remember.

## When this applies
- Anything only a person can do: accounts, passkeys, payments, values for
  `.env`, vendor console settings (Clerk, Cloudflare, Alexa developer
  console, Woolworths), installing a bookmarklet, allowing a browser
  prompt, speaking to an Echo, checking a page.
- Both places: the docs and a chat reply that ends with "you need to…".

**Not** for "run the tests", `make lint`, rebuilds and other developer
checks — write those the normal way.

## Where it goes
| What | File |
|---|---|
| One-time provisioning | `docs/human-setup.md` (numbered sections, §1–§8) |
| Recurring workflows (a shop, voice commands, undo) | `docs/using-the-app.md` |
| Needed once because of one change (re-drag the bookmarklet, rebuild the Alexa model) | Chat **and** the "After an update" table in `docs/using-the-app.md` |
| Why it works that way, API, data, findings | The feature spec (`docs/features/…`) — link to the setup/ops doc, never repeat the steps |

Read the target doc first. Put the new step in the section for that console
or screen, not at the end.

## Order (docs)
1. List each task with **where** it happens (shell, a vendor console, the
   web app, the store tab, the Echo) and its **inputs** (a value produced
   elsewhere, the stack running).
2. Group by place; finish one place before moving on.
3. Order by dependency: a step that consumes a value comes after the step
   that produces it (the Cloudflare hostname before the Alexa endpoint).
4. Collect every `.env` value before the first `make up`, so the stack starts
   once.
5. Optional sections go last, marked **Optional**.

## Writing for developers (docs)
- Use the real terms: environment variable, `.env` key, interaction model,
  intent, slot, utterance, endpoint, ingress route, Compose profile,
  bookmarklet, JWKS, migration, HTTP status codes.
- State the outcome and the reason in one line where it helps ("The tunnel
  gives Amazon an HTTPS ingress to the sidecar without opening a port").
- Numbered steps for sequences; one action per step is still preferred.
- **Copy → paste tables** whenever a value moves between places (`Clerk
  value` → `.env key`).
- Name UI controls exactly as the screen does, in bold (**Create Tunnel**).
  Check the current label (read the app code, or the vendor's docs). If a
  vendor has two UIs, give both ("older UI: …").
- Code blocks only for text to copy verbatim; one command per block.
- **Verify:** after each section — the observable result.
- **Troubleshooting:** at most three likely causes, most likely first.
- Security warnings in bold, right before the step they apply to.
- Link the spec for behaviour instead of explaining it inline.
- Never put a secret value in a doc. Name the key.

## In a chat reply to Jesse
- Lead with the answer. Short sentences, everyday words, no jargon.
- End with a numbered **"You need to do"** list, grouped by place, one
  action per step, buttons named exactly. Link the doc section rather than
  repeating long steps.
- Ask only for what an agent cannot do (logins, passkeys, payments, physical
  devices, approvals).

## Before you finish
- [ ] Every manual step from this change is in the right doc and section.
- [ ] Steps are grouped by place and ordered by their inputs.
- [ ] Control names match the current UI.
- [ ] Feature specs link to the setup/ops doc instead of repeating steps.
- [ ] Docs use developer terminology; chat stays plain.
