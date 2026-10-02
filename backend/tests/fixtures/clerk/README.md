# Test-only Clerk keys

These RSA keys exist only so `tests/clerk_tokens.rs` can sign session tokens
the way Clerk does. They protect nothing and are never used outside tests.

- `signing_key.pem` — signs the "real" tokens. Its public half is `jwks.json`
  under key id `test-key-1`.
- `other_key.pem` — signs forged tokens that must be refused.
