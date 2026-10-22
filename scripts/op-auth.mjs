/** Run one 1Password CLI call with the service account first, then the desktop app when allowed. */

/** The inherited environment without stale CLI sessions, ready for desktop-app authentication. */
function desktopEnvironment(inherited) {
  const desktop = { ...inherited, OP_BIOMETRIC_UNLOCK_ENABLED: 'true' };
  for (const key of Object.keys(desktop)) {
    if (key === 'OP_SERVICE_ACCOUNT_TOKEN' || key.startsWith('OP_CONNECT_') || key === 'OP_SESSION' || key.startsWith('OP_SESSION_')) delete desktop[key];
  }
  return desktop;
}

/** Call `attempt(env)`; desktop retry is allowed only outside token-file production use. */
export function withOnePassword(inherited, attempt) {
  const desktop = desktopEnvironment(inherited);
  if (inherited.OP_SERVICE_ACCOUNT_TOKEN) {
    const service = { ...desktop, OP_SERVICE_ACCOUNT_TOKEN: inherited.OP_SERVICE_ACCOUNT_TOKEN, OP_BIOMETRIC_UNLOCK_ENABLED: 'false' };
    delete service.OP_ACCOUNT;
    try { return attempt(service); }
    catch (error) {
      // Production must use the selected token, never an interactive fallback.
      if (inherited.GROCERY_OP_SERVICE_ONLY === '1') throw error;
    }
  }
  return attempt(desktop);
}
