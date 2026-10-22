---
name: desktop
description: Change the Tauri desktop connection, native commands, store windows or desktop launch tooling.
---

# Desktop client

Read `docs/features/FEATURE_DESKTOP_APP.md` before changing behavior.
For production URL changes or deployment, read the Tailscale Service section
in `docs/production.md`. The versioned loader reads the Developer
Environment selected by `/etc/grocery/op-environment-id` for both the frontend
build and the services; do not deploy a hand-kept loader overlay. Saved
Environment values alone do not prove the built assets use them, so check them.
`desktop/src-tauri/` owns native commands and settings. Never build a webview
window in a synchronous command or an event handler: it deadlocks on Windows.
Use `async` commands, and run tray actions that create windows on their own
thread (`tray.rs`). The frontend owns all
screens and shared trolley programs. Keep each module responsible for one task.
Queue notifications use the WebSocket `counts_snapshot` as a silent baseline
on every connection. Keep pending and held coverage for the first live increase
and reconnects; an initial HTTP count must not overwrite fresher socket data.
The main webview must disable background throttling because it owns those
listeners while hidden in the tray. Tauri supports this on macOS 14+; do not
claim Windows or Linux background behavior from this setting alone.

One saved frontend origin wins over the runtime `GROCERY_DESKTOP_SERVER_URL`,
then the build default (`startup.rs`; `lib.rs` stays wiring only). Validate
origins before granting commands: only a `settings::ServerOrigin` is granted. Never grant
wildcards, setup commands or plugin permissions to remote pages. Changing the
saved origin restarts the app. Only the configured main-window origin may call
native store commands.

`scripts/desktop.mjs` resolves only the optional public URL and passes the
`desktopEnvironment` and shared `toolchainEnvironment` allowlists to Cargo and Tauri. Never forward backend or
1Password credentials. Development identifiers derive from the real worktree
path; packaged builds retain the installed identity.

`SendToStore` selects native steps by the store's advertised abilities. Keep
Coles on its browser workflow until native support is verified. Both clients
use the same `StoreTab` fill implementation. `fill_operation.rs` is the one
owner of a desktop fill: keep overlap refusal, the result the command returns,
and cleanup when the ticket drops. Store pages get no IPC; the app reads the
program's `window.__groceryFill` progress instead. Desktop completion never opens
checkout automatically. Keep browser checkout confirmation intact.

Run native tests with `make test-desktop`, launcher tests with
`sh scripts/desktop.test.sh`, and the affected frontend tests. Native tests use
`cargo-results.mjs` and the shared verdict policy. `make check-desktop` covers
formatting and Clippy. Build the platform app and inspect it when changing
native startup. Record live tunnel, Clerk and store limitations honestly.
