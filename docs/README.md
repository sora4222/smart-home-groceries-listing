# Docs — which one to read

**Audience:** every doc here is written for software developers and uses
their terminology (`.env` keys, endpoints, intents, Compose profiles, HTTP
status codes). Plain-language explanations for the household belong in chat,
not in these files.

## Setup and operations
Manual steps, ordered by dependency and grouped by place.

| Doc | Read it when |
|---|---|
| [`human-setup.md`](human-setup.md) | Provisioning Clerk, Cloudflare, the Alexa skill, the triage model and the stack for the first time |
| [`using-the-app.md`](using-the-app.md) | Voice intake (add, remove, reduce, undo), a Woolworths shop, undoing a purchase, after an update, troubleshooting |

## Specs and findings
| Doc | What it holds |
|---|---|
| [`features/`](features/) | One spec per feature: behaviour, API, data, tests |
| [`FEAT_WOOLWORTHS_ACCESS.md`](FEAT_WOOLWORTHS_ACCESS.md) | How Woolworths is reached, what was tried, the live calls |

Agents: manual steps go in the two setup/operations docs above, written with
the `human-instructions` skill (`.claude/human-instructions/SKILL.md`).
Feature docs link to those steps; they do not repeat them.
