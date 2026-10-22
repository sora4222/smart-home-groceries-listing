# PostgreSQL runtime and test isolation

## Application connections

`DATABASE_URL` selects PostgreSQL for the Rust backend. The URL may be a
literal or a resolved 1Password reference. Existing database credentials and
volumes are preserved; startup does not rotate passwords or migrate data
between servers.

| Runtime | Connection |
|---|---|
| Backend and PostgreSQL in Compose | `db:5432`; startup adds the `db` profile if missing |
| Backend in Compose, PostgreSQL on the host | `host.docker.internal:<port>`; startup removes the `db` profile |
| Backend directly on the host | `localhost:<port>`, a socket URL, or the configured database host |

`POSTGRES_PORT` controls the development container's published port, not the
backend's internal Docker address. A host-installed server must allow the
Docker network when the backend runs in Compose. See
[bare-metal setup](../human-setup.md#optional-postgresql-installed-on-the-server).
Other Compose profiles and the original URL are preserved. The selected mode
only changes the current invocation, leaving `.env` and existing data intact.

## Test infrastructure

`make test` runs backend, frontend, Alexa, tooling, and browser tests without
starting the application Compose stack. `make test-backend` and `make test-frontend-playwright`
each own a PostgreSQL 16 Testcontainer with random credentials and a dynamically
allocated loopback port. SQLx owns a database per backend integration test.
Test startup waits for database readiness. No application volume or connection
settings are used; the legacy `TEST_DATABASE_URL` override is no longer used.

Browser tests also own local backend and frontend process groups, dynamic
ports, isolated configuration and logs, fake providers, and a matching OAuth
callback URL. Their reset operations can only reach that run's database.
All owned processes and containers stop on success, failure, or SIGINT/SIGTERM.
Ryuk provides additional container cleanup when a runner exits unexpectedly.
Failure logs remain in the printed temporary directory; no resolved
application credentials are written there.

Docker, root `pnpm install`, the frontend dependencies, and Playwright Chromium
are required. Missing infrastructure is a failure, never a silent skip. Worktrees
and concurrent runs get separate databases and ports. CI uses the same runners.

## Ownership and checks

`scripts/test-database.mjs` owns the container lifecycle;
`test-process.mjs` owns subprocess groups; `test-command.mjs` retains full logs
while summarizing test results; `test-environment.mjs` selects safe
toolchain variables; `backend-tests.mjs` runs Cargo; `e2e-tests.mjs` owns the
browser stack. `database-profiles.mjs` selects the application database profile
for `compose-up.sh`.
Tooling tests cover cleanup, cancellation, failed startup, environment
isolation, profile selection, and nonzero command results. Real SQLx and
Playwright suites exercise migrations and database-backed application behavior.
