# Docs — which one to read

## For people using or setting up the app
Plain words, step by step, grouped by place.

| Doc | Read it when |
|---|---|
| [`human-setup.md`](human-setup.md) | Setting the app up the first time |
| [`using-the-app.md`](using-the-app.md) | Adding items, doing a Woolworths shop (in Chrome or the desktop app), changing a delivery time, after an update, when something goes wrong |

## For agents and developers
| Doc | What it holds |
|---|---|
| [`features/`](features/) | One spec per feature: behaviour, API, data, tests |
| [`FEAT_WOOLWORTHS_ACCESS.md`](FEAT_WOOLWORTHS_ACCESS.md) | How Woolworths is reached, what was tried, the live calls |

Agents: steps a person must do go in the two people docs above, written with
the `human-instructions` skill (`.claude/skills/human-instructions/SKILL.md`).
Feature docs link to those steps; they do not repeat them.
