# Documentation

Developer documentation for the React/TanStack Start web app, Rust/Axum API,
PostgreSQL database, Flask Alexa sidecar and Tauri client. It covers provisioning,
watched development, systemd deployment and feature contracts. Required
environment variables are listed in the [root README](../README.md#environment-variables-required)
and component READMEs. The [contribution guide](../CONTRIBUTING.md) maps lifecycle
steps to Make commands.

**Audience:** every doc here is written for software developers and uses
their terminology (`.env` keys, endpoints, intents, Compose profiles, HTTP
status codes). Plain-language explanations for the household belong in chat,
not in these files.

## Setup and operations
Manual steps, ordered by dependency and grouped by place.

| Doc | Read it when |
|---|---|
| [`human-setup.md`](human-setup.md) | Provisioning Clerk, Cloudflare, the Alexa skill, the triage model and the stack for the first time |
| [`production.md`](production.md) | Running the app on the home server as systemd services, without Docker (`make build`, `sudo make install-server`) |
| [`using-the-app.md`](using-the-app.md) | Voice intake (add, remove, reduce, undo), a Woolworths shop, undoing a purchase, after an update, troubleshooting |

## Specs and findings
| Doc | What it holds |
|---|---|
| [`features/`](features/) | One spec per feature: behaviour, API, data, tests |
| [`FEAT_WOOLWORTHS_ACCESS.md`](FEAT_WOOLWORTHS_ACCESS.md) | How Woolworths is reached, what was tried, the live calls |

Agents: manual steps go in the two setup/operations docs above, written with
the `human-instructions` skill (`.claude/skills/human-instructions/SKILL.md`).
Feature docs link to those steps; they do not repeat them.
