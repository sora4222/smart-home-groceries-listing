//! The server address the desktop app shows, kept in `settings.json` in the
//! app's config folder.
//!
//! Only an origin is accepted (`https://host[:port]`): the main window loads
//! it, and `server_access` grants that one origin the app's commands, so a
//! path, query or login inside the address would only cause confusion.

use std::fs;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use url::Url;

/// The file name inside the app's config folder.
pub const SETTINGS_FILE: &str = "settings.json";

/// What the desktop app remembers between runs, as written in the file.
/// Only [`load`] and [`save`] see it; the rest of the app gets a checked
/// [`ServerOrigin`].
#[derive(Debug, Default, Serialize, Deserialize)]
struct Settings {
    /// The grocery server's origin, e.g. `https://grocery.example.com`.
    server_url: Option<String>,
}

/// Why a typed server address was refused. The message is shown on the
/// setup page, so it is a plain sentence.
#[derive(Debug, PartialEq, Eq)]
pub struct AddressProblem(pub String);

impl std::fmt::Display for AddressProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A server origin that passed [`parse_server_url`]: `scheme://host[:port]`,
/// with nothing after the host. Holding one proves the address was checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerOrigin(Url);

impl ServerOrigin {
    /// The origin as text, without a trailing `/`: `https://grocery.example.com`.
    pub fn as_str(&self) -> &str {
        self.0.as_str().trim_end_matches('/')
    }

    /// The origin's front page, for the main window to load.
    pub fn into_url(self) -> Url {
        self.0
    }
}

/// Turns what the person typed into the server's origin, or says why not.
///
/// Accepts `http` and `https`, a host, and an optional port. A trailing `/`
/// is fine; anything after it is refused.
pub fn parse_server_url(input: &str) -> Result<ServerOrigin, AddressProblem> {
    let problem = |text: &str| Err(AddressProblem(text.to_string()));
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return problem("Type the server address.");
    }
    let Ok(url) = Url::parse(trimmed) else {
        return problem("That is not a web address. It should start with https://");
    };
    if url.scheme() != "https" && url.scheme() != "http" {
        return problem("The address must start with https:// or http://");
    }
    if url.host_str().is_none_or(str::is_empty) {
        return problem("The address has no server name.");
    }
    if url.host_str().is_some_and(|host| host.contains('*')) {
        return problem("Use one server name, without wildcards.");
    }
    if !url.username().is_empty() || url.password().is_some() {
        return problem("Leave the login out of the address.");
    }
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return problem("Use only the start of the address, like https://grocery.example.com");
    }
    // Parsing already lowercased the host and dropped a default port.
    Ok(ServerOrigin(url))
}

/// Reads the saved server origin. A missing or unreadable file means "no
/// address yet": the setup page asks again rather than the app failing to
/// start.
pub fn load(dir: &Path) -> Option<ServerOrigin> {
    let path = dir.join(SETTINGS_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return None,
        Err(err) => {
            log::warn!("[settings] cannot read {}: {err}", path.display());
            return None;
        }
    };
    let settings: Settings = match serde_json::from_str(&text) {
        Ok(settings) => settings,
        Err(err) => {
            log::warn!("[settings] {} is not valid: {err}", path.display());
            return None;
        }
    };
    // A hand-edited address that is no longer valid is dropped, not trusted.
    match parse_server_url(&settings.server_url?) {
        Ok(origin) => Some(origin),
        Err(problem) => {
            log::warn!("[settings] saved server address refused: {problem}");
            None
        }
    }
}

/// A saved address takes priority over a launch or build default.
/// Invalid defaults return an error so startup can show setup without granting access.
pub fn starting_url(
    saved: Option<ServerOrigin>,
    runtime: Option<&str>,
    bundled: Option<&str>,
) -> Result<Option<ServerOrigin>, AddressProblem> {
    if saved.is_some() {
        return Ok(saved);
    }
    runtime
        .filter(|value| !value.trim().is_empty())
        .or_else(|| bundled.filter(|value| !value.trim().is_empty()))
        .map(parse_server_url)
        .transpose()
}

/// Saves `origin` as the server address, making the config folder if needed.
pub fn save(dir: &Path, origin: &ServerOrigin) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let settings = Settings {
        server_url: Some(origin.as_str().to_string()),
    };
    let text = serde_json::to_string_pretty(&settings).map_err(io::Error::other)?;
    fs::write(dir.join(SETTINGS_FILE), text)?;
    log::info!("[settings] saved server address {}", origin.as_str());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_an_https_origin_and_drops_the_trailing_slash() {
        let origin = parse_server_url(" https://grocery.example.com/ ").unwrap();
        assert_eq!(origin.as_str(), "https://grocery.example.com");
        assert_eq!(origin.into_url().as_str(), "https://grocery.example.com/");
    }

    #[test]
    fn keeps_a_port_and_accepts_http_for_a_home_network() {
        assert_eq!(
            parse_server_url("http://192.168.1.20:3000")
                .unwrap()
                .as_str(),
            "http://192.168.1.20:3000"
        );
    }

    #[test]
    fn refuses_other_schemes_paths_logins_and_blanks() {
        for bad in [
            "",
            "grocery.example.com",
            "ftp://grocery.example.com",
            "file:///etc/passwd",
            "https://grocery.example.com/order",
            "https://grocery.example.com/?a=1",
            "https://me:secret@grocery.example.com",
        ] {
            assert!(parse_server_url(bad).is_err(), "should refuse {bad:?}");
        }
    }

    #[test]
    fn missing_file_means_no_settings() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(load(dir.path()), None);
    }

    #[test]
    fn saves_then_loads_the_same_address() {
        let dir = tempfile::tempdir().unwrap();
        let origin = parse_server_url("https://grocery.example.com").unwrap();
        save(&dir.path().join("nested"), &origin).unwrap();
        assert_eq!(load(&dir.path().join("nested")), Some(origin));
    }

    #[test]
    fn broken_or_edited_file_means_no_settings() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(SETTINGS_FILE), "{not json").unwrap();
        assert_eq!(load(dir.path()), None);
        fs::write(
            dir.path().join(SETTINGS_FILE),
            r#"{"server_url":"javascript:alert(1)"}"#,
        )
        .unwrap();
        assert_eq!(load(dir.path()), None);
    }

    #[test]
    fn defaults_support_tailscale_and_cloudflare_but_saved_settings_win() {
        let start = |saved: Option<&str>, runtime, bundled| {
            let saved = saved.map(|text| parse_server_url(text).unwrap());
            starting_url(saved, runtime, bundled)
                .unwrap()
                .map(|origin| origin.as_str().to_string())
        };
        assert_eq!(
            start(None, Some("http://100.91.154.126:3000"), None),
            Some("http://100.91.154.126:3000".into())
        );
        assert_eq!(
            start(None, None, Some("https://groceries.example.com")),
            Some("https://groceries.example.com".into())
        );
        let saved = "https://server.tailnet.ts.net";
        assert_eq!(
            start(Some(saved), Some("invalid"), None),
            Some(saved.into())
        );
        assert_eq!(start(None, Some(" "), None), None);
        assert!(starting_url(None, Some("https://*.example.com"), None).is_err());
        assert!(starting_url(None, Some("https://%2a.example.com"), None).is_err());
    }

    #[test]
    fn normalizes_hand_edited_settings_before_granting_access() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(SETTINGS_FILE),
            r#"{"server_url":" https://Grocery.example.com/ "}"#,
        )
        .unwrap();
        assert_eq!(
            load(dir.path()).map(|origin| origin.as_str().to_string()),
            Some("https://grocery.example.com".into())
        );
    }
}
