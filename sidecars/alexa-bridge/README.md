# Alexa bridge sidecar

Verifies that an incoming request really came from Amazon, pulls the item and
quantity out of the intent, and posts them to the Rust backend. That is all it
does.

**This is the only Python in the repository.** It exists because Amazon's
request signing and skill request model are implemented in the `ask-sdk`
Python SDK and have no maintained Rust equivalent. Writing the signature
check by hand in Rust would mean reimplementing certificate-chain validation
for a security boundary — using Amazon's own implementation is the safer
trade, at the cost of one small extra process.

## What it does and does not do

| Does | Does not |
|---|---|
| Verify Alexa's signature, cert chain, timestamp and skill id (`ask-sdk`) | Decide anything about the grocery list |
| Extract the `item` and `quantity` slots | Detect duplicates, merge quantities, or accept items |
| Forward them to `POST /api/intake/alexa` | Touch PostgreSQL — it holds no database credentials |
| Speak a confirmation | Suggest, recommend or upsell an item |

Items reach the backend as **pending**. Nothing gets onto the household list
without a person accepting it in the web app — including items Alexa heard
perfectly.

## Request flow

```
"Alexa, add two oat milk to the shopping list"
  → Amazon  → Cloudflare Tunnel → this sidecar :8081/alexa
                                    ├─ ask-sdk verifies signature + timestamp + skill id
                                    ├─ parse_slots(item, quantity)
                                    └─ POST http://backend:8000/api/intake/alexa
                                         X-Bridge-Secret: <ALEXA_BRIDGE_SECRET>
                                         { item, quantity, external_id, raw_text }
                                              → pending request → web app → accept
```

`external_id` is Alexa's own request id. Alexa retries an endpoint it thinks
timed out and reuses that id, so the backend recognises the retry and does not
raise a second confirmation card.

## Configuration

| Variable | Required | Meaning |
|---|---|---|
| `ALEXA_SKILL_ID` | yes | `amzn1.ask.skill.<uuid>`; verified on every request |
| `ALEXA_BRIDGE_SECRET` | yes | Shared secret the backend authenticates this sidecar with |
| `BACKEND_BASE_URL` | no | Defaults to `http://backend:8000` (the Compose service) |
| `BACKEND_TIMEOUT_SECONDS` | no | Defaults to `3.0`, inside Alexa's ~8s window |
| `PORT` | no | Defaults to `8081` (development server only) |
| `LOG_LEVEL` | no | Defaults to `INFO` |

A missing skill id or bridge secret stops the process at startup rather than
running with an effectively unauthenticated endpoint.

## Running it

```bash
make alexa-up          # from the repository root, via Docker Compose
```

Locally, without Docker:

```bash
cd sidecars/alexa-bridge
uv sync                                    # or: python -m venv .venv && pip install -e '.[dev]'
ALEXA_SKILL_ID=amzn1.ask.skill.xxx \
ALEXA_BRIDGE_SECRET=... \
BACKEND_BASE_URL=http://127.0.0.1:8000 \
  uv run python -m alexa_bridge.app
```

## Tests

```bash
make test-alexa        # from the repository root
uv run pytest -q       # here
```

Tests never reach Amazon or the backend: the HTTP session is a stand-in and
the skill request tests assert that an **unsigned** request is refused, which
is the property that matters. Forging a valid signature would require
Amazon's private key.

## The oscrypto pin

`ask-sdk-webservice-support` depends on `certvalidator`, which depends on
`oscrypto`. The newest oscrypto release (1.3.0, 2022) cannot parse OpenSSL 3's
version string and raises `LibraryNotFoundError` on import — so on any current
Linux the official verifier will not even load. The fix is on oscrypto's
master but unreleased, so `pyproject.toml` pins an exact commit.

Remove that pin once oscrypto > 1.3.0 is published. If the pin ever stops
resolving, the fallback is to verify Alexa's signature in the Rust backend
with `x509-parser` and `rsa` and retire this sidecar — the endpoint contract
would not change.

## Skill interaction model

The skill needs one custom intent:

```
AddItemIntent
  "add {quantity} {item} to the shopping list"
  "add {item} to the shopping list"
  "put {item} on the shopping list"
  "add {item}"

  item     → AMAZON.Food  (or a custom GroceryItem slot type)
  quantity → AMAZON.NUMBER
```

`docs/human-setup.md` has the Alexa developer console steps.
