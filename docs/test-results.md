# Test result contract

Each framework adapter returns nonnegative integer `passed`, `failed`, and
`skipped` counts. `scripts/test-results.mjs` owns validation and reporting.
A gate succeeds only after the process exits successfully, a complete result
artifact is read, at least one test passes, and no test fails or skips.
Both Make output modes use this policy. Logs are diagnostics only.

Vitest and Playwright write JSON report files. The pytest plugin writes outcome
counts at session finish, including setup errors and expected failures.
Cargo emits compiler-artifact JSON to discover its test binaries. Each binary
writes a separate libtest result-record file. Full backend runs also execute
Cargo doc tests. Cargo selectors go before `--`; harness arguments go after it.
The stable Rust result-record option `--logfile` is deprecated. It is isolated
in the Cargo adapter because JSON/JUnit harness output still needs nightly.
An upstream removal must fail the gate, never fall back to diagnostic logs.

`scripts/test-command.mjs` owns compact logs and keeps failure artifacts in its
printed temporary directory. Backend and browser runners retain their owned
server/database cleanup and use the same result contract.
