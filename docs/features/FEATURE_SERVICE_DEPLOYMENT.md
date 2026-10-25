# Verified native service deployment

Production runs native systemd services, enabled through `grocery.target` at boot.
Reviewed local control code in `deploy/service/` installs runtime bundles without
executing downloaded installation hooks as root. Bundles contain only runtime
files and migrations; they reject traversal, duplicates, unexpected control
files, unsafe paths and checksum mismatches.

`deploy/start.sh` accepts application settings from exports or `--config PATH`.
1Password is optional: `--op-token-file PATH`, `--op-environment-id ID`, or the
corresponding `OP_SERVICE_ACCOUNT_TOKEN`/`OP_ENVIRONMENT_ID` startup exports enable
it. Tokens persist root-only and never enter application services or artifacts.
Omitted inputs retain existing local settings, Environment selection and token.
Environment values override local settings at each service start. Explicit
startup exports become local fallbacks. Database-target/encryption-key changes
require deliberate migration; password rotation for the same target is allowed.
Unknown existing local settings are retained.

Updates do not recreate/reset tables. The installer checks migrations/downgrades,
stops writes, takes a root-only PostgreSQL dump and runs embedded migrations as a
restricted OS/database user. It switches a release symlink and starts services.
Old releases and database backups remain available. Migration/health failures
require deliberate recovery: file rollback does not reverse database migrations.
The SQL heuristic is a guard, not proof arbitrary code cannot change data.
Reviewed migrations and tested backup restoration remain necessary.

Four explicit public frontend settings are selected at runtime and serialized
with HTML-safe escaping. Artifacts need no production secret or hostname.
Supported public settings change on restart. Unlisted `VITE_*`, Clerk secrets
and bootstrap tokens never enter the browser configuration object.

The `Native service install and release` Action builds on Ubuntu 26.04 and tests
real PostgreSQL/systemd installation, update preservation, backup restoration,
destructive-migration rejection, boot enablement and restart. Its matrix covers
file/export configuration without 1Password and fixture service-account inputs
via files/exports. The fake CLI tests the contract; CI has no account secrets.
Successful pushes publish attested stable `release-N.N.N` releases or immutable
`preview-COMMIT` prereleases. Production release commits must belong to main.
PR builds cannot publish. Host/workflow policy, verification and preview isolation
belong to the server repository. Updating root control code is separately reviewed.
Provenance establishes origin, not that application code is harmless.
