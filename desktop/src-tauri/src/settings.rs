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

/// What the desktop app remembers between runs.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    /// The grocery server's origin, e.g. `https://grocery.example.com`.
    pub server_url: Option<String>,
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

/// Turns what the person typed into the server's origin, or says why not.
///
/// Accepts `http` and `https`, a host, and an optional port. A trailing `/`
/// is fine; anything after it is refused.
pub fn parse_server_url(input: &str) -> Result<String, AddressProblem> {
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
    if !url.username().is_empty() || url.password().is_some() {
        return problem("Leave the login out of the address.");
    }
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        return problem("Use only the start of the address, like https://grocery.example.com");
    }
    Ok(url.origin().ascii_serialization())
}

/// Reads the settings. A missing or unreadable file means "no settings yet":
/// the setup page asks again rather than the app failing to start.
pub fn load(dir: &Path) -> Settings {
    let path = dir.join(SETTINGS_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Settings::default(),
        Err(err) => {
            log::warn!("[settings] cannot read {}: {err}", path.display());
            return Settings::default();
        }
    };
    let settings: Settings = match serde_json::from_str(&text) {
        Ok(settings) => settings,
        Err(err) => {
            log::warn!("[settings] {} is not valid: {err}", path.display());
            return Settings::default();
        }
    };
    // A hand-edited address that is no longer valid is dropped, not trusted.
    match settings.server_url.as_deref().map(parse_server_url) {
        Some(Err(problem)) => {
            log::warn!("[settings] saved server address refused: {problem}");
            Settings::default()
        }
        _ => settings,
    }
}

/// Writes the settings, making the config folder if needed.
pub fn save(dir: &Path, settings: &Settings) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let text = serde_json::to_string_pretty(settings).map_err(io::Error::other)?;
    fs::write(dir.join(SETTINGS_FILE), text)?;
    log::info!("[settings] saved server address {:?}", settings.server_url);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_an_https_origin_and_drops_the_trailing_slash() {
        assert_eq!(
            parse_server_url(" https://grocery.example.com/ "),
            Ok("https://grocery.example.com".to_string())
        );
    }

    #[test]
    fn keeps_a_port_and_accepts_http_for_a_home_network() {
        assert_eq!(
            parse_server_url("http://192.168.1.20:3000"),
            Ok("http://192.168.1.20:3000".to_string())
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
        assert_eq!(load(dir.path()), Settings::default());
    }

    #[test]
    fn saves_then_loads_the_same_address() {
        let dir = tempfile::tempdir().unwrap();
        let settings = Settings {
            server_url: Some("https://grocery.example.com".to_string()),
        };
        save(&dir.path().join("nested"), &settings).unwrap();
        assert_eq!(load(&dir.path().join("nested")), settings);
    }

    #[test]
    fn broken_or_edited_file_means_no_settings() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(SETTINGS_FILE), "{not json").unwrap();
        assert_eq!(load(dir.path()), Settings::default());
        fs::write(
            dir.path().join(SETTINGS_FILE),
            r#"{"server_url":"javascript:alert(1)"}"#,
        )
        .unwrap();
        assert_eq!(load(dir.path()), Settings::default());
    }
}
