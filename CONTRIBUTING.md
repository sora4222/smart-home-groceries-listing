# Contributing

## Software Lifecycle

1. **Set up.** Install Node 22+, pnpm, Rust, uv and Docker. Run `make setup`,
   then `make setup-env`. Fill in `.env` with [`docs/human-setup.md`](docs/human-setup.md).
2. **Develop.** Run `make dev`. Web app edits reload at once; backend and Alexa
   edits rebuild their containers. Ctrl-C or `make dev-stop` stops it and keeps
   the data.
3. **Check.** Run `make check` before each commit. It formats, lints, checks
   types and runs all tests.
4. **Change the database.** Run `make migration-new name="description"`, then
   `make update-db`. This keeps existing records.
5. **Build.** Run `make build` for the server, or `make build-desktop` for the
   desktop app.
6. **Install or update the server.** Run `sudo make install-server`, or
   `make hm-update-server` for a verified release. See
   [`docs/production.md`](docs/production.md).

Use one commit per feature, with a `feat:` or `fix:` prefix. Never commit secrets.
