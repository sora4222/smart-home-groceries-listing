# Desktop App (Tauri)

Status: **built for Woolworths** — server window, Woolworths window with a
kept login, fill trolley in the app, open checkout, notifications, tray.
Planned: one tap to place the order, Coles, start at login.
Spec: `docs/features/FEATURE_DESKTOP_APP.md`. Steps for people:
`docs/human-setup.md` (Optional — Desktop app), `docs/using-the-app.md`.

## What it is for
The server cannot log in to the stores. The desktop app runs on the
household's Mac or Windows computer, from the home internet, in the system
webview (WKWebView / WebView2), and keeps its own store login in that
webview's cookies. It talks only to the server and the stores. Voice
intake, PostgreSQL and every grocery rule stay on the server.

## Layout
```
desktop/
├── package.json         # @tauri-apps/cli: pnpm dev / pnpm build / pnpm icon
├── icon-source.svg      # copy of frontend/public/favicon.svg → `pnpm icon`
├── setup/               # bundled first-run page (frontendDist)
└── src-tauri/
    ├── build.rs         # COMMANDS → one allow-<name> permission each
    ├── capabilities/setup.json   # setup page: allow-save-server-url only
    └── src/
        ├── lib.rs       # wiring: plugins, state, commands, close → hide
        ├── settings.rs  # settings.json, parse_server_url (origin only)
        ├── server_access.rs  # runtime capability for the server origin
        ├── stores.rs    # Store: URLs, window label, abilities, owns(url)
        ├── pending_program.rs  # program waiting for the trolley page
        ├── notify.rs    # notification plugin wrapper
        ├── tray.rs      # Open / Open Woolworths / Change server / Quit
        ├── windows/     # main_window.rs, store_window.rs
        └── commands/    # desktop.rs, store.rs, setup.rs
```
Web app side: `frontend/src/lib/desktop/` (`bridge.ts` — the only import of
`@tauri-apps/api`; `commands.ts`; `handoff-notice.ts`), `hooks/useDesktop.ts`,
`useDesktopNotifications.ts`, `useHandoffNotice.ts`,
`components/trolley/desktop-steps.tsx`.

## Rules
- **No second copy of the screens.** The main window loads the server. A
  desktop-only control goes in the web app behind `useDesktop()` /
  `isDesktop()`, and must leave the browser page unchanged.
- **Add a command:** write it in `commands/`, add its name to `COMMANDS` in
  `build.rs`, register it in `lib.rs`, and — only if the server page needs
  it — add `allow-<kebab-name>` to `SERVER_PERMISSIONS` in
  `server_access.rs`. Then a typed wrapper in `lib/desktop/commands.ts`.
- **Never grant a plugin permission to the server origin.** Wrap the plugin
  in an app command that checks its arguments (see `notify`).
- **Store pages:** a program runs only through `PendingPrograms`, only on
  `Store::owns` + the trolley path, and only once. Never read or export the
  store window's cookies.
- **A store's abilities** (`Store::abilities`) stay `false` until its calls
  are captured and tested; the web app hides the buttons from them.
- **Paying:** never automatic. Goal 8 (one tap) must wait for the person's
  tap, use only the card saved on the store account, and never type a card
  number or CVV.
- **Logging:** `log::info!("[area] …")` via `tauri-plugin-log`. Log hosts and
  paths, never a query string, cookie, secret or program text.
- Changing the server address restarts the app (one capability per run).

## Versions (checked 2026-10-02)
| Crate / package | Version |
|---|---|
| `tauri` (features `tray-icon`, `dynamic-acl`) | 2.12.1 |
| `tauri-build` | 2.7.1 |
| `tauri-plugin-notification` | 2.5.1 |
| `tauri-plugin-log` | 2.10.0 |
| `tauri-plugin-single-instance` | 2.5.2 |
| `@tauri-apps/cli`, `@tauri-apps/api` | 2.12.1 |

Tauri 3 is alpha — stay on 2.x. Confirm with Context7 before upgrading.

## Tests and builds
- `make test-desktop` — Rust unit tests (settings, capability, stores,
  pending program, command argument checks). Linux needs
  `libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev`.
- Vitest covers the web app side (`lib/desktop/__tests__/`,
  `hooks/__tests__/useDesktopNotifications.test.ts`,
  `components/trolley/__tests__/desktop-steps.test.tsx`).
- `make desktop-build` — the installer for the computer it runs on (a Mac
  build needs a Mac, a Windows build needs Windows). Unsigned: see the
  human-setup steps for Gatekeeper / SmartScreen.
