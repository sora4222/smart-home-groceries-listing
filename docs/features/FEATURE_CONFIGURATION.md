# Configuration sources

Settings accept exported values, literals in `.env`, and complete `op://`
references. Exported values, including empty strings, override the file.
Selected references must resolve before startup. Errors name the variable
without values or CLI output. No shell evaluation or resolved file is used.

These rules cover `DATABASE_URL`, supplied `POSTGRES_*` credentials,
`CLERK_SECRET_KEY`, `CLERK_PUBLISHABLE_KEY`, `VITE_CLERK_PUBLISHABLE_KEY`,
`CLERK_JWKS_URL`, and `ALEXA_SKILL_ID`. Loading separate database credentials
does not rewrite the connection URL or change a running database password.

| Entry point | Configuration boundary |
|---|---|
| Compose startup / `make run-env` | `scripts/environment.mjs` merges sources and calls a replaceable resolver; `run-env.mjs` removes `OP_*` credentials before starting its child |
| Backend binary / migrations | `backend/src/environment/` initializes backend settings before threads and logging; only `onepassword.rs` calls the CLI |
| Vite dev/build / `pnpm start` | Shared JavaScript loader resolves `CLERK_*` and `VITE_*`; only Vite's public prefix reaches the browser |
| Alexa `load_settings()` | `alexa_bridge/environment.py` loads bridge settings only; database settings are neither retained nor resolved |

The backend and bridge search the working directory and ancestors for `.env`.
JavaScript entry points use the checkout root. `GROCERY_ENV_FILE` overrides
that path. Each worktree has independent configuration and no shared cache.

Only references need the host's `op` executable. Its desktop integration or
exported `OP_SERVICE_ACCOUNT_TOKEN` selects authentication; a provider error
stops startup without switching methods. Provider access stays outside settings,
HTTP handlers, and business logic. Compose images receive resolved values and
need no CLI. Backend health probes use existing values without unlocking secrets.

Compose explicitly maps backend settings. Raw `env_file: .env` would
reintroduce unresolved references. New backend settings belong in both the
backend startup scope and the Compose environment block.

Tests: `scripts/environment.test.mjs` covers all requested keys, both CLI auth
environments, precedence, scope, redacted errors, and actual Compose parsing.
Rust `tests/environment.rs` tests the provider-independent loader. Alexa's
configuration tests cover direct dotenv startup and both CLI auth environments.
Provider tests use fixtures, never real accounts.

Provisioning: [human setup](../human-setup.md#1password).
Commands: [operations](../using-the-app.md#configuration-and-1password).
