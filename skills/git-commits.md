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
When a task contains multiple features, **commit after each one** — do not
batch. This allows individual rollbacks and keeps CI meaningful.

## How to commit
```bash
# Stage only the files for this feature
git add backend/src/routes/voice.rs backend/src/services/voice/ backend/tests/voice_requests.rs
git commit -m "feat: voice request confirmation queue"

# Then move to the next feature
git add frontend/src/routes/pending.tsx frontend/src/components/pending/
git commit -m "feat: pending requests UI with accept/reject"
```

Never `git add .` across multiple features.

Watch what `git add backend/` sweeps in: `backend/target/` is gitignored, but
`Cargo.lock` is not and **must** be committed — the Docker build runs
`cargo build --locked` and fails without it.

## After finishing a task
```bash
git add skills/<relevant>.md
git commit -m "docs: update <area> skill after <feature>"
```

Skill file updates always get their own `docs:` commit.

## Examples
```
feat: rust axum backend replacing fastapi
feat: alexa intake endpoint with retry deduplication
feat: alexa bridge sidecar using ask-sdk
feat: woolworths internal XHR price fetch
feat: split-store cost optimisation
fix: duplicate item check on voice confirmation
fix: cap merged quantity at the column's maximum
refactor: split voice service into rules and repository
test: websocket event fan-out over a real socket
chore: pin oscrypto to the commit that supports openssl 3
docs: update store-integration skill for chromiumoxide
```
