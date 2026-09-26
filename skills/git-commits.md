# Git Commits

## Format
```
feat: <short description>      # new feature or behaviour
fix: <short description>       # bug fix
refactor: <short description>  # no behaviour change
test: <short description>      # tests only
docs: <short description>      # documentation / skill files
chore: <short description>     # build, deps, config, Makefile
```

## Rule: one commit per feature
When a task contains multiple features, **commit after each one** — do not batch.
This allows individual rollbacks and keeps CI meaningful.

## How to commit
```bash
# Stage only the files for this feature
git add backend/app/routes/voice.py backend/app/services/voice.py tests/integration/test_voice.py
git commit -m "feat: voice request confirmation queue"

# Then move to the next feature
git add frontend/src/routes/pending.tsx frontend/src/components/pending/
git commit -m "feat: pending requests UI with accept/reject"
```

Never `git add .` across multiple features.

## After finishing a task
```bash
git add skills/<relevant>.md
git commit -m "docs: update <area> skill after <feature>"
```

Skill file updates always get their own `docs:` commit.

## Examples
```
feat: woolworths internal XHR price fetch
feat: split-store cost optimisation
feat: Akamai cookie session reuse for Playwright
fix: duplicate item check on voice confirmation
refactor: extract order optimiser into service layer
test: integration tests for /api/orders endpoint
chore: add migration-new make target
docs: update store-integration skill with curl_cffi pattern
```
