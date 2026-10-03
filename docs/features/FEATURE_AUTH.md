# Feature: Sign-in (Clerk)

Status: **implemented** — the web app signs people in with Clerk (Google, or
email and password), and the backend checks a Clerk session token on every
household route. Steps for people: `docs/human-setup.md` §2 and §6, and
`docs/using-the-app.md` ("Add a household member") — keep those in step
with this spec.

## Household model (from the spec)
- Everyone signed in shares one grocery list. There are no admin roles.
- Who may sign up is decided in Clerk, with its **allowlist** of household
  email addresses. The app itself has no list of people.
- Per-person records (dislikes, preferences) key on the Clerk user id,
  `AuthUser.id` (`sub` in the token).

## What needs a session
| Route | How it is checked |
|---|---|
| Every `/api/*` household route | `AuthUser` extractor: `Authorization: Bearer <token>` |
| `GET /ws` (live counts) | `SocketUser` extractor: the header, or `?token=` (browsers cannot set headers on a WebSocket) |
| `POST /api/intake/alexa` | Alexa bridge shared secret — not a session |
| `POST /api/voice-requests` | webhook shared secret — not a session |
| `/api/store-tab/*` | store-tab shared secret — not a session |
| `GET /api/health` | open |

`backend/tests/auth_required.rs` sends every household route a request with
no token and with a forged one, and expects 401. **A new route goes in its
list.**

## Backend
- `backend/src/auth/mod.rs` — `AuthProvider` trait, `AuthUser` extractor,
  `authenticate()` shared by both extractors, `build_provider()` (the one
  place Clerk is named). Swap provider there only.
- `backend/src/auth/clerk.rs` — verifies RS256 tokens against
  `CLERK_JWKS_URL`, cached for an hour. The algorithm comes from the JWK,
  never the token header. `exp`, `nbf` and `sub` are required.
- `backend/src/auth/socket.rs` — the `/ws` extractor.
- `jsonwebtoken` needs a crypto backend feature (`aws_lc_rs`). Without one it
  **panics** on the first RS256 verification; that was the case before this
  feature and every signed-in request would have crashed.
- Request tracing logs the path only, not the query, so a `?token=` never
  reaches the logs. Clerk session tokens last about a minute anyway.

`backend/tests/clerk_tokens.rs` signs tokens with test-only keys
(`tests/fixtures/clerk/`) and serves their JWKS from a mock server. It covers
a valid token, expired, wrong key, unknown key id, HS256 algorithm
confusion, no subject, and the WebSocket with and without a token.

## Frontend
| File | Job |
|---|---|
| `lib/auth-mode.ts` | `clerk` or `off`, from `VITE_CLERK_PUBLISHABLE_KEY` and `VITE_AUTH_MODE` |
| `lib/auth.ts` | `getAuthToken()`: Clerk's token in the browser; `auth().getToken()` during server rendering |
| `lib/auth-state.ts` | server function: is this request signed in? |
| `lib/sign-in-guard.ts` | pure rules: who goes to `/sign-in` |
| `lib/ws-url.ts` | adds `?token=` to the socket address |
| `start.ts` | adds `clerkMiddleware()` with sign-in on |
| `routes/__root.tsx` | `beforeLoad` guard, before any loader asks the backend |
| `routes/sign-in.$.tsx` | Clerk's `<SignIn />`; goes home with sign-in off |
| `components/auth/` | provider, nav account button, "Sign-in is off" notice |

The web app reads the repository's root `.env` (`envDir: ".."` in
`vite.config.ts`). Clerk's server middleware needs `CLERK_SECRET_KEY` in the
process environment, so the Vite config copies it across for the dev server.

## Test mode (sign-in off)
Real sign-in needs a real Clerk account, so the e2e suite runs with sign-in
off on both sides:

```bash
# backend
DEV_AUTH_BYPASS=true
# web app — forces sign-in off even if .env has a publishable key
VITE_AUTH_MODE=off pnpm dev
```

The web app then shows a "Sign-in is off" line on every page
(`e2e/sign-in.spec.ts`). With sign-in off but the bypass **not** set, the
backend refuses everything — the backend is always the real gate.

## Known gaps
- Not checked against a real Clerk instance from the cloud workspace (no
  account). With a made-up key, the server middleware does start Clerk's
  development "handshake" redirect, so the wiring is live.
- A Clerk **development** instance shows a "Development mode" badge and uses
  Clerk's shared Google credentials. Fine for a household; a production
  instance needs its own Google OAuth app and a domain.
- `azp` (authorised party) is not checked. Tokens arrive only as bearer
  headers from this app, so cookie-based cross-site use does not apply.
