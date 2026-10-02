# Alexa Bridge Agent Guide

## What this is
The **only** Python in this repository. Everything else — the whole backend —
is Rust. This sidecar exists solely because Amazon's request signing and skill
request model live in the `ask-sdk` Python SDK with no maintained Rust
equivalent.

## Non-negotiable rule
**No business logic here.** If you are about to write a rule about the grocery
list — duplicate detection, quantity merging, what counts as an acceptable
item, anything touching PostgreSQL — it belongs in the Rust backend
(`backend/src/services/`). This process may only:

1. verify the request came from Amazon (via `ask-sdk`),
2. read the intent slots,
3. forward them over local HTTP,
4. say what happened.

A reviewer should be able to delete this directory, verify Alexa in Rust, and
lose nothing but the verification.

## Stack
Python 3.13 · Flask · `ask-sdk-core` · `ask-sdk-webservice-support` ·
`flask-ask-sdk` · `requests` · gunicorn · `uv` · `ruff` · `pytest`

## Structure
```
src/alexa_bridge/
├── app.py         # Flask app, SkillBuilder + SkillAdapter wiring, /health
├── config.py      # Settings from env; fails closed on a missing secret
├── handlers.py    # Add, launch, help, stop handlers — thin, no rules
├── list_change_handlers.py  # Remove/reduce and Undo handlers — thin, no rules
├── parsing.py     # Pure slot → (item, quantity); where the tests bite
├── speech.py      # Pure: what Alexa says after a remove, reduce or undo
├── backend.py     # Posts added items to the Rust backend
└── list_changes.py# Posts removes and undos; maps 404/409 to exceptions
tests/             # pytest; no network, no real Alexa, no real backend
```

## Rules
1. **File length** — ≤ 300 lines.
2. **TDD** — `parsing.py` and `backend.py` are pure or injectable for exactly
   this reason. Write the test first.
3. **No network in tests** — inject a fake session (see `RecordingSession` in
   `tests/test_backend.py`). Never reach Amazon, the backend or OpenAI in CI.
4. **Never weaken verification** — do not pass `verify_signature=False` or
   `verify_timestamp=False` to `SkillAdapter`/`WebserviceSkillHandler`, not
   even behind a debug flag. This process is the internet-facing boundary.
5. **Secrets** — `ALEXA_BRIDGE_SECRET` travels in the `X-Bridge-Secret`
   header and nowhere else: never in a URL, a body, or a log line.
6. **No suggesting items** — reducing marketing-driven buying is a project
   goal. Handlers confirm what was asked for and nothing more.
7. **Dependencies** — get any needed; use Context7 MCP for current docs.
8. **Docs** — document every public function and module.

## Adding an intent
1. Add the intent and slots to the skill's interaction model (Alexa console).
2. Write the failing test in `tests/`.
3. Parse the slots in `parsing.py` (pure function).
4. Add a handler in `handlers.py` that forwards and speaks.
5. Register it in `app.py`'s `build_skill`.
6. If the backend needs a new field, change `backend/src/models/schemas.rs`
   first and its tests with it.

## Remove, reduce and undo
`RemoveItemIntent` (no quantity = whole item), `ReduceItemIntent` (default
1) and `UndoIntent` forward to `POST /api/intake/alexa/remove` and `/undo`.
The backend matches the item, applies the change and records it for Undo —
see `docs/features/FEATURE_VOICE.md`. This side only reads slots, forwards
Alexa's request id as `external_id`, and speaks the answer (`speech.py`).
After a change the session is left open so a bare "undo" works.

## Alexa list events
The current skill uses custom intents (`AddItemIntent` and the three above). Amazon also offers household
list events (`AlexaHouseholdListEvent.ItemsCreated`), which would make
"Alexa, add milk to my shopping list" work without a custom invocation. That
path needs the List API and a permissions grant; it is unbuilt. Confirm the
current permissions and event delivery against Amazon's docs via Context7
before starting, and keep the backend contract (`POST /api/intake/alexa`)
unchanged so only this directory moves.

## The oscrypto pin
`pyproject.toml` pins `oscrypto` to a git commit because release 1.3.0 cannot
load against OpenSSL 3. Do not remove the pin without checking that a newer
oscrypto is on PyPI — removing it breaks the image at import time. See
`README.md`.
