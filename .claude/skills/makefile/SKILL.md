---
name: makefile
description: Maintain verified native deployment Make targets.
---

`install-server` calls `deploy/start.sh` with file-path/identifier arguments in
`args`. Bootstrap tokens come only from exports/protected input files. Runtime
bundles contain no helpers, units or config. `hm-update-server` calls the installed
verified updater and never pulls/builds source on production. `package-server`
creates the data-only artifact. Preserve migration checks, database backups,
config values and startup Environment overrides. Public frontend settings use
the explicit runtime allowlist. Add tests to the native deployment suite.
